use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{ConfigCache, CompilerCache};
use crate::utls::discordhelpers;
use crate::utls::parser::shortname_to_qualified;
use crate::managers::compilation::RequestHandler;
use crate::utls::discordhelpers::menu::Menu;

#[command]
pub async fn compilers(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // grab language arg
    let user_lang = match _args.parse::<String>() {
        Ok(s) => s,
        Err(_e) => {
            return Err(CommandError::from(
                "No language specified!\nPlease try giving me a language to search",
            ));
        }
    };

    let data_read = ctx.data.read().await;
    let compiler_cache = data_read.get::<CompilerCache>().unwrap();
    let compiler_manager = compiler_cache.read().await;

    // Get our list of compilers
    let mut langs: Vec<String> = Vec::new();

    let lower_lang = user_lang.to_lowercase();
    let language = shortname_to_qualified(&lower_lang);
    match compiler_manager.resolve_target(language) {
        RequestHandler::CompilerExplorer => {
            for cache_entry in &compiler_manager.gbolt.cache {
                if cache_entry.language.id == language {
                    for compiler in &cache_entry.compilers {
                        langs.push(format!("{} -> **{}**", &compiler.name, &compiler.id));
                    }
                }
            }
        }
        RequestHandler::WandBox => {
            match compiler_manager.wbox.get_compilers(&shortname_to_qualified(&language)) {
                Some(s) =>  {
                    for lang in s {
                        langs.push(lang.name);
                    }
                },
                None => {
                    return Err(CommandError::from(
                        format!("Unable to find compilers for target '{}'.", language),
                    ));
                }
            };
        }
        RequestHandler::None => {
            return Err(CommandError::from(
                format!("Unable to find compilers for target '{}'.", language),
            ));
        }
    }


    let avatar = {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        botinfo.get("BOT_AVATAR").unwrap().clone()
    };

    let pages = discordhelpers::build_menu_items(
        langs,
        15,
        "Supported Compilers",
        &avatar,
        &msg.author.tag(),
        ""
    );
    let mut menu = Menu::new(ctx, msg, &pages);
    menu.run().await?;

    debug!("Command executed");
    Ok(())
}
