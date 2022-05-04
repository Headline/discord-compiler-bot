pub mod embeds;
pub mod interactions;
pub mod menu;

use std::str;
use std::sync::Arc;

use serenity::{builder::CreateEmbed, http::Http, model::prelude::*};

use crate::cache::ConfigCache;
use crate::utls::constants::*;
use crate::utls::discordhelpers;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use tokio::sync::MutexGuard;

pub fn build_menu_items(
    items: Vec<String>,
    items_per_page: usize,
    title: &str,
    avatar: &str,
    author: &str,
    desc: &str,
) -> Vec<CreateEmbed> {
    let mut pages = Vec::new();
    let num_pages = items.len() / items_per_page;

    let mut current_page = 0;
    while current_page < num_pages + 1 {
        let start = current_page * items_per_page;
        let mut end = start + items_per_page;
        if end > items.len() {
            end = items.len();
        }
        let mut emb = CreateEmbed::default();
        let mut description = format!("{}\n", desc);
        for (i, item) in items[current_page * items_per_page..end].iter().enumerate() {
            if i > items_per_page {
                break;
            }
            description.push_str(&format!(
                "**{}**) {}\n",
                current_page * items_per_page + i + 1,
                item
            ))
        }
        emb.color(COLOR_OKAY);
        emb.title(title);
        emb.description(description);
        emb.footer(|f| {
            f.text(&format!(
                "Requested by {} | Page {}/{}",
                author,
                current_page + 1,
                num_pages + 1
            ))
        });
        emb.thumbnail(avatar);

        pages.push(emb);
        current_page += 1;
    }

    pages
}

// Pandas#3**2 on serenity disc, tyty
pub fn build_reaction(emoji_id: u64, emoji_name: &str) -> ReactionType {
    ReactionType::Custom {
        animated: false,
        id: EmojiId::from(emoji_id),
        name: Some(String::from(emoji_name)),
    }
}

pub async fn handle_edit(
    ctx: &Context,
    content: String,
    author: User,
    mut old: Message,
    original_message: Message,
) {
    let prefix = {
        let data = ctx.data.read().await;
        let info = data.get::<ConfigCache>().unwrap().read().await;
        info.get("BOT_PREFIX").unwrap().to_owned()
    };

    // try to clear reactions
    let _ = old.delete_reactions(&ctx).await;

    if content.starts_with(&format!("{}asm", prefix)) {
        if let Err(e) = handle_edit_asm(
            ctx,
            content,
            author.clone(),
            old.clone(),
            original_message.clone(),
        )
        .await
        {
            let err = embeds::build_fail_embed(&author, &e.to_string());
            embeds::edit_message_embed(ctx, &mut old, err).await;
        }
    } else if content.starts_with(&format!("{}compile", prefix)) {
        if let Err(e) = handle_edit_compile(
            ctx,
            content,
            author.clone(),
            old.clone(),
            original_message.clone(),
        )
        .await
        {
            let err = embeds::build_fail_embed(&author, &e.to_string());
            embeds::edit_message_embed(ctx, &mut old, err).await;
        }
    } else if content.starts_with(&format!("{}cpp", prefix)) {
        if let Err(e) = handle_edit_cpp(
            ctx,
            content,
            author.clone(),
            old.clone(),
            original_message.clone(),
        )
        .await
        {
            let err = embeds::build_fail_embed(&author, &e.to_string());
            embeds::edit_message_embed(ctx, &mut old, err).await;
        }
    } else {
        let err = embeds::build_fail_embed(&author, "Invalid command for edit functionality!");
        embeds::edit_message_embed(ctx, &mut old, err).await;
    }
}

pub async fn handle_edit_cpp(
    ctx: &Context,
    content: String,
    author: User,
    mut old: Message,
    original_msg: Message,
) -> CommandResult {
    let embed =
        crate::commands::cpp::handle_request(ctx.clone(), content, author, &original_msg).await?;

    let compilation_successful = embed.0.get("color").unwrap() == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &old, compilation_successful).await?;

    embeds::edit_message_embed(ctx, &mut old, embed).await;
    Ok(())
}

