use std::collections::HashMap;
use serenity::builder::{CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption, CreateSelectMenuOptions};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::guild::Guild;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandType};
use serenity::model::interactions::InteractionResponseType;
use crate::cache::{CompilerCache};
use crate::commands;
use crate::managers::compilation::CompilationManager;

pub struct CommandManager {

}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {

        }
    }

    pub async fn on_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
        match command.data.name.to_lowercase().as_str() {
            //"ping" => commands::ping::ping(ctx, command),
            "compile" => {
                commands::compile::compile(ctx, command).await
            },
            "assembly" => {
                commands::asm::asm(ctx, command).await
            },
            e => {
                println!("OTHER: {}", e);
                Ok(())
            }
        }
/*
        if let Err(e) = result {
            let emb = embeds::build_fail_embed(&command.user, &format!("{}", e));
            CommandManager::dispatch_response(ctx, command, emb)
        }*/

        /*if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }*/
    }

    pub async fn register_commands(&self, ctx: &Context, x: &Guild) {
        let data_read = ctx.data.read().await;
        let compiler_cache = data_read.get::<CompilerCache>().unwrap();
        let compiler_manager = compiler_cache.read().await;

        let mut godbolt_dict = HashMap::new();
        for cache_entry in &compiler_manager.gbolt.cache {
            godbolt_dict.insert(cache_entry.language.name.clone(), cache_entry.language.id.clone());
        }
        if let Err(err) = x.set_application_commands(&ctx.http, |builder| {
            /*builder.create_application_command(|command| {
                command.name("ping")
                    .description("Outputs the bot's ping to Discord")
            });*/

            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::Message).name("Compile")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::Message).name("Assembly")
            });

            builder
        }).await {
            error!("Unable to create application commands: {}", err);
        }
    }
}