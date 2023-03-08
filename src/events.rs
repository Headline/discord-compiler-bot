use serenity::{
    async_trait,
    collector::CollectReaction,
    framework::{standard::macros::hook, standard::CommandResult, standard::DispatchError},
    model::{
        application::interaction::Interaction, channel::Message, channel::ReactionType,
        event::MessageUpdateEvent, gateway::Ready, guild::Guild, id::ChannelId, id::GuildId,
        id::MessageId, prelude::UnavailableGuild,
    },
    prelude::*,
};
use std::env;

use chrono::{DateTime, Utc};
use serenity::model::application::component::ButtonStyle;

use crate::{
    cache::*,
    commands::compile::handle_request,
    managers::compilation::RequestHandler,
    utls::{
        discordhelpers,
        discordhelpers::embeds,
        discordhelpers::interactions::send_error_msg,
        parser::{get_message_attachment, shortname_to_qualified},
    },
};

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx: &Context);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx: &Context) {
        let data = ctx.data.read().await;
        let server_count = {
            let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            let guild_count = stats.get_boot_vec_sum();
            stats.post_servers(guild_count).await;
            stats.server_count()
        };

        // lock the shard manager to update our presences
        let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
        discordhelpers::send_global_presence(&shard_manager, server_count).await;
        info!("Ready in {} guilds", server_count);

        // register commands globally in release
        if !cfg!(debug_assertions) {
            let mut cmd_mgr = data.get::<CommandCache>().unwrap().write().await;
            cmd_mgr.register_commands_global(ctx).await;
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        let data = ctx.data.read().await;

        // in debug, we'll register on a guild-per-guild basis
        if cfg!(debug_assertions) {
            let mut cmd_mgr = data.get::<CommandCache>().unwrap().write().await;
            cmd_mgr.register_commands_guild(&ctx, &guild).await;
        }

        let now: DateTime<Utc> = Utc::now();
        if guild.joined_at.unix_timestamp() + 30 > now.timestamp() {
            // post new server to join log
            let id;
            {
                let info = data.get::<ConfigCache>().unwrap().read().await;
                id = info.get("BOT_ID").unwrap().parse::<u64>().unwrap();

                if let Some(log) = info.get("JOIN_LOG") {
                    if let Ok(id) = log.parse::<u64>() {
                        let emb = embeds::build_join_embed(&guild);
                        discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
                    }
                }
            }

            // publish/queue new server to stats
            let (server_count, shard_count) = {
                let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
                stats.new_server().await;
                (stats.server_count(), stats.shard_count())
            };

            // ensure we're actually loaded in before we start posting our server counts
            if server_count > 0 {
                let new_stats = dbl::types::ShardStats::Cumulative {
                    server_count,
                    shard_count: Some(shard_count),
                };

                if let Some(dbl_cache) = data.get::<DblCache>() {
                    let dbl = dbl_cache.read().await;
                    if let Err(e) = dbl.update_stats(id, new_stats).await {
                        warn!("Failed to post stats to dbl: {}", e);
                    }
                }

                // update guild count in presence
                let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
                discordhelpers::send_global_presence(&shard_manager, server_count).await;
            }

            info!("Joining {}", guild.name);

            if let Some(system_channel) = guild.system_channel_id {
                let _ = embeds::dispatch_embed(
                    &ctx.http,
                    system_channel,
                    embeds::build_welcome_embed(),
                )
                .await;
            }
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild) {
        let data = ctx.data.read().await;

        // post new server to join log
        let info = data.get::<ConfigCache>().unwrap().read().await;
        let id = info.get("BOT_ID").unwrap().parse::<u64>().unwrap(); // used later
        if let Some(log) = info.get("JOIN_LOG") {
            if let Ok(join_id) = log.parse::<u64>() {
                let emb = embeds::build_leave_embed(&incomplete.id);
                discordhelpers::manual_dispatch(ctx.http.clone(), join_id, emb).await;
            }
        }

        // publish/queue new server to stats
        let (server_count, shard_count) = {
            let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            stats.leave_server().await;
            (stats.server_count(), stats.shard_count())
        };

        // ensure we're actually loaded in before we start posting our server counts
        if server_count > 0 {
            let new_stats = dbl::types::ShardStats::Cumulative {
                server_count,
                shard_count: Some(shard_count),
            };

            if let Some(dbl_cache) = data.get::<DblCache>() {
                let dbl = dbl_cache.read().await;
                if let Err(e) = dbl.update_stats(id, new_stats).await {
                    warn!("Failed to post stats to dbl: {}", e);
                }
            }

            // update guild count in presence
            let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
            discordhelpers::send_global_presence(&shard_manager, server_count).await;
        }

        info!("Leaving {}", &incomplete.id);
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        if !new_message.attachments.is_empty() {
            if let Ok((code, language)) = get_message_attachment(&new_message.attachments).await {
                let data = ctx.data.read().await;
                let target = {
                    let cm = data.get::<CompilerCache>().unwrap().read().await;
                    cm.resolve_target(shortname_to_qualified(&language))
                };

                if !matches!(target, RequestHandler::None) {
                    let reaction = {
                        let botinfo = data.get::<ConfigCache>().unwrap().read().await;
                        if let Some(id) = botinfo.get("LOGO_EMOJI_ID") {
                            let name = botinfo
                                .get("LOGO_EMOJI_NAME")
                                .expect("Unable to find loading emoji name")
                                .clone();
                            discordhelpers::build_reaction(id.parse::<u64>().unwrap(), &name)
                        } else {
                            ReactionType::Unicode(String::from("💻"))
                        }
                    };

                    if new_message
                        .react(&ctx.http, reaction.clone())
                        .await
                        .is_err()
                    {
                        return;
                    }

                    let collector = CollectReaction::new(ctx.clone())
                        .message_id(new_message.id)
                        .timeout(core::time::Duration::new(30, 0))
                        .filter(move |r| r.emoji.eq(&reaction))
                        .await;
                    let _ = new_message.delete_reactions(&ctx.http).await;
                    if collector.is_some() {
                        let prefix = env::var("BOT_PREFIX").expect("Bot prefix is not set!");
                        let (emb, details) = match handle_request(
                            ctx.clone(),
                            format!("{}compile\n```{}\n{}\n```", prefix, language, code),
                            new_message.author.clone(),
                            &new_message,
                        )
                        .await
                        {
                            Ok((emb, details)) => (emb, details),
                            Err(e) => {
                                let emb = embeds::build_fail_embed(
                                    &new_message.author,
                                    &format!("{}", e),
                                );

                                let sent_fail =
                                    embeds::dispatch_embed(&ctx.http, new_message.channel_id, emb)
                                        .await;
                                if let Ok(sent) = sent_fail {
                                    let mut message_cache =
                                        data.get::<MessageCache>().unwrap().lock().await;
                                    message_cache.insert(
                                        new_message.id.0,
                                        MessageCacheEntry::new(sent, new_message),
                                    );
                                }
                                return;
                            }
                        };

                        // Send our final embed
                        let mut new_msg = embeds::embed_message(emb);
                        let data = ctx.data.read().await;
                        if let Some(link_cache) = data.get::<LinkAPICache>() {
                            if let Some(b64) = details.base64 {
                                let long_url = format!("https://godbolt.org/clientstate/{}", b64);
                                let link_cache_lock = link_cache.read().await;
                                if let Some(url) = link_cache_lock.get_link(long_url).await {
                                    new_msg.components(|cmp| {
                                        cmp.create_action_row(|row| {
                                            row.create_button(|btn| {
                                                btn.style(ButtonStyle::Link)
                                                    .url(url)
                                                    .label("View on godbolt.org")
                                            })
                                        })
                                    });
                                }
                            }
                        }

                        let _ = new_message
                            .channel_id
                            .send_message(&ctx.http, |e| {
                                *e = new_msg.clone();
                                e
                            })
                            .await;
                    }
                }
            }
        }
    }

    async fn message_delete(
        &self,
        ctx: Context,
        _channel_id: ChannelId,
        id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        let maybe_message = {
            let data = ctx.data.read().await;
            let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
            message_cache.remove(id.as_u64())
        };

        if let Some(msg) = maybe_message {
            let _ = msg.our_msg.delete(ctx.http).await;
        }
    }

    async fn message_update(&self, ctx: Context, new_data: MessageUpdateEvent) {
        let maybe_message = {
            let data = ctx.data.read().await;
            let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
            message_cache.get_mut(&new_data.id.0).map(|msg| msg.clone())
        };

        if let Some(msg) = maybe_message {
            if let Some(new_msg) = new_data.content {
                if let Some(author) = new_data.author {
                    discordhelpers::handle_edit(
                        &ctx,
                        new_msg,
                        author,
                        msg.our_msg.clone(),
                        msg.original_msg.clone(),
                    )
                    .await;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);
        let total_shards_to_spawn = ready.shard.unwrap()[1];

        let shard_count = {
            let data = ctx.data.read().await;
            let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            // occasionally we can have a ready event fire well after execution
            // this check prevents us from double calling all_shards_ready
            if stats.shard_count() + 1 > total_shards_to_spawn {
                info!("Skipping duplicate ready event...");
                return;
            }

            let guild_count = ready.guilds.len() as u64;
            stats.add_shard(guild_count);

            // insert avatar at first opportunity
            if stats.shard_count() == 1 {
                let mut info = data.get::<ConfigCache>().unwrap().write().await;
                info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());
            }
            stats.shard_count()
        };

        if shard_count == total_shards_to_spawn {
            self.all_shards_ready(&ctx).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd_result = {
                let data_read = ctx.data.read().await;
                let commands = data_read.get::<CommandCache>().unwrap().read().await;
                commands.on_command(&ctx, &command).await
            };

            match cmd_result {
                Ok(_) => {}
                Err(e) => {
                    // in order to respond to messages with errors, we'll first try to
                    // send an edit, and if that fails we'll pivot to create a new interaction
                    // response
                    let fail_embed = embeds::build_fail_embed(&command.user, &e.to_string());
                    if send_error_msg(&ctx, &command, false, fail_embed.clone())
                        .await
                        .is_err()
                    {
                        warn!("Sending new integration for error: {}", e);
                        let _ = send_error_msg(&ctx, &command, true, fail_embed.clone()).await;
                    }
                }
            }
        }
    }
}

#[hook]
pub async fn before(ctx: &Context, msg: &Message, _: &str) -> bool {
    // we'll go with 0 if we couldn't grab guild id
    let mut guild_id = 0;
    if let Some(id) = msg.guild_id {
        guild_id = id.0;
    }

    let (author_blocked, guild_blocked) = {
        let data = ctx.data.read().await;
        let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
        if stats.should_track() {
            stats.post_request().await;
        }
        let blocklist = data.get::<BlocklistCache>().unwrap().read().await;
        (
            blocklist.contains(msg.author.id.0),
            blocklist.contains(guild_id),
        )
    };

    // check user against our blocklist

    if author_blocked || guild_blocked {
        let emb = embeds::build_fail_embed(
            &msg.author,
            "This server or your user is blocked from executing commands.
        This may have happened due to abuse, spam, or other reasons.
        If you feel that this has been done in error, request an unban in the support server.",
        );

        let _ = embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await;
        if author_blocked {
            warn!("Blocked user {} [{}]", msg.author.tag(), msg.author.id.0);
        } else {
            warn!("Blocked guild {}", guild_id);
        }
        return false;
    }

    true
}

#[hook]
pub async fn after(
    ctx: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    let data = ctx.data.read().await;

    if let Err(e) = command_result {
        let emb = embeds::build_fail_embed(&msg.author, &format!("{}", e));
        let sent_fail = embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await;
        if let Ok(sent) = sent_fail {
            let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
            message_cache.insert(msg.id.0, MessageCacheEntry::new(sent, msg.clone()));
        }
    }

    // push command executed to api
    let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.command_executed(command_name, msg.guild_id).await;
    }
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _: &str) {
    if let DispatchError::Ratelimited(_) = error {
        let emb = embeds::build_fail_embed(&msg.author, "You are sending requests too fast!");
        let _ = embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await;
    }
}
