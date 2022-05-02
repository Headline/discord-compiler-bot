use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Instant;

#[command]
pub async fn ping(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let old = Instant::now();
    let mut m = msg.channel_id.say(&ctx.http, "🏓 Pong!\n...").await?;
    let new = Instant::now();

    m.edit(ctx, |m| {
        m.content(format!("🏓 Pong!\n{} ms", (new - old).as_millis()))
    })
    .await?;
    debug!("Command executed");
    Ok(())
}
