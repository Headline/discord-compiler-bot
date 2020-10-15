use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};
use crate::cache::{WandboxInfo, BotInfo};
use crate::utls::discordhelpers::DiscordHelpers;
use serenity_utils::menu::*;

#[command]
pub async fn compilers(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // grab language arg
    let language = match _args.parse::<String>() {
        Ok(s) => s,
        Err(_e) => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, "No language specified!\nPlease try giving me a language to search");
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(())
        }
    };


    // y lock on wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxInfo>() {
        Some(l) => l,
        None => {
            // no lock :(
            let emb = DiscordHelpers::build_fail_embed( &msg.author, "Internal request failure.\nWandbox cache is uninitialized, please file a bug if this error persists");
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    // Get our list of compilers
    let wbox = wandbox_lock.read().await;
    let lang = match wbox.get_compilers(&language) {
        Some(s) => s,
        None => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Could not find language '{}'", &language));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(())
        }
    };

    let avatar;
    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        avatar = botinfo.get("BOT_AVATAR").unwrap().clone();
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    // time to build the menu item list
    let mut items : Vec<String> = Vec::new();
    for c in lang {
        items.push( c.name);
    }

    // build menu
    let options = DiscordHelpers::build_menu_controls();
    let pages = DiscordHelpers::build_menu_items(items, 35, "Supported Compilers", &avatar, &msg.author.tag());
    let menu = Menu::new(ctx, msg, &pages, options);
    match menu.run().await {
        Ok(m) => m,
        Err(e) => {
            // When they click the "X", we get Unknown Message for some reason from serenity_utils
            // We'll manually check for that - and then let us return out
            if e.to_string() == "Unknown Message" {
                match msg.react(&ctx.http, DiscordHelpers::build_reaction(success_id, &success_name)).await {
                    Ok(r) => r,
                    Err(_e) => {
                        // No need to fail here - this case is handled above
                        return Ok(());
                    }
                };
                return Ok(())
            }

            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Failed to build languages menu\n{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    debug!("Command executed");
    Ok(())
}