use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::time::Instant;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;

pub async fn ping(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {

    let old = Instant::now();
    msg.create_interaction_response(&ctx.http, |resp| {
        resp.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|data| {
                data.content("🏓 Pong!\n...")
            })
    }).await?;
    let new = Instant::now();

    msg.edit_original_interaction_response(&ctx.http, |resp| {
        resp.content(format!("🏓 Pong!\n{} ms", (new - old).as_millis()))
    }).await?;
    Ok(())
}
