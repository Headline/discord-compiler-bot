use std::env;

use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use wandbox::*;

use crate::cache::{ConfigCache, WandboxCache, StatsManagerCache, MessageDeleteCache};
use crate::utls::{discordhelpers, parser, parser::*};
use serenity_utils::menu::Menu;

#[command]
#[bucket = "nospam"]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let success_id;
    let success_name;
    let loading_id;
    let loading_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected ConfigCache in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo
            .get("SUCCESS_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
        loading_id = botinfo
            .get("LOADING_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

    // parse user input
    let parse_result: ParserResult = parser::get_components(&msg.content, &msg.author).await?;

    // build user input
    let mut builder = CompilationBuilder::new();
    builder.code(&parse_result.code);
    builder.target(&parse_result.target);
    builder.stdin(&parse_result.stdin);
    builder.save(true);
    builder.options(parse_result.options);

    // aquire lock to our wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nWandbox cache is uninitialized, please file a bug.",
            ));
        }
    };
    let wbox = wandbox_lock.read().await;

    // build request
    match builder.build(&wbox) {
        Ok(()) => (),
        Err(e) => {
            return Err(CommandError::from(format!(
                "An internal error has occurred while building request.\n{}",
                e
            )));
        }
    };

    // lets see if we can manually fix botched java compilations...
    // for wandbox, "public class" is invalid, so lets do a quick replacement
    if builder.lang == "java" {
        builder.code(&parse_result.code.replacen("public class", "class", 1));
    }

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

    // dispatch our req
    let mut result = match builder.dispatch().await {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
                .await?;

            return Err(CommandError::from(format!("{}", e)));
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

    let page_count = discordhelpers::get_page_count(&result);
    if page_count > 1 {
        // build pages
        let mut pages = Vec::new();
        let mut i : i32 = 0;
        while i < page_count as i32 {
            let emb = discordhelpers::build_compilation_embed(&msg.author, &mut result, i);
            let emb_msg = discordhelpers::embed_message(emb);
            pages.push(emb_msg);
            i += 1;
        }

        // create menu
        let options = discordhelpers::build_compile_controls();
        let menu = Menu::new(ctx, msg, &pages, options);
        match menu.run().await {
            Ok(sent_msg) => {
                // add message to delete cache
                if let Some(m) = sent_msg {
                    let data_read = ctx.data.read().await;
                    let mut delete_cache = data_read.get::<MessageDeleteCache>().unwrap().lock().await;
                    delete_cache.insert(msg.id.0, m.clone());

                    let reaction;
                    if result.status == "0" {
                        reaction = discordhelpers::build_reaction(success_id, &success_name);
                    } else {
                        reaction = ReactionType::Unicode(String::from("❌"));
                    }
                    m.react(&ctx.http, reaction).await?;

                }
            },
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
    }
    else { // single page - display normally.
        let emb = discordhelpers::build_compilation_embed(&msg.author, &mut result, 0);
        let mut emb_msg = discordhelpers::embed_message(emb);

        // Dispatch our request
        let m = msg
            .channel_id
            .send_message(&ctx.http, |_| &mut emb_msg)
            .await?;

        // Success/fail react
        let reaction;
        if result.status == "0" {
            reaction = discordhelpers::build_reaction(success_id, &success_name);
        } else {
            reaction = ReactionType::Unicode(String::from("❌"));
        }
        m.react(&ctx.http, reaction).await?;
        let data_read = ctx.data.read().await;
        let mut delete_cache = data_read.get::<MessageDeleteCache>().unwrap().lock().await;
        delete_cache.insert(msg.id.0, m);
    }


    let data = ctx.data.read().await;
    let stats = data.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.compilation(&builder.lang, result.status == "1").await;
    }

    let mut guild = String::from("<unknown>");
    if let Some(g) = msg.guild_id {
        guild = g.to_string()
    }
    if let Ok(log) = env::var("COMPILE_LOG") {
        if let Ok(id) = log.parse::<u64>() {
            let emb = discordhelpers::build_complog_embed(
                result.status == "1",
                &parse_result.code,
                &builder.lang,
                &msg.author.tag(),
                msg.author.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }
    debug!("Command executed");
    Ok(())
}
