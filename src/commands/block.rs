use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::BlocklistCache;

#[command]
#[owners_only]
pub async fn block(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() != 1 {
        return Err(CommandError::from("Supply an id to block"));
    }

    let arg = args.parse::<u64>()?;

    let data = ctx.data.read().await;
    let mut blocklist = data.get::<BlocklistCache>().unwrap().write().await;

    blocklist.block(arg);

    msg.channel_id.say(&ctx.http, format!("Blocked snowflake `{}`", &arg)).await?;
    Ok(())
}
