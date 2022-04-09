use std::collections::HashMap;
use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType, ApplicationCommandType}
};
use serenity::model::interactions::application_command::ApplicationCommand;
use crate::{cache::{CompilerCache}, slashcmds};
use crate::cache::StatsManagerCache;

pub struct CommandManager;

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {

        }
    }

    pub async fn on_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
        let command_name = command.data.name.to_lowercase();
        // push command executed to api
        {
            let data = ctx.data.read().await;
            let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            if stats.should_track() {
                stats.command_executed(&command_name, command.guild_id).await;
            }
        }

        match command_name.as_str() {
            "compile" => {
                slashcmds::compile::compile(ctx, command).await
            },
            "assembly" => {
                slashcmds::asm::asm(ctx, command).await
            },
            "ping" => {
                slashcmds::ping::ping(ctx, command).await
            },
            "help" => {
                slashcmds::help::help(ctx, command).await
            },
            "cpp" => {
                slashcmds::cpp::cpp(ctx, command).await
            },
            "invite" => {
                slashcmds::invite::invite(ctx, command).await
            }
            "format" => {
                slashcmds::format::format(ctx, command).await
            }
            e => {
                println!("OTHER: {}", e);
                Ok(())
            }
        }
    }

    pub async fn register_commands(&self, ctx: &Context) {
        let data_read = ctx.data.read().await;
        let compiler_cache = data_read.get::<CompilerCache>().unwrap();
        let compiler_manager = compiler_cache.read().await;

        let mut godbolt_dict = HashMap::new();
        for cache_entry in &compiler_manager.gbolt.cache {
            godbolt_dict.insert(cache_entry.language.name.clone(), cache_entry.language.id.clone());
        }
        if let Err(err) = ApplicationCommand::set_global_application_commands(&ctx.http, |builder| {
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::Message).name("Compile")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::Message).name("Assembly")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::Message).name("Format")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::ChatInput).name("help").description("Information on how to use the compiler")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::ChatInput).name("invite").description("Grab my invite link to invite me to your server")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::ChatInput).name("ping").description("Test my ping to Discord's endpoint")
            });
            builder.create_application_command(|cmd| {
                cmd.kind(ApplicationCommandType::ChatInput)
                    .name("cpp")
                    .description("Shorthand C++ compilation using geordi-like syntax")
                    .create_option(|opt| {
                        opt.required(false)
                            .name("input")
                            .kind(ApplicationCommandOptionType::String)
                            .description("Geordi-like input")
                    })
            });

            builder
        }).await {
            error!("Unable to create application commands: {}", err);
        }
    }
}