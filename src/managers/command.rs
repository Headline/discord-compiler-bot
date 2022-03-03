use std::sync::Arc;
use serenity::http::Http;
use serenity::model::interactions::application_command::ApplicationCommand;

pub struct CommandManager {

}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {

        }
    }

    pub async fn register_commands(&self, http: &Arc<Http>) {
        let _ =
            ApplicationCommand::create_global_application_command(http, |command| {
                command.name("wonderful_command").description("An amazing command")
            }).await;
    }
}