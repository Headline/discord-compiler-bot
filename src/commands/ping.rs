use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Instant;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {

    let old = Instant::now();
    let mut m = msg.channel_id.say(&ctx.http, "🏓 Pong!\n...").await?;
    let new = Instant::now();

    m.edit(ctx, |m| m.content(format!("🏓 Pong!\n{} ms", (new - old).as_millis()))).await?;
    debug!("Command executed");
    Ok(())
}
