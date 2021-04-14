use crate::utls::{parser, discordhelpers};
use serenity::framework::standard::CommandError;
use serenity::builder::{CreateEmbed};
use serenity::client::Context;
use serenity::model::channel::Message;
use crate::cache::{ConfigCache, GodboltCache};
use serenity::model::user::User;
use godbolt::{Godbolt, CompilationFilters};
use crate::utls::parser::ParserResult;
use crate::utls::discordhelpers::embeds;

pub async fn send_request(ctx : Context, content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
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

    // parse user input
    let godbolt_lock = data_read.get::<GodboltCache>().unwrap();
    let result: ParserResult = match parser::get_components(&content, &author, godbolt_lock).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
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
            discordhelpers::build_reaction(loading_id, &loading_name),
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

    Ok(embeds::build_asm_embed(&author, &response))
}