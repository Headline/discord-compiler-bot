use serenity::framework::standard::CommandError;
use serenity::builder::{CreateEmbed};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::user::User;

use crate::cache::{ConfigCache, CompilerCache};
use crate::utls::{parser, discordhelpers};

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
    let comp_mngr = data_read.get::<CompilerCache>().unwrap();
    let result = match parser::get_components(&content, &author, comp_mngr, &msg.referenced_message).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
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

    let comp_mngr_lock = comp_mngr.read().await;
    let response = match comp_mngr_lock.assembly(&result, &author).await {
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

    Ok(response.1)
}

