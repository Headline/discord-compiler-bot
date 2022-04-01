use std::sync::Arc;
use serenity::{
    builder::{CreateComponents, CreateEmbed, CreateInteractionResponse, CreateSelectMenuOption},
    client::Context,
    framework::standard::CommandError,
    model::interactions::{InteractionApplicationCommandCallbackDataFlags, InteractionResponseType},
    model::interactions::message_component::{ActionRowComponent, ButtonStyle, InputTextStyle},
    model::prelude::message_component::MessageComponentInteraction,
    model::prelude::modal::ModalSubmitInteraction
};
use serenity::builder::EditInteractionResponse;
use crate::{
    managers::compilation::{CompilationManager, RequestHandler},
    cache::CompilerCache,
    utls::constants::COLOR_WARN,
    utls::parser::{ParserResult}
};
use crate::utls::constants::{C_ASM_COMPILERS, C_EXEC_COMPILERS, CPP_ASM_COMPILERS, CPP_EXEC_COMPILERS};
use crate::utls::discordhelpers::embeds::build_publish_embed;

pub fn create_compile_panel(compiler_options : Vec<CreateSelectMenuOption>) -> CreateComponents {
    let mut components = CreateComponents::default();
    components.create_action_row(|row| {
        row.create_select_menu(|menu| {
            menu.custom_id("compiler_select").options(|opts| {
                opts.set_options(compiler_options)
            })
        })
    })
    .create_action_row(|row3| {
        row3.create_button(|button| {
            button.label("Compile").style(ButtonStyle::Primary).custom_id(1)
        })
        .create_button(|button| {
            button.label("More options").style(ButtonStyle::Secondary).custom_id(2)
        })
    });
    components
}

pub async fn create_more_options_panel(ctx: &Context, interaction : Arc<MessageComponentInteraction>, parse_result: &mut ParserResult) -> Result<Option<Arc<ModalSubmitInteraction>>, CommandError> {
    interaction.create_interaction_response(&ctx.http, |resp| {
        resp.kind(InteractionResponseType::Modal)
            .interaction_response_data(|data| {
                data
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    .custom_id("more_options_panel")
                    .content("Select a compiler:")
                    .title("More options")
                    .components(|components| {
                        components.create_action_row(|row2| {
                            row2.create_input_text(|txt| {
                                txt
                                    .custom_id("compiler_options")
                                    .label("Compiler options")
                                    .style(InputTextStyle::Short)
                                    .placeholder("-Wall -O3 etc.")
                                    .required(false)
                            })
                        }).create_action_row(|row2| {
                            row2.create_input_text(|txt| {
                                    txt
                                        .custom_id("cmdlineargs")
                                        .label("Command line arguments")
                                        .style(InputTextStyle::Short)
                                        .placeholder("")
                                        .required(false)
                            })
                        }).create_action_row(|row2| {
                            row2.create_input_text(|txt| {
                                txt
                                    .custom_id("stdin")
                                    .label("Standard input")
                                    .style(InputTextStyle::Paragraph)
                                    .placeholder("stdin")
                                    .required(false)
                            })
                        })
                    })
            })
    }).await.unwrap();

    let msg = interaction.get_interaction_response(&ctx.http).await?;
    if let Some(resp) = msg.await_modal_interaction(&ctx.shard).await {
        if let ActionRowComponent::InputText(input) = &resp.data.components[0].components[0] {
            parse_result.options = input.value.clone().split(" ").map(|p| p.to_owned()).collect();
        }
        if let ActionRowComponent::InputText(input) = &resp.data.components[1].components[0] {
            parse_result.args = input.value.split(" ").map(|p| p.to_owned()).collect();
        }
        if let ActionRowComponent::InputText(input) = &resp.data.components[2].components[0] {
            parse_result.stdin = input.value.clone();
        }
        resp.defer(&ctx.http).await?;
        return Ok(Some(resp.clone()))
    }
    else {
        Ok(None)
    }
}

