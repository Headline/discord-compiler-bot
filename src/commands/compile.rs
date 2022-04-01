use serenity::{
    framework::standard::{CommandResult},
    framework::standard::CommandError,
    client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::InteractionResponseType,
    model::prelude::InteractionApplicationCommandCallbackDataFlags
};

use std::time::Duration;
use futures_util::StreamExt;
use tokio::sync::RwLockReadGuard;

use crate::{
    managers::compilation::{CompilationManager},
    cache::{CompilerCache, StatsManagerCache},
    utls::discordhelpers::{interactions},
    utls::{parser},
    utls::constants::COLOR_OKAY,
    utls::parser::{ParserResult}
};

pub async fn compile(ctx: &Context, command : &ApplicationCommandInteraction) -> CommandResult {
    let mut parse_result = ParserResult::default();

    let mut msg = None;
    for (_, value) in &command.data.resolved.messages {
        if !parser::find_code_block(& mut parse_result, &value.content) {
            return Err(CommandError::from("Unable to find a codeblock to compile!"))
        }
        msg = Some(value);
        break;
    }

    // We never got a target from the codeblock, let's have them manually select a language
    let mut sent_interaction = false;
    if parse_result.target.is_empty() {
        let languages = CompilationManager::slash_cmd_langs();
        command.create_interaction_response(&ctx.http, |response| {
            interactions::create_language_interaction(response, &languages)
        }).await?;

        let resp = command.get_interaction_response(&ctx.http).await?;
        let selection = match resp.await_component_interaction(ctx)
                                                            .timeout(Duration::from_secs(30)).await {
            Some(s) => s,
            None => {
                return Err(CommandError::from("Request timed out"))
            }
        };

        sent_interaction = true;
        parse_result.target = selection.data.values.get(0).unwrap().to_lowercase();
    }

    // ensure our options list is less than discord's limit
    let language = parse_result.target.clone();
    let mut options = interactions::create_compiler_options(ctx, &language, false).await?;

    if !sent_interaction {
        command.create_interaction_response(&ctx.http, |response| {
            response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|data| {
                let compile_components = interactions::create_compile_panel(options);

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
                    *c = interactions::create_compile_panel(options);
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
                more_options_response = interactions::create_more_options_panel(ctx, interaction.clone(), & mut parse_result).await?;
                cib.stop();
                break;
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
        interactions::create_think_interaction(resp)
    }).await.unwrap();

    let data = ctx.data.read().await;
    let result = {
        let compilation_manager= data.get::<CompilerCache>().unwrap();
        let compilation_manager_lock : RwLockReadGuard<CompilationManager> = compilation_manager.read().await;
        let compilation_res = compilation_manager_lock.compile(&parse_result, &command.user).await;
        let result = match compilation_res {
            Ok(r) => r,
            Err(e) => {
                return Err(CommandError::from(format!("{}", e)));
            }
        };
        result
    };

    //statistics
    {
        let stats_manager = data.get::<StatsManagerCache>().unwrap().lock().await;
        if stats_manager.should_track() {
            stats_manager.compilation(&language, result.0["color"] == COLOR_OKAY).await;
        }
    }

    command.edit_original_interaction_response(&ctx.http, |resp| {
        interactions::edit_to_confirmation_interaction(&result, resp)
    }).await.unwrap();

    let int_resp = command.get_interaction_response(&ctx.http).await?;
    if let Some(int) = int_resp.await_component_interaction(&ctx.shard).await {
        int.create_interaction_response(&ctx.http, |resp| {
            interactions::create_dismiss_response(resp)
        }).await?;

        // dispatch final response
        msg.unwrap().channel_id.send_message(&ctx.http, |new_msg| {
            new_msg
                .reference_message(msg.unwrap())
                .set_embed(result)
        }).await?;
    }
    Ok(())
}