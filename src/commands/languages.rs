use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serenity_utils::menu::*;

use crate::cache::{ConfigCache, CompilerCache};
use crate::utls::discordhelpers;

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

    let mut items_vec : Vec<String> = items.into_iter().collect();
    items_vec.sort();

    let options = discordhelpers::build_menu_controls();
    let pages = discordhelpers::build_menu_items(
        items_vec,
        15,
        "Supported Languages",
        &avatar,
        &msg.author.tag(),
        "*\\* = supports assembly output*"
    );
    let menu = Menu::new(ctx, msg, &pages, options);
    match menu.run().await {
        Ok(m) => m,
        Err(e) => {
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
