use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::BotInfo;
use std::env;

#[command]
pub async fn botinfo(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let invite = env::var("INVITE_LINK").expect("Expected invite link envvar");
    let topgg = env::var("DISCORDBOTS_LINK").expect("Expected top.gg link envvar");
    let github = env::var("GITHUB_LINK").expect("Expected github link envvar");
    let stats = env::var("STATS_LINK").expect("Expected stats link envvar");

    let avatar = {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<BotInfo>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        botinfo.get("BOT_AVATAR").unwrap().clone()
    };

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m: &mut CreateMessage| {
            m.embed(|e: &mut CreateEmbed| {
                e.title("Compiler Bot");

                let fmt = format!(
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

                e.description(fmt);
                e.thumbnail(avatar);
                e.color(COLOR_OKAY);

                let hash = get_github_build(false);
                let short = get_github_build(true);
                let str = format!(
                    "Built from commit [{}]({}{}{})",
                    short, github, "/commit/", hash
                );
                e.fields(vec![
                    ("Language", "Rust 2018", false),
                    ("Software Version", env!("CARGO_PKG_VERSION"), false),
                    ("Author", env!("CARGO_PKG_AUTHORS"), false),
                    ("Build Information", str.as_str(), false),
                ]);
                e
            });
            m
        })
        .await;

    if let Err(why) = msg {
        warn!("Error sending embed: {:?}", why);
    }

    debug!("Command executed");
    Ok(())
}

use crate::utls::constants::COLOR_OKAY;
use std::process::Command;

pub fn get_github_build(short: bool) -> String {
    // note: add error checking yourself.
    let mut args = vec!["rev-parse"];
    if short {
        args.push("--short");
    }

    args.push("HEAD");
    let output = Command::new("git").args(&args).output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}
