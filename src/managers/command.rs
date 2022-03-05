use std::collections::HashMap;
use serenity::builder::CreateApplicationCommandOption;
use serenity::client::Context;
use serenity::model::guild::Guild;
use serenity::model::interactions::application_command::{ApplicationCommandOptionType};
use crate::cache::CompilerCache;
use crate::managers::compilation::CompilationManager;

pub struct CommandManager {

}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {

        }
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
            let languages = CompilationManager::slash_cmd_langs();
            for language in languages {
                builder.create_application_command(|command| {
                    command.name(language).description(format!(""));

                    let stdin = CreateApplicationCommandOption::default()
                        .name(language)
                        .kind(ApplicationCommandOptionType::String)
                        .description("Input data stream")
                        .required(false);

                    command.add_option(stdin.to_owned())
                })
            }
            builder
        }).await {
            error!("Unable to create application commands: {}", err);
        }
    }
}