use serenity::{
    async_trait,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use serenity::model::gateway::Activity;
use serenity::model::user::{OnlineStatus, User};

use crate::cache::*;
use serenity::model::guild::{Guild, Member};
use crate::utls::discordhelpers::DiscordHelpers;
use serenity::model::id::GuildId;

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx : &Context, data : &TypeMap, shards : &Vec<usize>);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx : &Context, data : &TypeMap, shards : &Vec<usize>) {
        let sum : usize = shards.iter().sum();

        // update stats
        let mut stats = data.get::<Stats>().unwrap().lock().await;
        if stats.should_track() {
            stats.post_servers(sum).await;
        }

        let presence_str = format!("{} servers | ;invite", sum);
        ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;
        info!("{} shard(s) ready", shards.len());
        debug!("Existing in {} guilds", sum);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new : bool) {
        if is_new {
            let data = ctx.data.read().await;

            // publish new server to stats
            {
                let mut stats = data.get::<Stats>().unwrap().lock().await;
                if stats.should_track() {
                    stats.new_server().await;
                }
            }

            // post new server to join log
            {
                let info = data.get::<BotInfo>().unwrap().read().await;
                if let Some(log) = info.get("JOIN_LOG") {
                    if let Ok(id) = log.parse::<u64>() {
                        let emb = DiscordHelpers::build_join_embed(&guild);
                        DiscordHelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
                    }
                }
            }

            // update shard guild count & presence
            let sum : usize = {
                let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
                let index = ctx.shard_id as usize;
                shard_info[index] += 1;
                shard_info.iter().sum()
            };

            let presence_str = format!("{} servers | ;invite", sum);
            ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;

            info!("Joining {}", guild.name);
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _member_data_if_available: Option<Member>) {
        let data = ctx.data.read().await;
        let info = data.get::<BotInfo>().expect("No bot info.").read().await;

        if user.id.to_string() == *info.get("BOT_ID").unwrap() {

            let mut stats = data.get::<Stats>().unwrap().lock().await;
            if stats.should_track() {
                stats.leave_server().await;
            }

            if let Some(log) = info.get("JOIN_LOG") {
                if let Ok(id) = log.parse::<u64>() {
                    let emb = DiscordHelpers::build_leave_embed(&guild_id);
                    DiscordHelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
                }
            }

            // update shard guild count & presence
            let sum : usize = {
                let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
                let index = ctx.shard_id as usize;
                shard_info[index] += 1;
                shard_info.iter().sum()
            };
            let presence_str = format!("{} servers | ;invite", sum);
            ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;
            info!("Leaving {}", guild_id);
        }
    }

    async fn ready(&self, ctx : Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);
        let data = ctx.data.read().await;
        let mut info = data.get::<BotInfo>().unwrap().write().await;
        info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());

        let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
        shard_info.push(ready.guilds.len());

        if shard_info.len() == ready.shard.unwrap()[1] as usize {
            self.all_shards_ready(&ctx, &data, &shard_info).await;
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

}
