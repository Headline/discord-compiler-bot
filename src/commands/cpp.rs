use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cppeval::eval::CppEval;

use wandbox::CompilationBuilder;
use crate::cache::{WandboxCache, ConfigCache};
use crate::utls::discordhelpers;

#[command]
#[aliases("c++")]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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


    let start = msg.content.find(' ');
    if let None = start {
        return Err(CommandError::from(
            "Invalid usage. View `;help cpp`",
        ));
    }


    let mut eval = CppEval::new(msg.content.split_at(start.unwrap()).1);
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
    //msg.channel_id.say(&ctx.http, format!("```\n{}\n```", &output)).await?;


    let mut builder = CompilationBuilder::new();
    builder.code(&output);
    builder.target("gcc-10.1.0");
    builder.stdin("");
    builder.save(false);
    builder.options(vec![String::from("-O2")]);

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

    let emb = discordhelpers::build_small_compilation_embed(&msg.author, &mut result, 0);
    let mut emb_msg = discordhelpers::embed_message(emb);

    // Dispatch our request
    let _ = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut emb_msg)
        .await?;

    Ok(())
}

