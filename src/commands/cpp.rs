use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{MessageCache};
use crate::utls::discordhelpers::embeds;

#[command]
#[aliases("c++")]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let emb = crate::apis::wandbox::send_cpp_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;
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

