use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
};

use crate::cache::{ConfigCache, MessageCache, CompilerCache, MessageCacheEntry};
use crate::utls::constants::*;
use crate::utls::{discordhelpers};
use crate::utls::discordhelpers::embeds;

use serenity::builder::{CreateEmbed};
use serenity::model::channel::{Message, ReactionType};
use serenity::model::user::User;

use crate::utls::{parser};

#[command]
#[bucket = "nospam"]
pub async fn asm(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let emb = handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;
    let mut emb_msg = embeds::embed_message(emb);
    let asm_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut emb_msg)
        .await?;

    // Success/fail react
    let compilation_successful = asm_embed.embeds[0].colour.unwrap().0 == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &asm_embed, compilation_successful).await?;

    let data_read = ctx.data.read().await;
    let mut message_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    message_cache.insert(msg.id.0, MessageCacheEntry::new(asm_embed.clone(), msg.clone()));
    debug!("Command executed");
    Ok(())
}

pub async fn handle_request(ctx : Context, mut content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
    let data_read = ctx.data.read().await;
    let loading_reaction = {
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        if let Some(loading_id) = botinfo.get("LOADING_EMOJI_ID") {
            let loading_name = botinfo.get("LOADING_EMOJI_NAME").expect("Unable to find loading emoji name").clone();
            discordhelpers::build_reaction(loading_id.parse::<u64>()?, &loading_name)
        }
        else {
            ReactionType::Unicode(String::from("⏳"))
        }
    };

    // Try to load in an attachment
    let (code, ext) = parser::get_message_attachment(&msg.attachments).await?;
    if !code.is_empty() {
        content.push_str(&format!("\n```{}\n{}\n```\n", ext, code));
    }

    // parse user input
    let comp_mngr = data_read.get::<CompilerCache>().unwrap();
    let result = match parser::get_components(&content, &author, Some(comp_mngr), &msg.referenced_message).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // send out loading emote
    if let Err(_) = msg.react(&ctx.http, loading_reaction.clone()).await {
        return Err(CommandError::from("Unable to react to message, am I missing permissions to react or use external emoji?\n{}"))
    }

    let comp_mngr_lock = comp_mngr.read().await;
    let response = match comp_mngr_lock.assembly(&result, &author).await {
        Ok(resp) => resp,
        Err(e) => {
            discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction).await?;
            return Err(CommandError::from(format!(
                "Godbolt request failed!\n\n{}",
                e
            )));
        }
    };

    // remove our loading emote
    discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction).await?;

    Ok(response.1)
}
