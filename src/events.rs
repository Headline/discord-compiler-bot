use serenity::{
    async_trait,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

use crate::cache::*;

pub struct Handler; // event handler for serenity

#[async_trait]
trait ShardsReadyHandler {
    async fn all_shards_ready(&self, ctx : &Context, shards : &Vec<usize>);
}

#[async_trait]
impl ShardsReadyHandler for Handler {
    async fn all_shards_ready(&self, ctx : &Context, shards : &Vec<usize>) {
        let sum : usize = shards.iter().sum();
        let presence_str = format!("{} servers | ;invite", sum);
        ctx.set_presence(Some(Activity::listening(&presence_str)), OnlineStatus::Online).await;
        info!("{} shard(s) ready", shards.len());
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
                self.all_shards_ready(&ctx, &shard_info).await;
            }
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}
