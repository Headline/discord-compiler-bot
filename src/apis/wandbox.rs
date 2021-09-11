use std::env;
use tokio::sync::RwLockReadGuard;

use serenity::framework::standard::CommandError;
use serenity::builder::{CreateEmbed};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::user::User;

use crate::utls::{parser, discordhelpers};
use crate::cache::{ConfigCache, StatsManagerCache, CompilerCache};
use crate::utls::discordhelpers::{embeds, is_success_embed};
use crate::cppeval::eval::CppEval;
use crate::utls::compilation_manager::CompilationManager;
use crate::utls::parser::ParserResult;


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
    let compilation_manager = data_read.get::<CompilerCache>().unwrap();
    let parse_result = parser::get_components(&content, &author, &compilation_manager, &msg.referenced_message).await?;

    // send out loading emote
    let reaction = match msg
        .react(&ctx.http, discordhelpers::build_reaction(loading_id, &loading_name))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    // dispatch our req
    let compilation_manager_lock : RwLockReadGuard<CompilationManager> = compilation_manager.read().await;
    let awd = compilation_manager_lock.compile(&parse_result, &author).await;
    let result = match awd {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // remove our loading emote
    if msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await
        .is_err()
    {
        return Err(CommandError::from(
                "Unable to remove reactions!\nAm I missing permission to manage messages?",
            ));
    }

    let stats = data_read.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.compilation(&result.0, !is_success_embed(&result.1)).await;
    }

    let mut guild = String::from("<unknown>");
    if let Some(g) = msg.guild_id {
        guild = g.to_string()
    }
    if let Ok(log) = env::var("COMPILE_LOG") {
        if let Ok(id) = log.parse::<u64>() {
            let emb = embeds::build_complog_embed(
                is_success_embed(&result.1),
                &parse_result.code,
                &parse_result.target,
                &author.tag(),
                author.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }

    Ok(result.1)
}

pub async fn send_cpp_request(ctx : Context, content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
    let loading_id;
    let loading_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected ConfigCache in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        loading_id = botinfo
            .get("LOADING_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

    let start = content.find(' ');
    if start.is_none() {
        return Err(CommandError::from(
            "Invalid usage. View `;help cpp`",
        ));
    }

    let mut eval = CppEval::new(content.split_at(start.unwrap()).1);
    let out = eval.evaluate();

    if let Err(e) = out {
        return Err(CommandError::from(
            format!("{}", e),
        ));
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

    let fake_parse = ParserResult {
        url: "".to_string(),
        stdin: "".to_string(),
        target: "gcc-10.1.0".to_string(),
        code: out.unwrap(),
        options: vec![String::from("-O2"), String::from("-std=gnu++2a")]
    };

    let data_read = ctx.data.read().await;
    let compiler_lock = data_read.get::<CompilerCache>().unwrap().read().await;
    let mut result = match compiler_lock.wandbox(&fake_parse).await {
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

    return Ok(embeds::build_small_compilation_embed(&author, &mut result.1));
}