use crate::framework::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::env;

use crate::utls::discordhelpers::embeds;

pub async fn invite(ctx: &Context, msg: &Message, _args: &str) -> CommandResult {
    let invite = env::var("INVITE_LINK").expect("Expected invite link envvar");

    let emb = embeds::build_invite_embed(&invite);
    embeds::reply_embed(&ctx.http, msg, emb).await?;
    Ok(())
}
