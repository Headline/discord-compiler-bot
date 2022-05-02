use serenity::{
    client::Context,
    framework::standard::{CommandError, CommandResult},
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::{
    cache::CompilerCache, managers::compilation::CompilationManager,
    utls::discordhelpers::interactions,
};

pub async fn asm(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    interactions::handle_asm_or_compile_request(
        ctx,
        command,
        &CompilationManager::slash_cmd_langs_asm(),
        true,
        |parse_result| async move {
            let data = ctx.data.read().await;
            let compilation_manager = data.get::<CompilerCache>().unwrap();
            let compilation_manager_lock = compilation_manager.read().await;
            let compilation_res = compilation_manager_lock
                .assembly(&parse_result, &command.user)
                .await;
            let result = match compilation_res {
                Ok(r) => r,
                Err(e) => {
                    return Err(CommandError::from(format!("{}", e)));
                }
            };
            Ok(result.1)
        },
    )
    .await?;
    Ok(())
}
