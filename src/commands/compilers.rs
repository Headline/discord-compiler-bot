use crate::framework::{CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{CompilerCache, ConfigCache};
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::menu::Menu;

pub async fn compilers(ctx: &Context, msg: &Message, args: &str) -> CommandResult {
    // grab language arg
    let mut parts = args.splitn(2, char::is_whitespace);
    let user_lang = match parts.next().filter(|s| !s.is_empty()) {
        Some(s) => s.to_owned(),
        None => {
            return Err(CommandError::from(
                "No language specified!\nPlease try giving me a language to search",
            ));
        }
    };

    let mut filter_opt: Option<&str> = Option::None;
    let rest = parts.next().unwrap_or_default().trim();
    if !rest.is_empty() {
        filter_opt = Some(rest);
    };

    let data_read = ctx.data.read().await;
    let compiler_list = {
        let compiler_cache_lock = data_read
            .get::<CompilerCache>()
            .expect("Expected CompilerCache.")
            .clone();
        let cache = compiler_cache_lock.read().await;
        cache.get_compiler_list(&user_lang, filter_opt)?
    };

    let avatar = {
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        botinfo.get("BOT_AVATAR").unwrap().clone()
    };

    let pages = discordhelpers::build_menu_items(
        compiler_list,
        15,
        "Supported Compilers",
        &avatar,
        &msg.author.name,
        "",
    );
    let mut menu = Menu::new(ctx, msg, &pages);
    menu.run().await?;

    debug!("Command executed");
    Ok(())
}
