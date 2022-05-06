#![type_length_limit = "1146253"]

mod apis;
mod boilerplate;
mod cache;
mod commands;
mod cppeval;
mod events;
mod managers;
mod slashcmds;
mod stats;
mod tests;
mod utls;

use serenity::framework::{standard::macros::group, StandardFramework};

use serenity::http::Http;
use serenity::prelude::GatewayIntents;
use std::collections::HashSet;
use std::{env, error::Error};

use crate::apis::dbl::BotsListApi;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

/** Command Registration **/
use crate::commands::{
    asm::*, block::*, botinfo::*, compile::*, compilers::*, cpp::*, format::*, formats::*, help::*,
    invite::*, languages::*, ping::*, unblock::*,
};

#[group]
#[commands(
    botinfo, compile, languages, compilers, ping, help, asm, block, unblock, invite, cpp, formats,
    format
)]
struct General;

/** Spawn bot **/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenv::dotenv() {
        error!("Unable to find .env configuration file: {}", e);
    }

    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN").expect("Expected bot token in .env file");

    let http = Http::new(&token);
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();

            owners.insert(info.owner.id);

            if let Some(team) = info.team {
                for member in &team.members {
                    owners.insert(member.user.id);
                }
            }

            (owners, info.id)
        }
        Err(why) => {
            warn!("Could not access application info: {:?}", why);
            warn!("Trying environment variable for bot id...");
            let id = env::var("BOT_ID").expect("Unable to find BOT_ID environment variable");
            let bot_id = id.parse::<u64>().expect("Invalid bot id");
            (HashSet::new(), serenity::model::id::ApplicationId(bot_id))
        }
    };

    info!(
        "Registering owner(s): {}",
        owners
            .iter()
            .map(|o| format!("{}", o.0))
            .collect::<Vec<String>>()
            .join(", ")
    );

    if cfg!(debug_assertions) {
        warn!("Running bot in DEBUG mode...");
    }

    let prefix = env::var("BOT_PREFIX").expect("Expected bot prefix in .env file");
    let app_id = env::var("APPLICATION_ID").expect("Expected application id in .env file");
    let framework = StandardFramework::new()
        .before(events::before)
        .after(events::after)
        .configure(|c| c.owners(owners).prefix(&prefix))
        .group(&GENERAL_GROUP)
        .bucket("nospam", |b| b.delay(3).time_span(10).limit(3))
        .await
        .on_dispatch_error(events::dispatch_error);

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGES;
    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .event_handler(events::Handler)
        .application_id(app_id.parse::<u64>().unwrap())
        .await?;

    cache::fill(
        client.data.clone(),
        &prefix,
        bot_id.0,
        client.shard_manager.clone(),
    )
    .await?;

    let dbl = BotsListApi::new();
    if dbl.should_spawn() {
        dbl.spawn(client.cache_and_http.http.clone(), client.data.clone());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
