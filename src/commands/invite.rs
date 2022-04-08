use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::env;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use crate::cache::ConfigCache;

use crate::utls::discordhelpers::embeds;

pub async fn invite(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {
    let invite_link = {
        let data = ctx.data.read().await;
        let config= data.get::<ConfigCache>().unwrap();
        let config_cache = config.read().await;
        config_cache.get("INVITE_LINK").unwrap().clone()
    };

    let emb = embeds::build_invite_embed(&invite_link);

    msg.create_interaction_response(&ctx.http, |resp| {
        resp.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|data| {
                data.add_embed(emb)
            })
    }).await?;

    Ok(())
}
