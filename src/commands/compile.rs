use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};

use crate::cache::{WandboxInfo, BotInfo};
use wandbox::*;

use crate::utls::parser::{Parser, ParserResult};
use crate::utls::discordhelpers::*;

#[command]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    let success_id;
    let success_name;
    let loading_id;
    let loading_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<BotInfo>().expect("Expected BotInfo in global cache").clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo.get("SUCCESS_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
        loading_id = botinfo.get("LOADING_EMOJI_ID").unwrap().clone().parse::<u64>().unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

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

    // build user input
    let mut builder = CompilationBuilder::new();
    builder.code(&result.code);
    builder.target(&result.target);
    builder.stdin(&result.stdin);
    builder.save(true);
    builder.options(result.options);


    // aquire lock to our wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxInfo>() {
        Some(l) => l,
        None => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, "Internal request failure\nWandbox cache is uninitialized, please file a bug.");
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };
    let wbox = wandbox_lock.read().await;

    // build request
    match builder.build(&wbox) {
        Ok(()) => (),
        Err(e) => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("An internal error has occurred while building request.\n{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    // send out loading emote
    let reaction = match msg.react(&ctx.http, DiscordHelpers::build_reaction(loading_id, &loading_name)).await {
        Ok(r) => r,
        Err(e) => {
            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!(" Unable to react to message, am I missing permissions?\n{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            return Ok(());
        }
    };

    // dispatch our req
    let result = match builder.dispatch().await {
        Ok(r) => r,
        Err(e) => {

            let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("{}", e));
            let mut emb_msg = DiscordHelpers::embed_message(emb);
            msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

            // we failed, lets remove the loading react so it doesn't seem like we're still processing
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

    // Dispatch our request
    let emb = DiscordHelpers::build_compilation_embed( &msg.author, &result);
    let mut emb_msg = DiscordHelpers::embed_message(emb);
    let compilation_embed = msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await?;

    let reaction;
    if result.status == "0" {
        reaction = DiscordHelpers::build_reaction(success_id, &success_name);
    }
    else {
        reaction = ReactionType::Unicode(String::from("❌"));
    }
    compilation_embed.react(&ctx.http, reaction).await?;
    debug!("Command executed");
    Ok(())
}

