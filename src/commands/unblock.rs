use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::BlocklistCache;

#[command]
#[owners_only]
pub async fn unblock(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() != 1 {
        return Err(CommandError::from("Supply an id to unblock"));
    }

    let arg = args.parse::<u64>()?;

    let data = ctx.data.read().await;
    let mut blocklist = data.get::<BlocklistCache>().unwrap().write().await;

    blocklist.unblock(arg);

    msg.channel_id.say(&ctx.http, format!("Unblocked snowflake `{}`", &arg)).await?;
    Ok(())
}
