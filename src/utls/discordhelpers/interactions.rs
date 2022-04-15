use std::{
    sync::Arc,
    future::Future,
    time::Duration
};

use futures_util::StreamExt;

use serenity::{
    builder::{CreateComponents, CreateEmbed, CreateInteractionResponse, CreateSelectMenuOption},
    client::Context,
    framework::standard::CommandError,
    model::interactions::{InteractionApplicationCommandCallbackDataFlags, InteractionResponseType},
    model::interactions::message_component::{ActionRowComponent, ButtonStyle, InputTextStyle},
    model::prelude::message_component::MessageComponentInteraction,
    model::prelude::modal::ModalSubmitInteraction,
    builder::EditInteractionResponse,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::{
    utls::discordhelpers::embeds,
    utls::constants::{C_ASM_COMPILERS, C_EXEC_COMPILERS, COLOR_OKAY, CPP_ASM_COMPILERS, CPP_EXEC_COMPILERS},
    cache::StatsManagerCache,
    managers::compilation::{RequestHandler},
    cache::CompilerCache,
    utls::constants::COLOR_WARN,
    utls::parser::{ParserResult},
    utls::discordhelpers::embeds::build_publish_embed,
    utls::{discordhelpers, parser}
};
use crate::cache::ConfigCache;

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

    println!("awaiting response...");
    let msg = interaction.get_interaction_response(&ctx.http).await?;
    println!("response got...");
    if let Some(resp) = msg.await_modal_interaction(&ctx.shard).await {
        println!("response: {:?}", resp.kind);
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

pub(crate) async fn handle_asm_or_compile_request<F, Fut>(ctx: &Context, command: &ApplicationCommandInteraction, languages: &[&str], is_asm: bool, get_result: F) -> Result<(), CommandError>
where
    F: FnOnce(ParserResult) -> Fut,
    Fut: Future<Output = Result<CreateEmbed, CommandError>>,
{
    let mut parse_result = ParserResult::default();

    let mut msg = None;
    for (_, value) in &command.data.resolved.messages {
        if !parser::find_code_block(& mut parse_result, &value.content, &command.user).await? {
            command.create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            }).await?;
            return Err(CommandError::from("Unable to find a codeblock to compile!"))
        }
        msg = Some(value);
        break;
    }

    // We never got a target from the codeblock, let's have them manually select a language
    let mut sent_interaction = false;
    if parse_result.target.is_empty() {
        command.create_interaction_response(&ctx.http, |response| {
            create_language_interaction(response, &languages)
        }).await?;

        let resp = command.get_interaction_response(&ctx.http).await?;
        let selection = match resp.await_component_interaction(ctx)
            .timeout(Duration::from_secs(30)).await {
            Some(s) => s,
            None => {
                return Ok(())
            }
        };

        sent_interaction = true;
        parse_result.target = selection.data.values.get(0).unwrap().to_lowercase();
        selection.defer(&ctx.http).await?;
    }

    let language = parse_result.target.clone();
    let options = create_compiler_options(ctx, &language, is_asm).await?;

    if !sent_interaction {
        command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    let compile_components = create_compile_panel(options);

                    data
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        .content("Select a compiler:")
                        .set_components(compile_components)
                })
        }).await?;
    }
    else {
        command.edit_original_interaction_response(&ctx.http, |response| {
            response
                .content("Select a compiler:")
                .components(|c| {
                    *c = create_compile_panel(options);
                    c
                })
        }).await?;
    }

    let resp = command.get_interaction_response(&ctx.http).await?;
    let mut cib = resp.await_component_interactions(&ctx.shard).timeout(Duration::from_secs(30)).await;

    // collect compiler into var
    parse_result.target = language.to_owned();

    let mut last_interaction = None;
    let mut more_options_response = None;
    while let Some(interaction) = &cib.next().await {
        last_interaction = Some(interaction.clone());
        match interaction.data.custom_id.as_str() {
            "compiler_select" => {
                parse_result.target = interaction.data.values[0].clone();
                interaction.defer(&ctx.http).await?;
            }
            "2" => {
                command.edit_original_interaction_response(&ctx.http, |resp| {
                    resp.components(|cmps| {
                        cmps.set_action_rows(Vec::new())
                    })
                        .set_embeds(Vec::new())
                        .embed(|emb| {
                            emb.color(COLOR_WARN)
                                .description("Awaiting completion of modal interaction, \
                                if you have cancelled the menu you may safely dismiss the message")
                        })
                }).await?;

                more_options_response = create_more_options_panel(ctx, interaction.clone(), & mut parse_result).await?;
                if more_options_response.is_some() {
                    cib.stop();
                    break;
                }
            }
            "1" => {
                cib.stop();
                break;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }

    // exit, they let this expire
    if last_interaction.is_none() && more_options_response.is_none() {
        return Ok(())
    }

    command.edit_original_interaction_response(&ctx.http, |resp| {
        create_think_interaction(resp)
    }).await.unwrap();

    let data = ctx.data.read().await;
    let config = data.get::<ConfigCache>().unwrap();
    let config_lock = config.read().await;
    let comp_log_id = config_lock.get("COMPILE_LOG");

    let result = get_result(parse_result.clone()).await?;
    let is_success = result.0["color"] == COLOR_OKAY;
    //statistics
    {
        let stats_manager = data.get::<StatsManagerCache>().unwrap().lock().await;
        if !is_asm && stats_manager.should_track() {
            stats_manager.compilation(&language, is_success).await;
        }
    }

    if let Some(log) = comp_log_id {
        if let Ok(id) = log.parse::<u64>() {
            let guild = if command.guild_id.is_some() {command.guild_id.unwrap().0.to_string()} else {"<<unknown>>".to_owned()};
            let emb = embeds::build_complog_embed(
                is_success,
                &parse_result.code,
                &parse_result.target,
                &command.user.tag(),
                command.user.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }

    command.edit_original_interaction_response(&ctx.http, |resp| {
        edit_to_confirmation_interaction(&result, resp)
    }).await.unwrap();

    let int_resp = command.get_interaction_response(&ctx.http).await?;
    if let Some(int) = int_resp.await_component_interaction(&ctx.shard).await {
        int.create_interaction_response(&ctx.http, |resp| {
            create_dismiss_response(resp)
        }).await?;

        // dispatch final response
        msg.unwrap().channel_id.send_message(&ctx.http, |new_msg| {
            new_msg
                .allowed_mentions(|mentions| {
                    mentions.replied_user(false)
                })
                .reference_message(msg.unwrap())
                .set_embed(result)
        }).await?;
    }
    Ok(())
}

pub async fn send_error_msg(ctx : &Context, command : &ApplicationCommandInteraction, edit : bool, fail_embed : CreateEmbed) -> serenity::Result<()> {
    if edit {
        command.edit_original_interaction_response(&ctx.http, |rsp| {
            rsp.content("")
                .set_embeds(Vec::new())
                .components(|cmps| {
                    cmps.set_action_rows(Vec::new())
                })
                .set_embed(fail_embed)
        }).await?;
        Ok(())
    }
    else {
        command.create_interaction_response(&ctx.http, |resp| {
            resp.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.content("")
                        .set_embeds(Vec::new())
                        .components(|cmps| {
                            cmps.set_action_rows(Vec::new())
                        })
                        .set_embed(fail_embed)
                })
        }).await
    }
}