pub async fn create_compiler_options(ctx: &Context, language: &str, is_assembly : bool) -> Result<Vec<CreateSelectMenuOption>, CommandError> {
    let mut options = Vec::new();

    let data = ctx.data.read().await;
    let compilers = data.get::<CompilerCache>().unwrap().read().await;
    let target = compilers.resolve_target(language);
    match target {
        RequestHandler::None => {
            return Err(CommandError::from(format!("Unsupported language: {}", language)))
        }
        RequestHandler::WandBox => {
            let compilers = compilers.wbox.get_compilers(language).unwrap();
            let mut first = true;
            for compiler in if compilers.len() > 25 {&compilers[..24]} else {&compilers}{
                let mut option = CreateSelectMenuOption::default();
                if first {
                    option.default_selection(true);
                    first = !first;
                }

                option.label(&compiler.name);
                option.value(&compiler.name);
                option.description(&compiler.version);

                options.push(option);
            }
        }
        RequestHandler::CompilerExplorer => {
            let mut default = None;
            let mut list = None;

            for cache in &compilers.gbolt.cache {
                if cache.language.name.to_lowercase() == language {
                    list = Some(cache.compilers.iter().map(|c| [c.name.as_str(), c.id.as_str()]).collect());
                    default = Some(&cache.language.default_compiler)
                }
            }

            // override list for languages with plenty of compilers
            if language == "c++" {
                if is_assembly {
                    list = Some(CPP_ASM_COMPILERS.to_vec());
                }
                else {
                    list = Some(CPP_EXEC_COMPILERS.to_vec());
                }
            }
            else if language == "c" {
                if is_assembly {
                    list = Some(C_ASM_COMPILERS.to_vec());
                }
                else {
                    list = Some(C_EXEC_COMPILERS.to_vec());
                }
            }

            for compiler in list.unwrap() {
                let mut option = CreateSelectMenuOption::default();
                option.label(compiler[0]);
                option.value(compiler[1]);
                if let Some(def) = default {
                    if def.as_str() == compiler[1] {
                        option.default_selection(true);
                    }
                }
                options.push(option);
            }
        }
    }

    if options.len() > 25 {
        options.drain(25..);
    }

    Ok(options)
}

pub fn edit_to_confirmation_interaction<'a>(result: &CreateEmbed, resp: &'a mut EditInteractionResponse) -> &'a mut EditInteractionResponse {
    resp
        .set_embeds(Vec::from([build_publish_embed(), result.clone()]))
        .components(|components| {
            components.set_action_rows(Vec::new())
                .create_action_row(|row| {
                    row.create_button(|btn| {
                        btn
                            .custom_id("publish")
                            .label("Publish")
                            .style(ButtonStyle::Primary)
                    })
                })
        })
}

pub fn create_confirmation_interaction<'a>(result: & CreateEmbed, resp: &'a mut CreateInteractionResponse) -> &'a mut CreateInteractionResponse {
    resp
        .kind(InteractionResponseType::UpdateMessage)
        .interaction_response_data(|data| {
            data
                .content("")
                .embed(|emb| {
                    emb
                        .color(COLOR_WARN)
                        .description("This result will not be visible to others until you click the publish button.\n\n \
                    If you are unhappy with your results please start a new compilation request \
                    and dismiss this message.")
                })
                .add_embed(result.clone())
                .components(|components| {
                    components.set_action_rows(Vec::new())
                        .create_action_row(|row| {
                            row.create_button(|btn| {
                                btn
                                    .custom_id("publish")
                                    .label("Publish")
                                    .style(ButtonStyle::Primary)
                            })
                        })
                })
        })
}

pub fn create_language_interaction<'a>(resp : &'a mut CreateInteractionResponse, languages : &[&str]) -> &'a mut CreateInteractionResponse {
    resp
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|data| {
            data
                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                .content("Select a language:")
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_select_menu(|menu| {
                            menu.custom_id("language_select").options(|opts| {

                                for language in languages {
                                    let mut option = CreateSelectMenuOption::default();
                                    option.value(language);
                                    option.label(language);
                                    opts.add_option(option);
                                }

                                opts
                            })
                        })
                    })
                })
        })
}

pub fn create_dismiss_response(resp: &mut CreateInteractionResponse) -> &mut CreateInteractionResponse {
    resp
        .kind(InteractionResponseType::UpdateMessage)
        .interaction_response_data(|data| {
            data
                .set_embeds(Vec::new())
                .embed(|emb| {
                    emb.color(COLOR_WARN)
                        .description("Interaction completed, you may safely dismiss this message.")
                })
                .components(|components| {
                    components.set_action_rows(Vec::new())
                })
        })
}

pub(crate) fn create_think_interaction(resp: &mut EditInteractionResponse) -> &mut EditInteractionResponse {
    resp
        .content("")
        .components(|cmps| {
            cmps.set_action_rows(Vec::new())
        })
        .embed(|emb| {
            emb.color(COLOR_WARN)
                .description("Processing request...")
        })
}