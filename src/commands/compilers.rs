use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serenity_utils::menu::*;

use crate::cache::{WandboxCache, ConfigCache};
use crate::utls::discordhelpers;

#[command]
pub async fn compilers(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // grab language arg
    let language = match _args.parse::<String>() {
        Ok(s) => s,
        Err(_e) => {
            return Err(CommandError::from(
                "No language specified!\nPlease try giving me a language to search",
            ));
        }
    };

    // y lock on wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from("Internal request failure.\nWandbox cache is uninitialized, please file a bug if this error persists"));
        }
    };

    // Get our list of compilers
    let wbox = wandbox_lock.read().await;
    let lang = match wbox.get_compilers(&language) {
        Some(s) => s,
        None => {
            return Err(CommandError::from(format!(
                "Could not find language '{}'",
                &language
            )));
        }
    };

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

    // time to build the menu item list
    let mut items: Vec<String> = Vec::new();
    for c in lang {
        items.push(c.name);
    }

    // build menu
    let options = discordhelpers::build_menu_controls();
    let pages = discordhelpers::build_menu_items(
        items,
        35,
        "Supported Compilers",
        &avatar,
        &msg.author.tag(),
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
