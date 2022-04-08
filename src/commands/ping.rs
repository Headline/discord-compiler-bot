use serenity::{
    framework::standard::{CommandResult},
    model::prelude::*,
    prelude::*,
    model::interactions::application_command::ApplicationCommandInteraction
};

use std::time::Instant;

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
