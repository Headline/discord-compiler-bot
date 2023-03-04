use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::env;

use crate::utls::discordhelpers::embeds;

#[command]
pub async fn invite(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let invite = env::var("INVITE_LINK").expect("Expected invite link envvar");

    let emb = embeds::build_invite_embed(&invite);

    let emb_msg = embeds::embed_message(emb);
    msg.channel_id
        .send_message(&ctx.http, |e| {
            *e = emb_msg;
            e
        })
        .await?;

    Ok(())
}
