use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{MessageCache, CompilerCache, ConfigCache};
use crate::utls::discordhelpers::embeds;
use crate::utls::discordhelpers;
use crate::cppeval::eval::CppEval;
use crate::utls::parser::ParserResult;
use serenity::builder::CreateEmbed;
use crate::utls::discordhelpers::embeds::ToEmbed;

#[command]
#[aliases("c++")]
#[bucket = "nospam"]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let emb = handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;
    let mut emb_msg = embeds::embed_message(emb);

    // Dispatch our request
    let compilation_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut emb_msg)
        .await?;

    // add delete cache
    let data_read = ctx.data.read().await;
    let mut delete_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    delete_cache.insert(msg.id.0, compilation_embed);

    Ok(())
}

pub async fn handle_request(ctx : Context, content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
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
        target: "g101".to_string(),
        code: out.unwrap(),
        options: vec![String::from("-O2"), String::from("-std=gnu++2a")],
        args: vec![]
    };

    let data_read = ctx.data.read().await;
    let compiler_lock = data_read.get::<CompilerCache>().unwrap().read().await;
    let result = match compiler_lock.compiler_explorer(&fake_parse).await {
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

    return Ok(result.1.to_embed(&author, false));
}