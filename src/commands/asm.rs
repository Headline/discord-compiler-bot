pub mod hide {
    use serenity::prelude::*;
    use serenity::model::prelude::*;
    use serenity::framework::standard::{
        Args, CommandResult,
        macros::command,
    };

    use godbolt::*;

    use crate::cache::{GodboltInfo};
    use crate::utls::parser::{Parser, ParserResult};
    use crate::utls::discordhelpers::*;

    use crate::commands::asm::*;
    #[command]
    #[sub_commands(compilers,languages)]
    pub async fn asm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
        // parse user input
        let result : ParserResult = match Parser::get_components(&msg.content).await {
            Ok(r) => r,
            Err(e) => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("{}", e));
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

                return Ok(());
            }
        };


        // aquire lock to our godbolt cache
        let data_read = ctx.data.read().await;
        let godbolt_lock = match data_read.get::<GodboltInfo>() {
            Some(l) => l,
            None => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, "Internal request failure\nGodbolt cache is uninitialized, please file a bug.");
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

                return Ok(());
            }
        };
        let godbolt = godbolt_lock.read().await;

        let c = match godbolt.resolve(&result.target) {
            Some(c) => c,
            None => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Unable to find valid compiler or language '{}'\n", &result.target));
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

                return Ok(());
            }
        };


        // send out loading emote
        let reaction = match msg.react(&ctx.http, DiscordHelpers::build_reaction(752440820036272139, "compiler_loading2")).await {
            Ok(r) => r,
            Err(e) => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Unable to react to message, am I missing permissions?\n{}", e));
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

                return Ok(());
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
            trim: Some(true)
        };

        let response = match Godbolt::send_request(&c, &result.code, &result.options.join(" "), &filters).await {
            Ok(resp) => resp,
            Err(e) => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Godbolt request failed!\n\n{}", e));
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

                // we failed, lets remove the loading react before leaving so it doesn't seem like we're still processing
                msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await?;
                return Ok(());
            }
        };

        // remove our loading emote
        match msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await {
            Ok(()) => (),
            Err(_e) => {
                let emb = DiscordHelpers::build_fail_embed( &msg.author, "Unable to remove reactions!\nAm I missing permission to manage messages?");
                let mut emb_msg = DiscordHelpers::embed_message(emb);
                msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;
            }
        }

        let emb = DiscordHelpers::build_asm_embed( &msg.author, &response);
        let mut emb_msg = DiscordHelpers::embed_message(emb);
        let asm_embed = msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;


        let reaction;
        if response.asm_size.is_some() {
            reaction = DiscordHelpers::build_reaction(764356794352009246, "checkmark2");
        }
        else {
            reaction = ReactionType::Unicode(String::from("❌"));
        }

        asm_embed.react(&ctx.http, reaction).await?;
        debug!("Command executed");
        Ok(())
    }
}

use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};
use serenity_utils::menu::Menu;


use crate::cache::{GodboltInfo, BotInfo};
use crate::utls::discordhelpers::*;
use crate::utls::constants::*;

#[command]
async fn compilers(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let godbolt_lock = match data_read.get::<GodboltInfo>() {
        Some(l) => l,
        None => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, "Internal request failure\nGodbolt cache is uninitialized, please file a bug.");
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    if args.is_empty() {
        let emb = DiscordHelpers::build_fail_embed( &msg.author, "Unable to find language '', did you mean to supply one?");
        let mut emb_msg = DiscordHelpers::embed_message(emb);
        msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;
        return Ok(());
    }
    let godbolt = godbolt_lock.read().await;

    let language = args.parse::<String>().unwrap();

    let mut vec : Vec<String> = Vec::new();
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
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    let options = DiscordHelpers::build_menu_controls();
    let pages = DiscordHelpers::build_menu_items(vec, 15, &format!("\"{}\" compilers", &language), COMPILER_EXPLORER_ICON, &msg.author.tag());
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

            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Failed to build asm compilers menu\n{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };


    debug!("Command executed");
    Ok(())
}

#[command]
async fn languages(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let godbolt_lock = match data_read.get::<GodboltInfo>() {
        Some(l) => l,
        None => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, "Internal request failure\nGodbolt cache is uninitialized, please file a bug.");
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    let godbolt = godbolt_lock.read().await;

    let mut vec : Vec<String> = Vec::new();
    for cache_entry in &godbolt.cache {
        let lang = &cache_entry.language;
        vec.push(format!("{}", &lang.id));
    }


    let success_id;
    let success_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
    }

    let options = DiscordHelpers::build_menu_controls();
    let pages = DiscordHelpers::build_menu_items(vec, 15, &format!("Godbolt languages"), COMPILER_EXPLORER_ICON, &msg.author.tag());
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

            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("Failed to build asm compilers menu\n{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };


    debug!("Command executed");
    Ok(())
}