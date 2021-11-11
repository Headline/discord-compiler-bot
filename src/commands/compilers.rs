use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serenity_utils::menu::*;

use crate::cache::{ConfigCache, CompilerCache};
use crate::utls::discordhelpers;
use crate::utls::parser::shortname_to_qualified;
use crate::managers::compilation::RequestHandler;

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

    let language = shortname_to_qualified(&user_lang);
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


    let avatar;
    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        avatar = botinfo.get("BOT_AVATAR").unwrap().clone();
        success_id = botinfo
            .get("SUCCESS_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    // build menu
    let options = discordhelpers::build_menu_controls();
    let pages = discordhelpers::build_menu_items(
        langs,
        15,
        "Supported Compilers",
        &avatar,
        &msg.author.tag(),
        ""
    );
    let menu = Menu::new(ctx, msg, &pages, options);
    match menu.run().await {
        Ok(m) => m,
        Err(e) => {
            // When they click the "X", we get Unknown Message for some reason from serenity_utils
            // We'll manually check for that - and then let us return out
            if e.to_string() == "Unknown Message" {
                match msg
                    .react(
                        &ctx.http,
                        discordhelpers::build_reaction(success_id, &success_name),
                    )
                    .await
                {
                    Ok(r) => r,
                    Err(_e) => {
                        // No need to fail here - this case is handled above
                        return Ok(());
                    }
                };
                return Ok(());
            }

            return Err(CommandError::from(format!(
                "Failed to build languages menu\n{}",
                e
            )));
        }
    };

    debug!("Command executed");
    Ok(())
}
