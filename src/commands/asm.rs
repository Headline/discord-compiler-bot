use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::prelude::*,
};
use serenity_utils::menu::Menu;

use godbolt::*;

use crate::cache::{GodboltCache, ConfigCache};
use crate::utls::constants::*;
use crate::utls::parser::*;
use crate::utls::{discordhelpers, parser};

#[command]
#[sub_commands(compilers, languages)]
#[bucket = "nospam"]
pub async fn asm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // parse user input
    let result: ParserResult = match parser::get_components(&msg.content, &msg.author).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // aquire lock to our godbolt cache
    let data_read = ctx.data.read().await;
    let godbolt_lock = match data_read.get::<GodboltCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nGodbolt cache is uninitialized, please file a bug.",
            ));
        }
    };
    let godbolt = godbolt_lock.read().await;

    let c = match godbolt.resolve(&result.target) {
        Some(c) => c,
        None => {
            return Err(CommandError::from(format!(
                "Unable to find valid compiler or language '{}'\n",
                &result.target
            )));
        }
    };

    // send out loading emote
    let reaction = match msg
        .react(
            &ctx.http,
            discordhelpers::build_reaction(752440820036272139, "compiler_loading2"),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    let filters = CompilationFilters {
        binary: None,
        comment_only: Some(true),
        demangle: Some(true),
        directives: Some(true),
        execute: None,
        intel: Some(true),
        labels: Some(true),
        library_code: None,
        trim: Some(true),
    };

    let response =
        match Godbolt::send_request(&c, &result.code, &result.options.join(" "), &filters).await {
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

    let emb = discordhelpers::build_asm_embed(&msg.author, &response);
    let mut emb_msg = discordhelpers::embed_message(emb);
    let asm_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut emb_msg)
        .await?;

    let reaction;
    if response.asm_size.is_some() {
        reaction = discordhelpers::build_reaction(764356794352009246, "checkmark2");
    } else {
        reaction = ReactionType::Unicode(String::from("❌"));
    }

    asm_embed.react(&ctx.http, reaction).await?;
    debug!("Command executed");
    Ok(())
}

#[command]
async fn compilers(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let godbolt_lock = match data_read.get::<GodboltCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nGodbolt cache is uninitialized, please file a bug.",
            ));
        }
    };

    if args.is_empty() {
        return Err(CommandError::from(
            "No language specified, did you mean to supply one?",
        ));
    }

    let language = args.parse::<String>().unwrap();

    let godbolt = godbolt_lock.read().await;
    let mut vec: Vec<String> = Vec::new();
    for cache_entry in &godbolt.cache {
        if cache_entry.language.id == language {
            for compiler in &cache_entry.compilers {
                vec.push(format!("{} -> **{}**", &compiler.name, &compiler.id));
            }
        }
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
    let godbolt_lock = match data_read.get::<GodboltCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nGodbolt cache is uninitialized, please file a bug.",
            ));
        }
    };

    let godbolt = godbolt_lock.read().await;

    let mut vec: Vec<String> = Vec::new();
    for cache_entry in &godbolt.cache {
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
