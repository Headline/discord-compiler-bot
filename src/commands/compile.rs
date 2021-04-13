use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{MessageCache};
use crate::utls::{discordhelpers};
use crate::utls::constants::COLOR_OKAY;
use crate::utls::discordhelpers::embeds;

#[command]
#[bucket = "nospam"]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;

    // Handle wandbox request logic
    let embed = crate::apis::wandbox::send_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;

    // Send our final embed
    let mut message = embeds::embed_message(embed);
    let compilation_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut message)
        .await?;

    // Success/fail react
    let compilation_successful = compilation_embed.embeds[0].colour.0 == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &compilation_embed, compilation_successful).await?;

    let mut delete_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    delete_cache.insert(msg.id.0, compilation_embed);
    debug!("Command executed");
    Ok(())
}
