use crate::cache::StatsManagerCache;
use crate::slashcmds;

use serenity::{
    builder::CreateApplicationCommand, client::Context, framework::standard::CommandResult,
    model::application::command::Command, model::application::command::CommandOptionType,
    model::application::command::CommandType,
    model::application::interaction::application_command::ApplicationCommandInteraction,
    model::guild::Guild,
};

pub struct CommandManager {
    commands_registered: bool,
    commands: Vec<CreateApplicationCommand>,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            commands_registered: false,
            commands: CommandManager::build_commands(),
        }
    }

    pub async fn on_command(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> CommandResult {
        let command_name = command.data.name.to_lowercase();
        // push command executed to api
        {
            let data = ctx.data.read().await;
            let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
            if stats.should_track() {
                stats
                    .command_executed(&command_name, command.guild_id)
                    .await;
            }
        }

        match command_name.as_str() {
            "compile" | "compile [beta]" => slashcmds::compile::compile(ctx, command).await,
            "assembly" | "assembly [beta]" => slashcmds::asm::asm(ctx, command).await,
            "ping" => slashcmds::ping::ping(ctx, command).await,
            "help" => slashcmds::help::help(ctx, command).await,
            "cpp" => slashcmds::cpp::cpp(ctx, command).await,
            "invite" => slashcmds::invite::invite(ctx, command).await,
            "format" | "format [beta]" => slashcmds::format::format(ctx, command).await,
            "diff" | "diff [beta]" => {
                if command.data.kind == CommandType::Message {
                    slashcmds::diff_msg::diff_msg(ctx, command).await
                } else {
                    slashcmds::diff::diff(ctx, command).await
                }
            }
            e => {
                warn!("Unknown application command received: {}", e);
                Ok(())
            }
        }
    }

    pub async fn register_commands_guild(&mut self, ctx: &Context, guild: &Guild) {
        match guild
            .set_application_commands(&ctx.http, |setter| {
                setter.set_application_commands(self.commands.clone())
            })
            .await
        {
            Err(e) => error!(
                "Unable to set application commands for guild '{}': {}",
                guild.id, e
            ),
            Ok(commands) => info!(
                "Registered {} commands in guild: {}",
                commands.len(),
                guild.id
            ),
        }
    }

    pub async fn register_commands_global(&mut self, ctx: &Context) {
        if self.commands_registered {
            return;
        }
        self.commands_registered = true;

        match Command::set_global_application_commands(&ctx.http, |setter| {
            setter.set_application_commands(self.commands.clone())
        })
        .await
        {
            Ok(cmds) => info!("Registered {} application commands", cmds.len()),
            Err(e) => error!("Unable to set application commands: {}", e),
        }
    }

    pub fn build_commands() -> Vec<CreateApplicationCommand> {
        let mut cmds = Vec::new();

        let mut cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::Message).name(format!(
            "Compile{}",
            if cfg!(debug_assertions) {
                " [BETA]"
            } else {
                ""
            }
        ));
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::Message).name(format!(
            "Assembly{}",
            if cfg!(debug_assertions) {
                " [BETA]"
            } else {
                ""
            }
        ));
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::Message).name(format!(
            "Format{}",
            if cfg!(debug_assertions) {
                " [BETA]"
            } else {
                ""
            }
        ));
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::Message).name(format!(
            "Diff{}",
            if cfg!(debug_assertions) {
                " [BETA]"
            } else {
                ""
            }
        ));
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::ChatInput)
            .name("help")
            .description("Information on how to use the compiler");
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::ChatInput)
            .name("invite")
            .description("Grab my invite link to invite me to your server");
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::ChatInput)
            .name("ping")
            .description("Test my ping to Discord's endpoint");
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::ChatInput)
            .name("cpp")
            .description("Shorthand C++ compilation using geordi-like syntax")
            .create_option(|opt| {
                opt.required(false)
                    .name("input")
                    .kind(CommandOptionType::String)
                    .description("Geordi-like input")
            });
        cmds.push(cmd);

        cmd = CreateApplicationCommand::default();
        cmd.kind(CommandType::ChatInput)
            .name("diff")
            .description("Posts a diff of two message code blocks")
            .create_option(|opt| {
                opt.required(true)
                    .name("message1")
                    .kind(CommandOptionType::String)
                    .description("Message id of first code-block")
            })
            .create_option(|opt| {
                opt.required(true)
                    .name("message2")
                    .kind(CommandOptionType::String)
                    .description("Message id of second code-block")
            });
        cmds.push(cmd);

        cmds
    }
}
