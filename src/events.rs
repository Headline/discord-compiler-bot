use serenity::{
    framework::standard::DispatchError,
    framework::standard::CommandResult,
    framework::standard::macros::hook,
    async_trait,
    model::channel::Message,
    model::guild::Guild,
    model::id::ChannelId,
    model::id::MessageId,
    model::gateway::Ready,
    prelude::*,
    model::id::{GuildId},
    model::event::{MessageUpdateEvent},
    model::channel::{ReactionType},
    collector::CollectReaction,
    model::interactions::{Interaction}
};
use serenity::model::prelude::UnavailableGuild;

use tokio::sync::MutexGuard;

use chrono::{DateTime, Utc};

use crate::{
    utls::discordhelpers::embeds,
    cache::*,
    utls::discordhelpers,
    managers::stats::StatsManager,
    managers::compilation::RequestHandler,
    commands::compile::handle_request,
    utls::discordhelpers::embeds::embed_message,
    utls::discordhelpers::interactions::send_error_msg,
    utls::parser::{get_message_attachment, shortname_to_qualified}
};

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx: &Context, stats: & mut MutexGuard<'_, StatsManager>, ready : &Ready);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx: &Context, stats: & mut MutexGuard<'_, StatsManager>, ready : &Ready) {
        let data = ctx.data.read().await;
        let mut info = data.get::<ConfigCache>().unwrap().write().await;
        info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());

        let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
        let guild_count = stats.get_boot_vec_sum();

        stats.post_servers(guild_count).await;

        discordhelpers::send_global_presence(&shard_manager, stats.server_count()).await;

        info!("Ready in {} guilds", stats.server_count());
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        let data = ctx.data.read().await;

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
            let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            stats.new_server().await;

            // ensure we're actually loaded in before we start posting our server counts
            if stats.server_count() > 0
            {
                let new_stats = dbl::types::ShardStats::Cumulative {
                    server_count: stats.server_count(),
                    shard_count: Some(stats.shard_count())
                };

                let dbl = data.get::<DblCache>().unwrap().read().await;
                if let Err(e) = dbl.update_stats(id, new_stats).await {
                    warn!("Failed to post stats to dbl: {}", e);
                }

                // update guild count in presence
                let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
                discordhelpers::send_global_presence(&shard_manager, stats.server_count()).await;
            }

            info!("Joining {}", guild.name);

            if let Some(system_channel) = guild.system_channel_id {
                let mut message = embeds::embed_message(embeds::build_welcome_embed());
                let _ = system_channel.send_message(&ctx.http, |_| &mut message).await;
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
        let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
        stats.leave_server().await;

        // ensure we're actually loaded in before we start posting our server counts
        if stats.server_count() > 0
        {
            let new_stats = dbl::types::ShardStats::Cumulative {
                server_count: stats.server_count(),
                shard_count: Some(stats.shard_count())
            };

            let dbl = data.get::<DblCache>().unwrap().read().await;
            if let Err(e) = dbl.update_stats(id, new_stats).await {
                warn!("Failed to post stats to dbl: {}", e);
            }

            // update guild count in presence
            let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
            discordhelpers::send_global_presence(&shard_manager, stats.server_count()).await;
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

                if !matches!(target,  RequestHandler::None) {
                    let reaction = {
                        let botinfo = data.get::<ConfigCache>().unwrap().read().await;
                        if let Some(id) = botinfo.get("LOGO_EMOJI_ID") {
                            let name = botinfo.get("LOGO_EMOJI_NAME").expect("Unable to find loading emoji name").clone();
                            discordhelpers::build_reaction(id.parse::<u64>().unwrap(), &name)
                        }
                        else {
                            ReactionType::Unicode(String::from("💻"))
                        }
                    };

                    if let Err(_) = new_message.react(&ctx.http, reaction.clone()).await {
                        return;
                    }

                    let collector = CollectReaction::new(ctx.clone())
                        .message_id(new_message.id)
                        .timeout(core::time::Duration::new(30, 0))
                        .filter(move |r| r.emoji.eq(&reaction)).await;
                    let _ = new_message.delete_reactions(&ctx.http).await;
                    if let Some(_) = collector {
                        let emb = match handle_request(ctx.clone(), format!(";compile\n```{}\n{}\n```", language, code), new_message.author.clone(), &new_message).await {
                            Ok(emb) => emb,
                            Err(e) => {
                                let emb = embeds::build_fail_embed(&new_message.author, &format!("{}", e));
                                let mut emb_msg = embeds::embed_message(emb);
                                if let Ok(sent) = new_message
                                    .channel_id
                                    .send_message(&ctx.http, |_| &mut emb_msg)
                                    .await
                                {
                                    let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
                                    message_cache.insert(new_message.id.0, MessageCacheEntry::new(sent, new_message));
                                }
                                return;
                            }
                        };
                        let mut emb_msg = embed_message(emb);
                        emb_msg.reference_message(&new_message);
                        let _= new_message
                            .channel_id
                            .send_message(&ctx.http, |_| &mut emb_msg)
                            .await;

                    }
                }
            }
        }
    }

    async fn message_delete(&self, ctx: Context, _channel_id: ChannelId, id: MessageId, _guild_id: Option<GuildId>) {
        let data = ctx.data.read().await;
        let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
        if let Some(msg) = message_cache.get_mut(id.as_u64()) {
            if msg.our_msg.delete(ctx.http).await.is_err() {
                // ignore for now
            }
            message_cache.remove(id.as_u64());
        }
    }

    async fn message_update(&self, ctx: Context, new_data: MessageUpdateEvent) {
        let data = ctx.data.read().await;
        let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
        if let Some(msg) = message_cache.get_mut(&new_data.id.0) {
            if let Some(new_msg) = new_data.content {
                if let Some (author) = new_data.author {
                    discordhelpers::handle_edit(&ctx, new_msg, author, msg.our_msg.clone(), msg.original_msg.clone()).await;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);

        {
            let data = ctx.data.read().await;
            let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            // occasionally we can have a ready event fire well after execution
            // this check prevents us from double calling all_shards_ready
            let total_shards_to_spawn = ready.shard.unwrap()[1];
            if stats.shard_count() + 1 > total_shards_to_spawn {
                info!("Skipping duplicate ready event...");
                return;
            }

            let guild_count = ready.guilds.len() as u64;
            stats.add_shard(guild_count);

            if stats.shard_count() == total_shards_to_spawn {
                self.all_shards_ready(&ctx, &mut stats, &ready).await;
            }
        }

        tokio::task::spawn(async move {
            let ctx = ctx.clone();
            let data = ctx.data.read().await;
            let cmd_mgr = data.get::<CommandCache>().unwrap().read().await;
            cmd_mgr.register_commands(&ctx).await;
            info!("[Shard {}] Registered commands", ctx.shard_id);
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let data_read = ctx.data.read().await;
            let commands = data_read.get::<CommandCache>().unwrap().read().await;
            match commands.on_command(&ctx, &command).await {
                Ok(_) => {}
                Err(e) => {
                    // in order to respond to messages with errors, we'll first try to
                    // send an edit, and if that fails we'll pivot to create a new interaction
                    // response
                    let fail_embed = embeds::build_fail_embed(&command.user, &e.to_string());
                    if let Err(_) = send_error_msg(&ctx, &command, false, fail_embed.clone()).await {
                        warn!("Sending new integration for error: {}", e);
                        let _ = send_error_msg(&ctx, &command, true, fail_embed.clone()).await;
                    }
                }
            }
        }
    }
}

#[hook]
pub async fn before(ctx: &Context, msg : &Message, _: &str) -> bool {
    let data = ctx.data.read().await;
    {
        let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
        if stats.should_track() {
            stats.post_request().await;
        }
    }

    // we'll go with 0 if we couldn't grab guild id
    let mut guild_id = 0;
    if let Some(id) = msg.guild_id {
        guild_id = id.0;
    }

    // check user against our blocklist
    {
        let blocklist = data.get::<BlocklistCache>().unwrap().read().await;
        let author_blocklisted = blocklist.contains(msg.author.id.0);
        let guild_blocklisted = blocklist.contains(guild_id);

        if author_blocklisted || guild_blocklisted {
            let emb = embeds::build_fail_embed(&msg.author,
       "This server or your user is blocked from executing commands.
            This may have happened due to abuse, spam, or other reasons.
            If you feel that this has been done in error, request an unban in the support server.");

            let mut emb_msg = embeds::embed_message(emb);
            if msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await.is_ok() {
                if author_blocklisted {
                    warn!("Blocked user {} [{}]", msg.author.tag(), msg.author.id.0);
                }
                else {
                    warn!("Blocked guild {}", guild_id);
                }
            }
            return false;
        }
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
        let mut emb_msg = embeds::embed_message(emb);
        if let Ok(sent) = msg
            .channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await
        {
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
        let emb =
            embeds::build_fail_embed(&msg.author, "You are sending requests too fast!");
        let mut emb_msg = embeds::embed_message(emb);
        if msg
            .channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await
            .is_err()
        {}
    }
}