pub async fn handle_edit_compile(
    ctx: &Context,
    content: String,
    author: User,
    mut old: Message,
    original_msg: Message,
) -> CommandResult {
    let embed =
        crate::commands::compile::handle_request(ctx.clone(), content, author, &original_msg)
            .await?;

    let compilation_successful = embed.0.get("color").unwrap() == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &old, compilation_successful).await?;

    embeds::edit_message_embed(ctx, &mut old, embed).await;
    Ok(())
}

pub async fn handle_edit_asm(
    ctx: &Context,
    content: String,
    author: User,
    mut old: Message,
    original_msg: Message,
) -> CommandResult {
    let emb =
        crate::commands::asm::handle_request(ctx.clone(), content, author, &original_msg).await?;

    let success = emb.0.get("color").unwrap() == COLOR_OKAY;
    embeds::edit_message_embed(ctx, &mut old, emb).await;

    send_completion_react(ctx, &old, success).await?;
    Ok(())
}

pub fn is_success_embed(embed: &CreateEmbed) -> bool {
    embed.0.get("color").unwrap() == COLOR_OKAY
}

pub async fn send_completion_react(
    ctx: &Context,
    msg: &Message,
    success: bool,
) -> Result<Reaction, serenity::Error> {
    let reaction;
    if success {
        {
            let data = ctx.data.read().await;
            let botinfo_lock = data.get::<ConfigCache>().unwrap();
            let botinfo = botinfo_lock.read().await;
            if let Some(success_id) = botinfo.get("SUCCESS_EMOJI_ID") {
                let success_name = botinfo
                    .get("SUCCESS_EMOJI_NAME")
                    .expect("Unable to find success emoji name")
                    .clone();
                reaction = discordhelpers::build_reaction(
                    success_id.parse::<u64>().unwrap(),
                    &success_name,
                );
            } else {
                reaction = ReactionType::Unicode(String::from("✅"));
            }
        }
    } else {
        reaction = ReactionType::Unicode(String::from("❌"));
    }
    return msg.react(&ctx.http, reaction).await;
}

// Certain compiler outputs use unicode control characters that
// make the user experience look nice (colors, etc). This ruins
// the look of the compiler messages in discord, so we strip them out
//
// Here we also limit the text to 1000 chars, this prevents discord from
// rejecting our embeds for being to long if someone decides to spam.
pub fn conform_external_str(input: &str, max_len: usize) -> String {
    let mut str = if let Ok(vec) = strip_ansi_escapes::strip(input) {
        String::from_utf8_lossy(&vec).to_string()
    } else {
        String::from(input)
    };

    // while we're at it, we'll escape ` characters with a
    // zero-width space to prevent our embed from getting
    // messed up later
    str = str.replace('`', "\u{200B}`");

    // Conform our string.
    if str.len() > MAX_OUTPUT_LEN {
        str.chars().take(max_len).collect()
    } else {
        str
    }
}

pub async fn manual_dispatch(http: Arc<Http>, id: u64, emb: CreateEmbed) {
    match serenity::model::id::ChannelId(id)
        .send_message(&http, |m| {
            m.embed(|mut e| {
                e.0 = emb.0;
                e
            })
        })
        .await
    {
        Ok(m) => m,
        Err(e) => return error!("Unable to dispatch manually: {}", e),
    };
}

pub async fn send_global_presence(shard_manager: &MutexGuard<'_, ShardManager>, sum: u64) {
    let server_count = {
        if sum < 10000 {
            sum.to_string()
        } else {
            format!("{:.1}k", sum / 1000)
        }
    };

    // update shard guild count & presence
    let presence_str = format!("in {} servers | ;invite", server_count);

    let runners = shard_manager.runners.lock().await;
    for (_, v) in runners.iter() {
        v.runner_tx
            .set_presence(Some(Activity::playing(&presence_str)), OnlineStatus::Online);
    }
}

pub async fn delete_bot_reacts(ctx: &Context, msg: &Message, react: ReactionType) -> CommandResult {
    let bot_id = {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        let id = botinfo.get("BOT_ID").unwrap();
        UserId::from(id.parse::<u64>().unwrap())
    };

    msg.channel_id
        .delete_reaction(&ctx.http, msg.id, Some(bot_id), react)
        .await?;
    Ok(())
}
