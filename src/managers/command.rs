use crate::cache::StatsManagerCache;
use crate::slashcmds;

use serenity::all::{Command, CommandInteraction, CommandType, CreateCommand};
use serenity::{client::Context, framework::standard::CommandResult, model::guild::Guild};

pub struct CommandManager {
    commands_registered: bool,
    commands: Vec<CreateCommand>,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            commands_registered: false,
            commands: CommandManager::build_commands(),
        }
    }

    pub async fn on_command(&self, ctx: &Context, command: &CommandInteraction) -> CommandResult {
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
            "ping" => slashcmds::ping::ping(ctx, command).await,
            "help" => slashcmds::help::help(ctx, command).await,
            "invite" => slashcmds::invite::invite(ctx, command).await,
            "format" | "format [beta]" => slashcmds::format::format(ctx, command).await,
            "diff" | "diff [beta]" => slashcmds::diff_msg::diff_msg(ctx, command).await,
            e => {
                warn!("Unknown application command received: {}", e);
                Ok(())
            }
        }
    }

    pub async fn register_commands_guild(&mut self, ctx: &Context, guild: &Guild) {
        match guild.set_commands(&ctx.http, self.commands.clone()).await {
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

        match Command::set_global_commands(&ctx.http, self.commands.clone()).await {
            Ok(cmds) => info!("Registered {} application commands", cmds.len()),
            Err(e) => error!("Unable to set application commands: {}", e),
        }
    }

    pub fn build_commands() -> Vec<CreateCommand> {
        let mut cmds = Vec::new();

        cmds.push(
            CreateCommand::new(format!(
                "Format{}",
                if cfg!(debug_assertions) {
                    " [BETA]"
                } else {
                    ""
                }
            ))
            .kind(CommandType::Message),
        );

        cmds.push(
            CreateCommand::new(format!(
                "Diff{}",
                if cfg!(debug_assertions) {
                    " [BETA]"
                } else {
                    ""
                }
            ))
            .kind(CommandType::Message),
        );

        cmds.push(
            CreateCommand::new("help")
                .description("Information on how to use the compiler")
                .kind(CommandType::ChatInput),
        );

        cmds.push(
            CreateCommand::new("invite")
                .description("Grab my invite link to invite me to your server")
                .kind(CommandType::ChatInput),
        );

        cmds.push(
            CreateCommand::new("ping")
                .description("Test my ping to Discord's endpoint")
                .kind(CommandType::ChatInput),
        );

        cmds
    }
}
