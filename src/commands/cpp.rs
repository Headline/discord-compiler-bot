use serenity::{
    builder::CreateEmbed,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::utls::discordhelpers::embeds::EmbedOptions;
use crate::{
    cache::{CompilerCache, ConfigCache, MessageCache, MessageCacheEntry},
    cppeval::eval::CppEval,
    utls::discordhelpers,
    utls::discordhelpers::embeds,
    utls::discordhelpers::embeds::ToEmbed,
    utls::parser::ParserResult,
};

#[command]
#[aliases("c++")]
#[bucket = "nospam"]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let emb = handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;

    // Dispatch our request
    let compilation_embed = embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await?;

    // add delete cache
    let data_read = ctx.data.read().await;
    let mut delete_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    delete_cache.insert(
        msg.id.0,
        MessageCacheEntry::new(compilation_embed, msg.clone()),
    );

    Ok(())
}

pub async fn handle_request(
    ctx: Context,
    content: String,
    author: User,
    msg: &Message,
) -> std::result::Result<CreateEmbed, CommandError> {
    let loading_reaction = {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        if let Some(loading_id) = botinfo.get("LOADING_EMOJI_ID") {
            let loading_name = botinfo
                .get("LOADING_EMOJI_NAME")
                .expect("Unable to find loading emoji name")
                .clone();
            discordhelpers::build_reaction(loading_id.parse::<u64>()?, &loading_name)
        } else {
            ReactionType::Unicode(String::from("⏳"))
        }
    };

    let start = content.find(' ');
    if start.is_none() {
        return Err(CommandError::from("Invalid usage. View `;help cpp`"));
    }

    let mut eval = CppEval::new(content.split_at(start.unwrap()).1);
    let out = eval.evaluate()?;

    // send out loading emote
    if msg
        .react(&ctx.http, loading_reaction.clone())
        .await
        .is_err()
    {
        return Err(CommandError::from(
            "Unable to react to message, am I missing permissions to react or use external emoji?",
        ));
    }

    let fake_parse = ParserResult {
        url: "".to_string(),
        stdin: "".to_string(),
        target: "g101".to_string(),
        code: out,
        options: vec![String::from("-O2"), String::from("-std=gnu++2a")],
        args: vec![],
    };

    let data_read = ctx.data.read().await;
    let compiler_lock = data_read.get::<CompilerCache>().unwrap().read().await;
    let result = match compiler_lock.compiler_explorer(&fake_parse).await {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction.clone()).await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // remove our loading emote
    discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction).await?;
    let options = EmbedOptions::new(false, fake_parse.target.clone(), String::default());

    Ok(result.1.to_embed(&author, &options))
}
