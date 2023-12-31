use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Instant;
use serenity::all::EditMessage;

#[command]
pub async fn ping(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let old = Instant::now();
    let mut m = msg.channel_id.say(&ctx.http, "🏓 Pong!\n...").await?;
    let new = Instant::now();

    let edit = EditMessage::new()
        .content(format!("🏓 Pong!\n{} ms", (new - old).as_millis()));
    m.edit(ctx, edit).await?;

    debug!("Command executed");
    Ok(())
}
