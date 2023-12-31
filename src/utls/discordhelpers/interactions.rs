use serenity::all::{
    ButtonStyle, CommandInteraction, CreateActionRow, CreateButton,
    CreateInteractionResponseMessage, InteractionResponseFlags,
};
use serenity::{
    builder::EditInteractionResponse,
    builder::{CreateEmbed, CreateInteractionResponse},
    client::Context,
};

use crate::{utls::constants::COLOR_OKAY, utls::constants::COLOR_WARN};

pub fn edit_to_dismiss_response() -> EditInteractionResponse {
    let embed = CreateEmbed::new()
        .color(COLOR_OKAY)
        .description("Interaction completed, you may safely dismiss this message.");

    EditInteractionResponse::new()
        .embed(embed)
        .components(Vec::new())
}

pub fn create_diff_select_response() -> CreateInteractionResponse {
    let notice_embed = CreateEmbed::new()
        .color(COLOR_WARN)
        .description("Please re-run this command on another message to generate a diff");

    let button = CreateButton::new("cancel")
        .label("Cancel")
        .style(ButtonStyle::Danger);

    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .flags(InteractionResponseFlags::EPHEMERAL)
            .embed(notice_embed)
            .components(vec![CreateActionRow::Buttons(vec![button])]),
    )
}

pub fn create_diff_response(output: &str) -> CreateInteractionResponse {
    let embed = CreateEmbed::new()
        .color(COLOR_OKAY)
        .title("Diff completed")
        .description(format!("```diff\n{}\n```", output));

    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
}

pub fn create_think_interaction() -> EditInteractionResponse {
    EditInteractionResponse::new()
        .content("")
        .components(Vec::new())
        .embed(
            CreateEmbed::new()
                .color(COLOR_WARN)
                .description("Processing request..."),
        )
}

pub async fn send_error_msg(
    ctx: &Context,
    command: &CommandInteraction,
    edit: bool,
    fail_embed: CreateEmbed,
) -> serenity::Result<()> {
    if edit {
        let edit_response = EditInteractionResponse::new()
            .embeds(Vec::new())
            .components(Vec::new())
            .embeds(vec![fail_embed]);
        command.edit_response(&ctx.http, edit_response).await?;
        Ok(())
    } else {
        let new_response = CreateInteractionResponseMessage::new()
            .embeds(Vec::new())
            .components(Vec::new())
            .embeds(vec![fail_embed]);

        command
            .create_response(&ctx.http, CreateInteractionResponse::Message(new_response))
            .await
    }
}
