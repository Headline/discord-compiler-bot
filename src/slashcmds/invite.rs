use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::{framework::standard::CommandResult, prelude::*};

use crate::{cache::ConfigCache, utls::discordhelpers::embeds};

pub async fn invite(ctx: &Context, msg: &CommandInteraction) -> CommandResult {
    let invite_link = {
        let data = ctx.data.read().await;
        let config = data.get::<ConfigCache>().unwrap();
        let config_cache = config.read().await;
        config_cache.get("INVITE_LINK").unwrap().clone()
    };

    let emb = embeds::build_invite_embed(&invite_link);

    msg.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().add_embed(emb)),
    )
    .await?;

    Ok(())
}
