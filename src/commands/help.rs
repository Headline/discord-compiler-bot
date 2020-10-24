use std::env;

use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::utls::constants::*;
use crate::utls::discordhelpers;

#[command]
pub async fn help(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let prefix = env::var("BOT_PREFIX").expect("Bot prefix is not set!");
    if !args.is_empty() {
        let cmd = args.parse::<String>().unwrap();
        let mut emb = CreateEmbed::default();
        emb.thumbnail(ICON_HELP);
        emb.color(COLOR_OKAY);

        let unknown = format!("Unknown command '{}'", cmd);
        let description = match cmd.as_str() {
            "help" => "Do you like recursion or something?",
            "invite" => {
                emb.title("Invite command");
                emb.field("Example", format!("{}invite", prefix), false);
                "Grabs the bot's invite link\n\n"
            }
            "compile" => {
                emb.title("Compile command");
                emb.field(
                    "Example",
                    format!(
                        "{}compile c++\n\
          \\`\\`\\`\n\
          #include <iostream>\n\n\
          int main() {{ \n\
          \tstd::cout << \"Hello, world\";\n\
          }}\n\
          \\`\\`\\`\n",
                        prefix
                    ),
                    false,
                );
                "Sends a compilation request\n\n"
            }
            "compilers" => {
                emb.title("Compilers command");
                emb.field("Example", format!("{}compilers <language>", prefix), false);
                "Lists all compilers supported for a given language"
            }
            "languages" => {
                emb.title("Languages command");
                emb.field("Example", format!("{}languages", prefix), false);
                "Lists all languages supported"
            }
            "asm" => {
                emb.title("Assembly command");
                emb.field(
                    "Example",
                    format!(
                        "{}asm c++\n\
          \\`\\`\\`\n\
          #include <iostream>\n\n\
          int main() {{ \n\
          \tstd::cout << \"Hello, world\";\n\
          }}\n\
          \\`\\`\\`\n",
                        prefix
                    ),
                    false,
                );
                "Sends an assembly request, displaying the assembly output\n\n"
            }
            "botinfo" => {
                emb.title("Bot info command");
                emb.field("Example", format!("{}botinfo", prefix), false);
                "Outputs information about the bot"
            }
            _ => {
                emb.title("Command not found");
                emb.color(COLOR_FAIL);
                emb.thumbnail(ICON_FAIL);
                unknown.as_str()
            }
        };

        emb.description(description);

        let mut emb_msg = discordhelpers::embed_message(emb);
        msg.channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await?;

        return Ok(());
    }

    let prefix = env::var("BOT_PREFIX").expect("Prefix has not been set!");
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.thumbnail(ICON_HELP);
            e.description(format!("For help with a specific command, type `{}help <command>`\n\nStruggling? Check out [our wiki](https://github.com/Headline/discord-compiler-bot/wiki)", prefix));
            e.color(COLOR_OKAY);
            e.title("Commands");
            e.field("invite", "``` Grabs the bot's invite link ```", false);
            e.field("compile", "``` Compiles a script ```", false);
            e.field("compilers", "``` Displays the compilers for the specified language ```", false);
            e.field("languages", "``` Displays all supported languages ```", false);
            e.field("asm", "```\nOutputs the assembly for the input code```", false);
            e.field("botinfo", "``` Displays information about the bot ```", false);
            e
        })
    }).await?;

    debug!("Command executed");
    Ok(())
}
