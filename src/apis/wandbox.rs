use crate::utls::{parser, discordhelpers};
use wandbox::CompilationBuilder;
use serenity::framework::standard::CommandError;
use serenity::builder::{CreateEmbed};
use serenity::client::Context;
use serenity::model::channel::Message;
use crate::cache::{WandboxCache, ConfigCache, StatsManagerCache};
use std::env;
use serenity::model::user::User;
use crate::utls::discordhelpers::embeds;
use crate::cppeval::eval::CppEval;

pub async fn send_request(ctx : Context, mut content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
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
    let wandbox_lock = data_read.get::<WandboxCache>().unwrap();
    let parse_result = parser::get_components(&content, &author, wandbox_lock, &msg.referenced_message).await?;

    // build user input
    let mut builder = CompilationBuilder::new();
    builder.code(&parse_result.code);
    builder.target(&parse_result.target);
    builder.stdin(&parse_result.stdin);
    builder.save(true);
    builder.options(parse_result.options);

    // build request
    {
        let wbox = wandbox_lock.read().await;
        builder.build(&wbox)?;
    }

    // lets see if we can manually fix botched java compilations...
    // for wandbox, "public class" is invalid, so lets do a quick replacement
    if builder.lang == "java" {
        builder.code(&parse_result.code.replacen("public class", "class", 1));
    }

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
    let mut result = match builder.dispatch().await {
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
        stats.compilation(&builder.lang, result.status == "1").await;
    }

    let mut guild = String::from("<unknown>");
    if let Some(g) = msg.guild_id {
        guild = g.to_string()
    }
    if let Ok(log) = env::var("COMPILE_LOG") {
        if let Ok(id) = log.parse::<u64>() {
            let emb = embeds::build_complog_embed(
                result.status == "1",
                &parse_result.code,
                &builder.lang,
                &author.tag(),
                author.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }

    let emb = embeds::build_compilation_embed(&author, &mut result);
    Ok(emb)
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

    let output = out.unwrap();
    let mut builder = CompilationBuilder::new();
    builder.code(&output);
    builder.target("gcc-10.1.0");
    builder.stdin("");
    builder.save(false);
    builder.options(vec![String::from("-O2"), String::from("-std=gnu++2a")]);

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

    return Ok(embeds::build_small_compilation_embed(&author, &mut result));
}