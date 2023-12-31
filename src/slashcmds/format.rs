use crate::{
    cache::CompilerCache, utls::constants::COLOR_WARN, utls::discordhelpers::interactions,
    utls::parser, utls::parser::ParserResult,
};
use futures_util::StreamExt;
use godbolt::{Format, Godbolt};
use serenity::{
    builder::{CreateInteractionResponse, EditInteractionResponse},
    framework::standard::{CommandError, CommandResult},
    prelude::*,
};

use serenity::all::{
    ButtonStyle, CommandInteraction, ComponentInteractionDataKind, CreateActionRow,
    CreateAllowedMentions, CreateButton, CreateEmbed, CreateInteractionResponseMessage,
    CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    InteractionResponseFlags,
};
use std::time::Duration;

pub async fn format(ctx: &Context, command: &CommandInteraction) -> CommandResult {
    let mut msg = None;
    let mut parse_result = ParserResult::default();

    if let Some((_, value)) = command.data.resolved.messages.iter().next() {
        if !parser::find_code_block(&mut parse_result, &value.content, &command.user).await? {
            return Err(CommandError::from("Unable to find a codeblock to format!"));
        }
        msg = Some(value);
    }

    let data = ctx.data.read().await;
    let comp_mgr = data.get::<CompilerCache>().unwrap().read().await;
    if comp_mgr.gbolt.is_none() {
        return Err(CommandError::from(
            "Compiler Explorer service is currently down, please try again later.",
        ));
    }

    command
        .create_response(
            &ctx.http,
            create_formats_interaction(&comp_mgr.gbolt.as_ref().unwrap().formats),
        )
        .await?;
    // Handle response from select menu / button interactions
    let resp = command.get_response(&ctx.http).await?;
    let mut cic = resp
        .await_component_interactions(&ctx.shard)
        .timeout(Duration::from_secs(30))
        .stream();

    let mut formatter = String::from("clangformat");
    let mut selected = false;
    while let Some(interaction) = &cic.next().await {
        match interaction.data.custom_id.as_str() {
            "formatter" => {
                formatter = match &interaction.data.kind {
                    ComponentInteractionDataKind::StringSelect { values } => values[0].clone(),
                    _ => panic!("unexpected interaction data kind"),
                };

                interaction.defer(&ctx.http).await?;
            }
            "select" => {
                interaction.defer(&ctx.http).await?;
                selected = true;
                drop(cic);
                break;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }

    // interaction expired...
    if !selected {
        return Ok(());
    }

    let styles = &comp_mgr
        .gbolt
        .as_ref()
        .unwrap()
        .formats
        .iter()
        .find(|p| p.format_type == formatter)
        .unwrap()
        .styles;

    // WebKit is our default value for clang-fmt
    let mut style = String::from("WebKit");
    if !styles.is_empty() {
        command
            .edit_response(&ctx.http, create_styles_interaction(styles))
            .await?;

        let resp = command.get_response(&ctx.http).await?;
        cic = resp
            .await_component_interactions(&ctx.shard)
            .timeout(Duration::from_secs(30))
            .stream();
        selected = false;
        while let Some(interaction) = &cic.next().await {
            match interaction.data.custom_id.as_str() {
                "style" => {
                    style = match &interaction.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => values[0].clone(),
                        _ => panic!("unexpected interaction data kind"),
                    };
                    interaction.defer(&ctx.http).await?;
                }
                "select" => {
                    selected = true;
                    drop(cic);
                    break;
                }
                _ => {
                    unreachable!("Cannot get here..");
                }
            }
        }
    }

    // they let this expire
    if !selected {
        return Ok(());
    }

    command
        .edit_response(&ctx.http, interactions::create_think_interaction())
        .await
        .unwrap();

    let result = match Godbolt::format_code(&formatter, &style, &parse_result.code, true, 4).await {
        Ok(r) => r,
        Err(e) => return Err(CommandError::from(format!("{}", e))),
    };

    let complete_embed = CreateEmbed::new()
        .color(COLOR_WARN)
        .description("Interaction completed, you may safely dismiss this message");

    let edit = EditInteractionResponse::new().embed(complete_embed);
    command.edit_response(&ctx.http, edit).await.unwrap();

    // dispatch final response
    let mentions = CreateAllowedMentions::new().replied_user(false);
    let new_msg = CreateMessage::new()
        .allowed_mentions(mentions)
        .reference_message(msg.unwrap())
        .content(format!(
            "```{}\n{}\n```Requested by: {}",
            if parse_result.target.is_empty() {
                ""
            } else {
                &parse_result.target
            },
            result.answer,
            command.user.name
        ));
    msg.unwrap()
        .channel_id
        .send_message(&ctx.http, new_msg)
        .await?;
    Ok(())
}

fn create_styles_interaction(styles: &Vec<String>) -> EditInteractionResponse {
    let mut opts = Vec::new();
    for style in styles {
        opts.push(CreateSelectMenuOption::new(style, style).default_selection(style == "WebKit"));
    }

    let menu = CreateSelectMenu::new("style", CreateSelectMenuKind::String { options: opts });
    let submit_button = CreateButton::new("select")
        .label("Select")
        .style(ButtonStyle::Primary);

    EditInteractionResponse::new()
        .content("Select a style:")
        .components(vec![
            CreateActionRow::SelectMenu(menu),
            CreateActionRow::Buttons(vec![submit_button]),
        ])
}

fn create_formats_interaction(formats: &Vec<Format>) -> CreateInteractionResponse {
    let mut discord_menu_options = Vec::new();

    for format in formats {
        discord_menu_options.push(
            CreateSelectMenuOption::new(&format.name, &format.format_type)
                .description(&format.exe)
                .default_selection(format.format_type == "clangformat"),
        )
    }

    let discord_menu = CreateSelectMenu::new(
        "formatter",
        CreateSelectMenuKind::String {
            options: discord_menu_options,
        },
    );

    let select_button = CreateButton::new("select")
        .label("Select")
        .style(ButtonStyle::Primary);

    let response = CreateInteractionResponseMessage::new()
        .flags(InteractionResponseFlags::EPHEMERAL)
        .components(vec![
            CreateActionRow::SelectMenu(discord_menu),
            CreateActionRow::Buttons(vec![select_button]),
        ]);

    CreateInteractionResponse::Message(response)
}
