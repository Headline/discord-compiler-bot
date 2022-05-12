use std::time::Duration;

use serenity::{
    framework::standard::CommandResult,
    model::interactions::application_command::ApplicationCommandInteraction, prelude::*,
};

use crate::slashcmds::diff::run_diff;
use crate::{
    cache::{DiffCommandCache, DiffCommandEntry},
    slashcmds::diff::get_code_block_or_content,
    utls::constants::COLOR_OKAY,
    utls::discordhelpers::interactions,
};

pub async fn diff_msg(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {
    let data = ctx.data.read().await;
    let diff_cache_lock = data.get::<DiffCommandCache>().unwrap();

    let is_first = {
        let mut diff_cache = diff_cache_lock.lock().await;
        if let Some(entry) = diff_cache.get_mut(msg.user.id.as_u64()) {
            entry.is_expired()
        } else {
            true
        }
    };

    if is_first {
        let (_, new_msg) = msg.data.resolved.messages.iter().next().unwrap();

        msg.create_interaction_response(&ctx.http, |resp| {
            interactions::create_diff_select_response(resp)
        })
        .await
        .unwrap();
        {
            let mut diff_cache = diff_cache_lock.lock().await;
            let content = get_code_block_or_content(&new_msg.content, &new_msg.author).await?;
            diff_cache.insert(msg.user.id.0, DiffCommandEntry::new(&content, msg));
        }
        let resp = msg.get_interaction_response(&ctx.http).await?;
        let button_resp = resp
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(30))
            .author_id(msg.user.id.0)
            .await;
        if let Some(interaction) = button_resp {
            interaction.defer(&ctx.http).await?;
            let mut diff_cache = diff_cache_lock.lock().await;
            diff_cache.remove(interaction.user.id.as_u64());
            msg.edit_original_interaction_response(&ctx.http, |edit| {
                edit.set_embeds(Vec::new())
                    .embed(|emb| {
                        emb.color(COLOR_OKAY).description(
                            "Interaction cancelled, you may safely dismiss this message",
                        )
                    })
                    .components(|cmps| cmps.set_action_rows(Vec::new()))
            })
            .await?;
        } else {
            // Button expired
            msg.edit_original_interaction_response(&ctx.http, |edit| {
                edit.set_embeds(Vec::new())
                    .embed(|emb| {
                        emb.color(COLOR_OKAY).description(
                            "Interaction expired, you may safely dismiss this messsage",
                        )
                    })
                    .components(|cmps| cmps.set_action_rows(Vec::new()))
            })
            .await?;
        }
        return Ok(());
    }

    // we can execute our diff now

    let (entry, first_interaction) = {
        let mut diff_cache = diff_cache_lock.lock().await;
        let entry = diff_cache.remove(msg.user.id.as_u64()).unwrap();
        (entry.content, entry.first_interaction)
    };

    if let Some((_, new_msg)) = msg.data.resolved.messages.iter().next() {
        let content = get_code_block_or_content(&new_msg.content, &new_msg.author).await?;
        let output = run_diff(&entry, &content);

        first_interaction
            .edit_original_interaction_response(&ctx.http, interactions::edit_to_dismiss_response)
            .await?;

        msg.create_interaction_response(&ctx.http, |resp| {
            interactions::create_diff_response(resp, &output)
        })
        .await?;
    }
    Ok(())
}
