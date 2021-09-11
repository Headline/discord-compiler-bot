use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
};
use serenity_utils::menu::Menu;

use crate::cache::{ConfigCache, MessageCache, CompilerCache};
use crate::utls::constants::*;
use crate::utls::{discordhelpers};
use crate::utls::discordhelpers::embeds;
use crate::utls::parser::shortname_to_qualified;

use serenity::builder::{CreateEmbed};
use serenity::model::channel::Message;
use serenity::model::user::User;

use crate::utls::{parser};

#[command]
#[sub_commands(compilers, languages)]
#[bucket = "nospam"]
pub async fn asm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let emb = handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;
    let mut emb_msg = embeds::embed_message(emb);
    let asm_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut emb_msg)
        .await?;

    // Success/fail react
    let compilation_successful = asm_embed.embeds[0].colour.0 == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &asm_embed, compilation_successful).await?;

    let data_read = ctx.data.read().await;
    let mut message_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    message_cache.insert(msg.id.0, asm_embed.clone());
    debug!("Command executed");
    Ok(())
}

pub async fn handle_request(ctx : Context, mut content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
    let data_read = ctx.data.read().await;
    let loading_id;
    let loading_name;
    {
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        loading_id = botinfo
            .get("LOADING_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

    // Try to load in an attachment
    let attached = parser::get_message_attachment(&msg.attachments).await?;
    if !attached.is_empty() {
        content.push_str(&format!("\n```\n{}\n```\n", attached));
    }

    // parse user input
    let comp_mngr = data_read.get::<CompilerCache>().unwrap();
    let result = match parser::get_components(&content, &author, comp_mngr, &msg.referenced_message).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // send out loading emote
    let reaction = match msg
        .react(
            &ctx.http,
            discordhelpers::build_reaction(loading_id, &loading_name),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    let comp_mngr_lock = comp_mngr.read().await;
    let response = match comp_mngr_lock.assembly(&result, &author).await {
        Ok(resp) => resp,
        Err(e) => {
            // we failed, lets remove the loading react before leaving so it doesn't seem like we're still processing
            msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
                .await?;
            return Err(CommandError::from(format!(
                "Godbolt request failed!\n\n{}",
                e
            )));
        }
    };

    // remove our loading emote
    match msg
        .delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
        .await
    {
        Ok(()) => (),
        Err(_e) => {
            return Err(CommandError::from(
                "Unable to remove reactions!\nAm I missing permission to manage messages?",
            ));
        }
    }

    Ok(response.1)
}

#[command]
async fn compilers(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        return Err(CommandError::from(
            "No language specified, did you mean to supply one?",
        ));
    }

    let data_read = ctx.data.read().await;
    let compiler_cache = data_read.get::<CompilerCache>().unwrap();
    let compiler_manager = compiler_cache.read().await;

    let user_lang = args.parse::<String>().unwrap();
    let language = shortname_to_qualified(&user_lang);
    let mut found = false;
    let mut vec: Vec<String> = Vec::new();
    for cache_entry in &compiler_manager.gbolt.cache {
        if cache_entry.language.id == language {
            found = true;
            for compiler in &cache_entry.compilers {
                vec.push(format!("{} -> **{}**", &compiler.name, &compiler.id));
            }
        }
    }

    if !found {
        return Err(CommandError::from(
            format!("Unable to find compilers for language '{}'", language)
        ));
    }

    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo
            .get("SUCCESS_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    let options = discordhelpers::build_menu_controls();
    let pages = discordhelpers::build_menu_items(
        vec,
        15,
        &format!("\"{}\" compilers", &language),
        COMPILER_EXPLORER_ICON,
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
                "Failed to build asm compilers menu\n{}",
                e
            )));
        }
    };

    debug!("Command executed");
    Ok(())
}

#[command]
async fn languages(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let compiler_cache = data_read.get::<CompilerCache>().unwrap();
    let compiler_manager = compiler_cache.read().await;

    let mut vec: Vec<String> = Vec::new();
    for cache_entry in &compiler_manager.gbolt.cache {
        let lang = &cache_entry.language;
        vec.push(String::from(&lang.id));
    }

    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo
            .get("SUCCESS_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    let options = discordhelpers::build_menu_controls();
    let pages = discordhelpers::build_menu_items(
        vec,
        15,
        "Godbolt languages",
        COMPILER_EXPLORER_ICON,
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
                "Failed to build asm compilers menu\n{}",
                e
            )));
        }
    };

    debug!("Command executed");
    Ok(())
}