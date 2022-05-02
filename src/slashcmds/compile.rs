use serenity::{
    client::Context, framework::standard::CommandError, framework::standard::CommandResult,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use tokio::sync::RwLockReadGuard;

use crate::{
    cache::CompilerCache, managers::compilation::CompilationManager,
    utls::discordhelpers::interactions,
};

pub async fn compile(ctx: &Context, command: &ApplicationCommandInteraction) -> CommandResult {
    interactions::handle_asm_or_compile_request(
        ctx,
        command,
        &CompilationManager::slash_cmd_langs(),
        false,
        |parse_result| async move {
            let data = ctx.data.read().await;
            let compilation_manager = data.get::<CompilerCache>().unwrap();
            let compilation_manager_lock: RwLockReadGuard<CompilationManager> =
                compilation_manager.read().await;
            let compilation_res = compilation_manager_lock
                .compile(&parse_result, &command.user)
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
