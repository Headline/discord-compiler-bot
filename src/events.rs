use serenity::{
    async_trait,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

use crate::cache::*;
use serenity::model::guild::{Guild, PartialGuild};

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
    async fn ready(&self, ctx : Context, ready: Ready) {
        info!("[Shard {}] Ready", ctx.shard_id);
        let data = ctx.data.write().await;
        {
            let mut info = data.get::<BotInfo>().unwrap().write().await;
            info.insert("BOT_AVATAR", ready.user.avatar_url().unwrap());

            let mut shard_info = data.get::<ShardServers>().unwrap().lock().await;
            shard_info.push(ready.guilds.len());

            if shard_info.len() == ready.shard.unwrap()[1] as usize {
                self.all_shards_ready(&ctx, &data, &shard_info).await;
            }
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn guild_create(&self, ctx: Context, _guild: Guild, is_new : bool) {
        if is_new {
            let data = ctx.data.write().await;
            let mut stats = data.get::<Stats>().unwrap().lock().await;
            if stats.should_track() {
                stats.new_server().await;
            }
        }
    }

    async fn guild_delete(&self, ctx: Context, _incomplete: PartialGuild, _full: Option<Guild>) {
        let data = ctx.data.write().await;
        let mut stats = data.get::<Stats>().unwrap().lock().await;
        if stats.should_track() {
            stats.leave_server().await;
        }
    }

}