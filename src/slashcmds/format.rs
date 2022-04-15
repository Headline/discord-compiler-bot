use serenity::{
    framework::standard::{CommandResult, CommandError},
    model::prelude::*,
    prelude::*,
    builder::{CreateInteractionResponse, EditInteractionResponse},
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::message_component::ButtonStyle
};
use crate::{
    cache::{CompilerCache},
    utls::parser::{ParserResult},
    utls::constants::COLOR_WARN,
    utls::discordhelpers::interactions,
    utls::parser
};
use godbolt::{Format, Godbolt};
use std::time::Duration;
use futures_util::StreamExt;

pub async fn format(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    let mut msg = None;
    let mut parse_result = ParserResult::default();

    for (_, value) in &command.data.resolved.messages {
        if !parser::find_code_block(& mut parse_result, &value.content, &command.user).await? {
            return Err(CommandError::from("Unable to find a codeblock to format!"))
        }
        msg = Some(value);
        break;
    }

    let data = ctx.data.read().await;
    let comp_mgr = data.get::<CompilerCache>().unwrap().read().await;
    let gbolt = &comp_mgr.gbolt;

    command.create_interaction_response(&ctx.http, |response| {
        create_formats_interaction(response, &gbolt.formats)
    }).await?;

    // Handle response from select menu / button interactions
    let resp = command.get_interaction_response(&ctx.http).await?;
    let mut cib = resp.await_component_interactions(&ctx.shard).timeout(Duration::from_secs(30)).await;
    let mut formatter = String::from("clangformat");
    let mut selected = false;
    while let Some(interaction) = &cib.next().await {
        match interaction.data.custom_id.as_str() {
            "formatter" => {
                formatter = interaction.data.values[0].clone();
                interaction.defer(&ctx.http).await?;
            }
            "select" => {
                interaction.defer(&ctx.http).await?;
                selected = true;
                cib.stop();
                break;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }

    // interaction expired...
    if !selected {
        return Ok(())
    }

    let styles = &gbolt.formats.iter().find(|p| p.format_type == formatter).unwrap().styles;
    command.edit_original_interaction_response(&ctx.http, |resp| {
        create_styles_interaction(resp, styles)
    }).await?;

    let resp = command.get_interaction_response(&ctx.http).await?;
    cib = resp.await_component_interactions(&ctx.shard).timeout(Duration::from_secs(30)).await;

    selected = false;
    let mut style = String::from("WebKit");
    while let Some(interaction) = &cib.next().await {
        match interaction.data.custom_id.as_str() {
            "style" => {
                style = interaction.data.values[0].clone();
                interaction.defer(&ctx.http).await?;
            }
            "select" => {
                selected = true;
                cib.stop();
                break;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }

    // they let this expire
    if !selected {
        return Ok(())
    }

    command.edit_original_interaction_response(&ctx.http, |resp| {
        interactions::create_think_interaction(resp)
    }).await.unwrap();

    let result = match Godbolt::format_code(&formatter, &style, &parse_result.code, true, 4).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)))
        }
    };


    command.edit_original_interaction_response(&ctx.http, |resp| {
        resp
            .set_embeds(Vec::new())
            .embed(|emb| {
                emb.color(COLOR_WARN)
                    .description("Interaction completed, you may safely dismiss this message.")
            })
            .components(|components| {
                components.set_action_rows(Vec::new())
            })
    }).await.unwrap();

    // dispatch final response
    msg.unwrap().channel_id.send_message(&ctx.http, |new_msg| {
        new_msg
            .allowed_mentions(|mentions| {
                mentions.replied_user(false)
            })
            .reference_message(msg.unwrap())
            .content(format!("```{}\n{}\n```Requested by: {}", if parse_result.target.is_empty() {""} else {&parse_result.target}, result.answer, command.user.tag()))
    }).await?;

    Ok(())
}

fn create_styles_interaction<'a>(response: &'a mut EditInteractionResponse, styles: &Vec<String>) -> &'a mut EditInteractionResponse {
    response
        .content("Select a style:")
        .components(|cmps| {
            cmps.create_action_row(|row| {
                row.create_select_menu(|menu| {
                    menu.custom_id("style")
                        .options(|opts| {
                            for style in styles {
                                opts.create_option(|opt| {
                                    opt.label(style)
                                        .value(style);
                                    if style == "WebKit" {
                                        opt.default_selection(true);
                                    }
                                    opt
                                });
                            }
                            opts
                        })
                })
            })
                .create_action_row(|row| {
                    row.create_button(|btn| {
                        btn.custom_id("select")
                            .label("Select")
                            .style(ButtonStyle::Primary)
                    })
                })
        })
}

fn create_formats_interaction<'a>(response: &'a mut CreateInteractionResponse, formats: &Vec<Format>) -> &'a mut CreateInteractionResponse {
    response
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|data| {
            data.content("Select a formatter to use:")
            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            .components(|cmps| {
                cmps.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("formatter")
                            .options(|opts| {
                            for format in formats {
                                opts.create_option(|opt| {
                                    opt.label(&format.name)
                                        .value(&format.format_type)
                                        .description(&format.exe);
                                    if format.format_type == "clangformat" {
                                        opt.default_selection(true);
                                    }
                                    opt
                                });
                            }
                            opts
                        })
                    })
                })
                .create_action_row(|row| {
                    row.create_button(|btn| {
                        btn.custom_id("select")
                            .label("Select")
                            .style(ButtonStyle::Primary)
                    })
                })
            })
    })
}