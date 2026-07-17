use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{CompilerCache, ConfigCache};
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::menu::Menu;

#[command]
#[bucket = "nospam"]
pub async fn libraries(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // grab language arg
    let user_lang = match args.single::<String>() {
        Ok(s) => s,
        Err(_e) => {
            return Err(CommandError::from(
                "No language specified!\nPlease try giving me a language to search, i.e. ;libraries c++",
            ));
        }
    };

    let mut filter_opt: Option<&str> = Option::None;
    let rest = args.rest();
    if !rest.is_empty() {
        filter_opt = Some(rest);
    };

    let data_read = ctx.data.read().await;
    let library_list = {
        let compiler_cache_lock = data_read
            .get::<CompilerCache>()
            .expect("Expected CompilerCache.")
            .clone();
        let cache = compiler_cache_lock.read().await;
        cache.get_library_list(&user_lang, filter_opt).await?
    };

    if library_list.is_empty() {
        return Err(CommandError::from(match filter_opt {
            Some(filter) => format!(
                "No libraries found for '{}' matching '{}'.",
                user_lang, filter
            ),
            None => format!("No libraries are available for '{}'.", user_lang),
        }));
    }

    let avatar = {
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        botinfo.get("BOT_AVATAR").unwrap().clone()
    };

    let pages = discordhelpers::build_menu_items(
        library_list,
        15,
        "Supported Libraries",
        &avatar,
        &msg.author.name,
        "*Use a library with `-lib <library>:<version>`, or `-lib <library>` for the newest version*",
    );
    let mut menu = Menu::new(ctx, msg, &pages);
    menu.run().await?;

    debug!("Command executed");
    Ok(())
}
