use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::env;

use crate::cache::ConfigCache;
use crate::utls::constants::COLOR_OKAY;

#[command]
pub async fn botinfo(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let invite = env::var("INVITE_LINK").expect("Expected invite link envvar");
    let topgg = env::var("DISCORDBOTS_LINK").expect("Expected top.gg link envvar");
    let github = env::var("GITHUB_LINK").expect("Expected github link envvar");
    let stats = env::var("STATS_LINK").expect("Expected stats link envvar");

    let hash_short;
    let hash_long;
    let avatar = {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected ConfigCache in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        hash_short = botinfo.get("GIT_HASH_SHORT").unwrap().clone();
        hash_long = botinfo.get("GIT_HASH_LONG").unwrap().clone();
        botinfo.get("BOT_AVATAR").unwrap().clone()
    };

    let build_info = format!(
        "Built from commit [{}]({}{}{})",
        hash_short, github, "/commit/", hash_long
    );

    let body_txt = format!(
        "{}\n
                {}
                [Invitation link]({})
                [Vote for us!]({})
                [GitHub Repository]({})
                [Statistics Tracker]({})
                {}",
        env!("CARGO_PKG_DESCRIPTION"),
        "==========================",
        invite,
        topgg,
        github,
        stats,
        "=========================="
    );

    let emb = CreateEmbed::new()
        .title("Compiler Bot")
        .description(body_txt)
        .thumbnail(avatar)
        .color(COLOR_OKAY)
        .fields(vec![
            ("Language", "Rust 2021", false),
            ("Software Version", env!("CARGO_PKG_VERSION"), false),
            ("Author", env!("CARGO_PKG_AUTHORS"), false),
            ("Build Information", build_info.as_str(), false),
        ]);

    let new_msg = CreateMessage::new().embed(emb);
    let msg = msg.channel_id.send_message(&ctx.http, new_msg).await;

    if let Err(why) = msg {
        warn!("Error sending embed: {:?}", why);
    }

    debug!("Command executed");
    Ok(())
}
