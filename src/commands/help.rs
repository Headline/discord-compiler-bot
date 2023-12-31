use std::env;

use serenity::all::CreateMessage;
use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::utls::constants::*;
use crate::utls::discordhelpers::embeds;

#[command]
pub async fn help(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let prefix = env::var("BOT_PREFIX").expect("Bot prefix is not set!");
    if !args.is_empty() {
        let cmd = args.parse::<String>().unwrap();
        let mut emb = CreateEmbed::default()
            .thumbnail(ICON_HELP)
            .color(COLOR_OKAY);

        let unknown = format!("Unknown command '{}'", cmd);
        let description = match cmd.as_str() {
            "help" => "Do you like recursion or something?",
            "invite" => {
                emb = emb.title("Invite command").field(
                    "Example",
                    format!("{}invite", prefix),
                    false,
                );
                "Grabs the bot's invite link\n\n"
            }
            "compile" => {
                emb = emb.title("Compile command").field(
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
                emb = emb.title("Compilers command").field(
                    "Example",
                    format!("{}compilers <language>", prefix),
                    false,
                );
                "Lists all compilers supported for a given language"
            }

            "cpp" | "c++" => {
                emb = emb.title("c++/cpp command")
                    .field(
                        "Example 1",
                        format!(
                            "{}cpp {{ int a = 4; if (a > 3) {{ cout << \"true\"; }} }}",
                            prefix
                        ),
                        false,
                    )
                    .field(
                        "Example 2",
                        format!("{}cpp << (4*12) << \"Hello world!\"", prefix),
                        false,
                    )
                    .field(
                        "Example 3",
                        format!("{}cpp << f(2); int f(int a) {{ return a*12; }}", prefix),
                        false,
                    )
                    .field("Example 4", format!("{}cpp int main() {{ cout << \"Main\"; f(); }} void f() {{ cout << \"f()\"; }}", prefix), false)
                    .field("Example 5", format!("*You may also use in-line code blocks if discord makes you escape some chars*\n{}cpp `<< (4*12) << \"\\\"Hello world!\\\"\"`", prefix), false);
                "Allows you to quickly compile and execute c++ snippets using geordi-like syntax.\nSee section 2.1 of http://eel.is/geordi/#syntax"
            }

            "languages" => {
                emb = emb.title("Languages command").field(
                    "Example",
                    format!("{}languages", prefix),
                    false,
                );
                "Lists all languages supported"
            }
            "asm" => {
                emb = emb.title("Assembly command").field(
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
                emb = emb.title("Bot info command").field(
                    "Example",
                    format!("{}botinfo", prefix),
                    false,
                );
                "Outputs information about the bot"
            }
            "format" => {
                emb = emb
                    .title("Format command")
                    .field("Example", format!("{}format clang Google", prefix), false)
                    .field("Example", format!("{}format clang Mozilla", prefix), false)
                    .field("Example", format!("{}format rustfmt", prefix), false);
                "Formats the input code with the formatter specified. Defaults to clang-format WebKit\n\n*(see .formats command for all formats)*\n\n"
            }
            _ => {
                emb = emb
                    .title("Command not found")
                    .color(COLOR_FAIL)
                    .thumbnail(ICON_FAIL);
                unknown.as_str()
            }
        };

        emb = emb.description(description);
        embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await?;

        return Ok(());
    }

    let prefix = env::var("BOT_PREFIX").expect("Prefix has not been set!");
    let embed = CreateEmbed::new()
        .thumbnail(ICON_HELP)
        .description(format!("For help with a specific command, type `{}help <command>`\n\nStruggling? Check out [our wiki](https://github.com/Headline/discord-compiler-bot/wiki)", prefix))
        .color(COLOR_OKAY)
        .title("Commands")
        .field("invite", "``` Grabs the bot's invite link ```", false)
        .field("compile", "``` Compiles and executes code ```", false)
        .field("compilers", "``` Displays the compilers for the specified language ```", false)
        .field("languages", "``` Displays all supported languages ```", false)
        .field("asm", "``` Outputs the assembly for the input code```", false)
        .field("botinfo", "``` Displays information about the bot ```", false)
        .field("cpp", format!("``` Executes c++ code using geordi-like syntax\n See {}help cpp for more info ```", prefix), false)
        .field("format", "``` Formats code using a code formatter (i.e. clang-format or rustfmt) ```", false)
        .field("formats", "``` Displays all formatting options & styles ```", false)
        .field("insights", "``` Sends a code block to cppinsights.io ```", false);

    let new_msg = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, new_msg).await?;

    debug!("Command executed");
    Ok(())
}
