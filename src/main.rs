#![type_length_limit="1146253"]

mod apis;
mod cache;
mod commands;
mod events;
mod stats;
mod utls;
mod cppeval;
mod managers;
mod tests;
mod slashcmds;

use serenity::{
    client::bridge::gateway::GatewayIntents,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
};

use std::{collections::HashSet, env, error::Error};

use crate::apis::dbl::BotsListApi;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

/** Command Registration **/
use crate::commands::{
    asm::*, botinfo::*, compile::*, compilers::*,
    help::*, languages::*, ping::*, block::*, unblock::*,
    invite::*, cpp::*, format::*, formats::*
};

#[group]
#[commands(botinfo, compile, languages, compilers, ping, help, asm, block, unblock, invite, cpp, formats, format)]
struct General;

/** Spawn bot **/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenv::dotenv() {
        error!("Unable to find .env configuration file: {}", e);
    }

    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN")
        .expect("Expected bot token in .env file");
    let http = Http::new_with_token(&token);
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
            let id = env::var("BOT_ID")
                .expect("Unable to find BOT_ID environment variable");
            let bot_id = id.parse::<u64>()
                .expect("Invalid bot id");
            (HashSet::new(), serenity::model::id::UserId(bot_id))
        },
    };

    info!(
        "Registering owner(s): {}",
        owners
            .iter()
            .map(|o| format!("{}", o.0))
            .collect::<Vec<String>>()
            .join(", ")
    );

    let prefix = env::var("BOT_PREFIX")
        .expect("Expected bot prefix in .env file");
    let app_id = env::var("APPLICATION_ID")
        .expect("Expected application id in .env file");
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(&prefix))
        .before(events::before)
        .after(events::after)
        .group(&GENERAL_GROUP)
        .bucket("nospam", |b| b.delay(3).time_span(10).limit(3))
        .await
        .on_dispatch_error(events::dispatch_error);
    let mut client = serenity::Client::builder(token)
        .framework(framework)
        .event_handler(events::Handler)
        .intents(GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .application_id(app_id.parse::<u64>().unwrap())
        .await?;

    cache::fill(client.data.clone(), &prefix, &bot_id, client.shard_manager.clone()).await?;

    let dbl = BotsListApi::new();
    if dbl.should_spawn() {
        dbl.spawn(client.cache_and_http.http.clone(), client.data.clone());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
