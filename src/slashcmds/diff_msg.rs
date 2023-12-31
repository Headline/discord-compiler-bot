use std::time::Duration;

use serenity::all::standard::CommandError;
use serenity::all::{CommandInteraction, CreateEmbed, EditInteractionResponse, User};
use serenity::{framework::standard::CommandResult, prelude::*};
use similar::ChangeTag;

use crate::utls::parser::{find_code_block, ParserResult};
use crate::{
    cache::{DiffCommandCache, DiffCommandEntry},
    utls::constants::COLOR_OKAY,
    utls::discordhelpers::interactions,
};

pub async fn diff_msg(ctx: &Context, msg: &CommandInteraction) -> CommandResult {
    let data = ctx.data.read().await;
    let diff_cache_lock = data.get::<DiffCommandCache>().unwrap();

    let is_first = {
        let mut diff_cache = diff_cache_lock.lock().await;
        if let Some(entry) = diff_cache.get_mut(&msg.user.id.get()) {
            entry.is_expired()
        } else {
            true
        }
    };

    if is_first {
        let (_, new_msg) = msg.data.resolved.messages.iter().next().unwrap();

        msg.create_response(&ctx.http, interactions::create_diff_select_response())
            .await
            .unwrap();
        {
            let content = get_code_block_or_content(&new_msg.content, &new_msg.author).await?;
            let mut diff_cache = diff_cache_lock.lock().await;
            diff_cache.insert(msg.user.id.get(), DiffCommandEntry::new(&content, msg));
        }
        let resp = msg.get_response(&ctx.http).await?;
        let button_resp = resp
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(30))
            .author_id(msg.user.id)
            .await;
        if let Some(interaction) = button_resp {
            interaction.defer(&ctx.http).await?;

            let cancel_embed = CreateEmbed::new()
                .color(COLOR_OKAY)
                .description("Interaction cancelled, you may safely dismiss this message");
            let edit = EditInteractionResponse::new()
                .embed(cancel_embed)
                .components(Vec::new());
            msg.edit_response(&ctx.http, edit).await?;

            let mut diff_cache = diff_cache_lock.lock().await;
            diff_cache.remove(&interaction.user.id.get());
        } else {
            // Button expired
            let expired_embed = CreateEmbed::new()
                .color(COLOR_OKAY)
                .description("Interaction expired, you may safely dismiss this message");
            let edit = EditInteractionResponse::new()
                .embed(expired_embed)
                .components(Vec::new());
            msg.edit_response(&ctx.http, edit).await?;
        }
        return Ok(());
    }

    // we can execute our diff now

    let (entry, first_interaction) = {
        let mut diff_cache = diff_cache_lock.lock().await;
        let entry = diff_cache.remove(&msg.user.id.get()).unwrap();
        (entry.content, entry.first_interaction)
    };

    if let Some((_, new_msg)) = msg.data.resolved.messages.iter().next() {
        let content = get_code_block_or_content(&new_msg.content, &new_msg.author).await?;
        let output = run_diff(&entry, &content);

        first_interaction
            .edit_response(&ctx.http, interactions::edit_to_dismiss_response())
            .await?;

        msg.create_response(&ctx.http, interactions::create_diff_response(&output))
            .await?;
    }
    Ok(())
}

pub fn run_diff(first: &str, second: &str) -> String {
    let diff = similar::TextDiff::from_lines(first, second);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

pub async fn get_code_block_or_content(
    input: &str,
    author: &User,
) -> std::result::Result<String, CommandError> {
    let mut fake_parse = ParserResult::default();
    if find_code_block(&mut fake_parse, input, author).await? {
        Ok(fake_parse.code)
    } else {
        // assume content is message content itself
        Ok(input.to_owned())
    }
}
