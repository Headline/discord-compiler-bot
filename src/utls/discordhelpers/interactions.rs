use futures_util::StreamExt;

use serenity::{
    builder::EditInteractionResponse,
    builder::{CreateEmbed, CreateInteractionResponse, CreateSelectMenuOption},
    client::Context,
    framework::standard::CommandError,
};
use serenity::all::{ButtonStyle, CommandInteraction, CreateActionRow, CreateButton, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind, InteractionResponseFlags, MessageFlags, ModalInteraction};

use crate::{
    cache::CompilerCache,
    managers::compilation::RequestHandler,
    utls::constants::COLOR_WARN,
    utls::constants::{
        COLOR_OKAY, CPP_ASM_COMPILERS, CPP_EXEC_COMPILERS, C_ASM_COMPILERS, C_EXEC_COMPILERS,
    },
    utls::discordhelpers::embeds::build_publish_embed,
};

pub fn create_compile_panel(compiler_options: Vec<CreateSelectMenuOption>) -> Vec<CreateActionRow> {
    let mut components = Vec::new();

    components.push(CreateActionRow::SelectMenu(
        CreateSelectMenu::new("compiler_select", CreateSelectMenuKind::String {options: compiler_options})
    ));
    components.push(CreateActionRow::Buttons(
        vec![
            CreateButton::new("1").style(ButtonStyle::Primary).label("Compile"),
            CreateButton::new("2").style(ButtonStyle::Secondary).label("More options"),
        ]
    ));

    components
}

pub async fn create_compiler_options(
    ctx: &Context,
    language: &str,
    is_assembly: bool,
) -> Result<Vec<CreateSelectMenuOption>, CommandError> {
    let mut options = Vec::new();

    let data = ctx.data.read().await;
    let compilers = data.get::<CompilerCache>().unwrap().read().await;
    let target = compilers.resolve_target(language);
    match target {
        RequestHandler::None => {
            return Err(CommandError::from(format!(
                "Unsupported language: {}",
                language
            )))
        }
        RequestHandler::WandBox => {
            let wbox = compilers.wbox.as_ref().unwrap();
            let compilers = wbox.get_compilers(language).unwrap();
            let mut first = true;
            for compiler in if compilers.len() > 25 {
                &compilers[..24]
            } else {
                &compilers
            } {
                let option = CreateSelectMenuOption::new(&compiler.name, &compiler.name)
                    .description(&compiler.version)
                    .default_selection(first);

                if first {
                    first = !first;
                }

                options.push(option);
            }
        }
        RequestHandler::CompilerExplorer => {
            let mut default = None;
            let mut list = None;

            for cache in &compilers.gbolt.as_ref().unwrap().cache {
                if cache.language.name.to_lowercase() == language {
                    list = Some(
                        cache
                            .compilers
                            .iter()
                            .map(|c| [c.name.as_str(), c.id.as_str()])
                            .collect(),
                    );
                    default = Some(&cache.language.default_compiler)
                }
            }

            // override list for languages with plenty of compilers
            if language == "c++" {
                if is_assembly {
                    list = Some(CPP_ASM_COMPILERS.to_vec());
                } else {
                    list = Some(CPP_EXEC_COMPILERS.to_vec());
                }
            } else if language == "c" {
                if is_assembly {
                    list = Some(C_ASM_COMPILERS.to_vec());
                } else {
                    list = Some(C_EXEC_COMPILERS.to_vec());
                }
            }

            if list.is_none() {
                warn!("No suitable compilers found for: {}", &language);
                return Err(CommandError::from(format!(
                    "No suitable compilers found for: {}",
                    language
                )));
            }

            for compiler in list.unwrap() {
                let option = CreateSelectMenuOption::new(compiler[0], compiler[1])
                    .default_selection(default.is_some() && default.unwrap().as_str() == compiler[1]);
                options.push(option);
            }
        }
    }

    if options.len() > 25 {
        options.drain(25..);
    }

    Ok(options)
}

pub fn edit_to_dismiss_response() ->  EditInteractionResponse {
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
            .components(vec![CreateActionRow::Buttons(vec![button])])
    )
}

pub fn create_diff_response(
    output: &str,
) -> CreateInteractionResponse {
    let embed = CreateEmbed::new()
        .color(COLOR_OKAY)
        .title("Diff completed")
        .description(format!("```diff\n{}\n```", output));

    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
}

pub(crate) fn create_think_interaction(
    resp: &mut EditInteractionResponse,
) -> EditInteractionResponse {
    resp.content("")
        .components(Vec::new())
        .embed(CreateEmbed::new().color(COLOR_WARN).description("Processing request..."))
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
        command
            .edit_response(&ctx.http, edit_response)
            .await?;
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
