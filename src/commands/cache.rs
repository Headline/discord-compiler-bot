use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::StatsManagerCache;

#[command]
pub async fn cache(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut stats_manager = data.get::<StatsManagerCache>().unwrap().lock().await;

    if stats_manager.should_track() {
        if let Some(id) = msg.guild_id {
            stats_manager.clear_user(id.0);
            debug!("Cleared cache for: {}", id.0);
            msg.channel_id.say(&ctx.http, "Your cache has been cleared!").await?;
            return Ok(());
        }
    }

    msg.channel_id.say(&ctx.http, "Unable to find id in cache, this is probably an error.").await?;
    Ok(())
}
