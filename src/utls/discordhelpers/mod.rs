pub mod embeds;
pub mod interactions;

use std::{
    str,
    sync::Arc
};

use serenity::{
    builder::{CreateEmbed},
    http::Http,
    model::prelude::*,
};

use crate::utls::constants::*;
use tokio::sync::{MutexGuard};
use serenity::client::bridge::gateway::{ShardManager};

// Certain compiler outputs use unicode control characters that
// make the user experience look nice (colors, etc). This ruins
// the look of the compiler messages in discord, so we strip them out
//
// Here we also limit the text to 1000 chars, this prevents discord from
// rejecting our embeds for being to long if someone decides to spam.
pub fn conform_external_str(input: &str, max_len : usize) -> String {
    let mut str: String;
    if let Ok(vec) = strip_ansi_escapes::strip(input) {
        str = String::from_utf8_lossy(&vec).to_string();
    } else {
        str = String::from(input);
    }

    // while we're at it, we'll escape ` characters with a
    // zero-width space to prevent our embed from getting
    // messed up later
    str = str.replace("`", "\u{200B}`");

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

pub async fn send_global_presence(shard_manager : &MutexGuard<'_, ShardManager>, sum : u64) {
    // update shard guild count & presence
    let presence_str = format!("in {} servers | ;invite", sum);

    let runners = shard_manager.runners.lock().await;
    for (_, v) in runners.iter() {
        v.runner_tx.set_presence(Some(Activity::playing(&presence_str)), OnlineStatus::Online);
    }
}