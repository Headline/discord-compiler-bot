use crate::framework::{CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::BlocklistCache;

pub async fn unblock(ctx: &Context, msg: &Message, args: &str) -> CommandResult {
    let mut parts = args.split_whitespace();
    let arg = match (parts.next(), parts.next()) {
        (Some(id), None) => id.parse::<u64>()?,
        _ => return Err(CommandError::from("Supply an id to unblock")),
    };

    {
        let data = ctx.data.read().await;
        let mut blocklist = data.get::<BlocklistCache>().unwrap().write().await;
        blocklist.unblock(arg);
    }

    msg.channel_id
        .say(&ctx.http, format!("Unblocked snowflake `{}`", arg))
        .await?;
    Ok(())
}
