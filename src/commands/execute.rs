use serenity::all::{CreateActionRow, CreateMessage};
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::cache::{MessageCache, MessageCacheEntry};
use crate::commands::compile::{build_link_button, handle_request};
use crate::utls::discordhelpers;

#[command]
#[bucket = "nospam"]
pub async fn execute(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let result = handle_request(ctx, &msg.content, &msg.author, msg, true).await?;

    // Build message with optional godbolt link button
    let mut new_msg = CreateMessage::new().embed(result.embed);
    let buttons = build_link_button(ctx, &result.details).await;
    if !buttons.is_empty() {
        new_msg = new_msg.components(vec![CreateActionRow::Buttons(buttons)]);
    }

    let sent = msg.channel_id.send_message(&ctx.http, new_msg).await?;

    // React with success/fail indicator
    discordhelpers::send_completion_react(ctx, &sent, result.details.success).await?;

    // Cache for edit tracking
    let data = ctx.data.read().await;
    let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
    let mut entry = MessageCacheEntry::new(sent, msg.clone());
    entry.executed = true;
    message_cache.insert(msg.id.get(), entry);

    debug!("Command executed");
    Ok(())
}
