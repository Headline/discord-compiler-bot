use serenity::{
    async_trait,
    framework::{standard::macros::hook, standard::CommandResult},
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::Activity,
        gateway::Ready,
        guild::{Guild, GuildUnavailable},
        user::OnlineStatus,
    },
    prelude::*,
};

use crate::cache::*;
use crate::utls::discordhelpers;
use chrono::{DateTime, Duration, Utc};
use serenity::framework::standard::DispatchError;
use dbl::types::ShardStats;

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx: &Context, data: &TypeMap, shards: &[u64]);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx: &Context, data: &TypeMap, shards: &[u64]) {
        let sum: u64 = shards.iter().sum();

        // update stats
        let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
        if stats.should_track() {
            stats.post_servers(sum).await;
        }

        let presence_str = format!("in {} servers | ;invite", sum);
        ctx.set_presence(Some(Activity::playing(&presence_str)), OnlineStatus::Online).await;

        info!("{} shard(s) ready", shards.len());
        debug!("Existing in {} guilds", sum);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        let now: DateTime<Utc> = Utc::now();
        if guild.joined_at + Duration::seconds(30) > now {
            let data = ctx.data.read().await;

            // publish new server to stats
            {
                let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
                if stats.should_track() {
                    stats.new_server().await;
                }
            }


            // post new server to join log
            let id;
            {
                let info = data.get::<ConfigCache>().unwrap().read().await;
                id = info.get("BOT_ID").unwrap().parse::<u64>().unwrap();

                if let Some(log) = info.get("JOIN_LOG") {
                    if let Ok(id) = log.parse::<u64>() {
                        let emb = discordhelpers::build_join_embed(&guild);
                        discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
                    }
                }
            }

            // update DBL site
            let sum : u64;
            {
                let mut shard_info = data.get::<ServerCountCache>().unwrap().lock().await;
                let index = ctx.shard_id as usize;
                shard_info[index] += 1;
                sum = shard_info.iter().sum();

                let dbl = data.get::<DBLCache>().unwrap().read().await;
                let new_stats = ShardStats::Shards {
                    shards: shard_info.clone(),
                };

                if dbl.update_stats(id, new_stats).await.is_err() {
                    warn!("Failed to post stats to dbl");
                }
            }

            // update shard guild count & presence
            let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
            discordhelpers::send_global_presence(&shard_manager, sum).await;

            info!("Joining {}", guild.name);
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable) {
        let data = ctx.data.read().await;
        let mut stats = data.get::<StatsManagerCache>().unwrap().lock().await;
        if stats.should_track() {
            stats.leave_server().await;
        }

        // post new server to join log
        let info = data.get::<ConfigCache>().unwrap().read().await;
        let id = info.get("BOT_ID").unwrap().parse::<u64>().unwrap();
        if let Some(log) = info.get("JOIN_LOG") {
            if let Ok(id) = log.parse::<u64>() {
                let emb = discordhelpers::build_leave_embed(&incomplete.id);
                discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
            }
        }

        // update DBL site
        let sum : u64;
        {
            let mut shard_info = data.get::<ServerCountCache>().unwrap().lock().await;
            let index = ctx.shard_id as usize;
            shard_info[index] -= 1;
            sum = shard_info.iter().sum();

            let dbl = data.get::<DBLCache>().unwrap().read().await;
            let new_stats = ShardStats::Shards {
                shards: shard_info.clone(),
            };

            if dbl.update_stats(id, new_stats).await.is_err() {
                warn!("Failed to post stats to dbl");
            }
        }

        let shard_manager = data.get::<ShardManagerCache>().unwrap().lock().await;
        discordhelpers::send_global_presence(&shard_manager, sum).await;
        info!("Leaving {}", &incomplete.id);
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);
        let data = ctx.data.read().await;
        let mut info = data.get::<ConfigCache>().unwrap().write().await;
        info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());

        let mut shard_info = data.get::<ServerCountCache>().unwrap().lock().await;
        shard_info.push(ready.guilds.len() as u64);

        if shard_info.len() == ready.shard.unwrap()[1] as usize {
            self.all_shards_ready(&ctx, &data, &shard_info).await;
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
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
            let emb = discordhelpers::build_fail_embed(&msg.author,
       "This server or user is blocked from executing commands.
            This may have happened due to abuse, spam, or other reasons.
            If you feel that this has been done in error, request an unban in the support server.");

            let mut emb_msg = discordhelpers::embed_message(emb);
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
    use crate::cache::StatsManagerCache;
    if let Err(e) = command_result {
        let emb = discordhelpers::build_fail_embed(&msg.author, &format!("{}", e));
        let mut emb_msg = discordhelpers::embed_message(emb);
        if msg
            .channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await
            .is_err()
        {
            // missing permissions, just ignore...
        }
    }

    let data = ctx.data.read().await;
    let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.command_executed(command_name).await;
    }
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(_) = error {
        let emb =
            discordhelpers::build_fail_embed(&msg.author, "You are sending requests too fast!");
        let mut emb_msg = discordhelpers::embed_message(emb);
        if msg
            .channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await
            .is_err()
        {}
    }
}
