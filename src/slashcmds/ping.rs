use serenity::{framework::standard::CommandResult, prelude::*};

use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use std::time::Instant;

pub async fn ping(ctx: &Context, msg: &CommandInteraction) -> CommandResult {
    let old = Instant::now();
    let ping_response = CreateInteractionResponseMessage::new().content("🏓 Pong!\n...");
    msg.create_response(&ctx.http, CreateInteractionResponse::Message(ping_response))
        .await?;

    let new = Instant::now();
    let edit_ping_response =
        EditInteractionResponse::new().content(format!("🏓 Pong!\n{} ms", (new - old).as_millis()));
    msg.edit_response(&ctx.http, edit_ping_response).await?;
    Ok(())
}
