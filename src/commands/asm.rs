use std::time::Duration;
use futures_util::StreamExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
};

<<<<<<< HEAD
use crate::cache::{ConfigCache, MessageCache, CompilerCache, MessageCacheEntry};
=======
use crate::cache::{ConfigCache, MessageCache, CompilerCache, StatsManagerCache};
>>>>>>> 6d3df4a (Begin Discord integrations API implementation)
use crate::utls::constants::*;
use crate::utls::{discordhelpers};
use crate::utls::discordhelpers::{embeds, interactions};

use serenity::builder::{CreateEmbed};
use serenity::model::channel::Message;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::{InteractionApplicationCommandCallbackDataFlags, InteractionResponseType};
use serenity::model::user::User;
use crate::managers::compilation::CompilationManager;

use crate::utls::{parser};
use crate::utls::parser::ParserResult;

pub async fn asm(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    interactions::handle_asm_or_compile_request(ctx, command, &CompilationManager::slash_cmd_langs_asm(), true, |parse_result| async move {
        let data = ctx.data.read().await;
        let compilation_manager= data.get::<CompilerCache>().unwrap();
        let compilation_manager_lock = compilation_manager.read().await;
        let compilation_res = compilation_manager_lock.assembly(&parse_result, &command.user).await;
        let result = match compilation_res {
            Ok(r) => r,
            Err(e) => {
                return Err(CommandError::from(format!("{}", e)));
            }
        };
        Ok(result)
    }).await?;
    Ok(())
}