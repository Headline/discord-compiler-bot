use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{CompilerCache, ConfigCache};
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::menu::Menu;

#[command]
pub async fn compilers(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    // grab language arg
    let user_lang = match _args.single::<String>() {
        Ok(s) => s,
        Err(_e) => {
            return Err(CommandError::from(
                "No language specified!\nPlease try giving me a language to search",
            ));
        }
    };

    let mut filter_opt: Option<&str> = Option::None;
    let rest = _args.rest();
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
        cache.get_compiler_list(user_lang, filter_opt)?
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
