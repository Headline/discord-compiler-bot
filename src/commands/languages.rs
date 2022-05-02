use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{CompilerCache, ConfigCache};
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::menu::Menu;

#[command]
pub async fn languages(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let compiler_cache = data_read.get::<CompilerCache>().unwrap();
    let compiler_manager = compiler_cache.read().await;

    let mut items = Vec::new();

    for cache_entry in &compiler_manager.gbolt.cache {
        items.push(format!("{}*", cache_entry.language.id));
    }
    let langs = compiler_manager.wbox.get_languages();
    for lang in langs {
        if !items.contains(&lang.name) && !items.contains(&format!("{}*", &lang.name)) {
            items.push(lang.name);
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

    let mut items_vec: Vec<String> = items.into_iter().collect();
    items_vec.sort();

    let pages = discordhelpers::build_menu_items(
        items_vec,
        15,
        "Supported Languages",
        &avatar,
        &msg.author.tag(),
        "*\\* = supports assembly output*",
    );
    let mut menu = Menu::new(ctx, msg, &pages);
    menu.run().await?;

    debug!("Command executed");
    Ok(())
}
