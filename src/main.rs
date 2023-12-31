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

use serenity::all::standard::{BucketBuilder, Configuration};
use serenity::http::Http;
use serenity::model::id::ApplicationId;
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
    insights::*, invite::*, languages::*, ping::*, unblock::*,
};
use crate::utls::discordhelpers::embeds::panic_embed;
use crate::utls::discordhelpers::manual_dispatch;

#[group]
#[commands(
    botinfo, compile, languages, compilers, ping, help, asm, block, unblock, invite, cpp, formats,
    format, insights
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

            owners.insert(info.owner.expect("Expected owner ID to be registered!").id);

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
            (HashSet::new(), ApplicationId::new(bot_id))
        }
    };

    info!(
        "Registering owner(s): {}",
        owners
            .iter()
            .map(|o| format!("{}", o))
            .collect::<Vec<String>>()
            .join(", ")
    );

    if cfg!(debug_assertions) {
        warn!("Running bot in DEBUG mode...");
    }

    let prefix = env::var("BOT_PREFIX").expect("Expected bot prefix in .env file");
    let app_id_str = env::var("APPLICATION_ID").expect("Expected application id in .env file");
    let application_id = ApplicationId::new(app_id_str.parse::<u64>().unwrap());

    let configuration = Configuration::new().owners(owners).prefix(&prefix);
    let mut framework = StandardFramework::new()
        .group(&GENERAL_GROUP)
        .before(events::before)
        .after(events::after)
        .on_dispatch_error(events::dispatch_error);

    framework.configure(configuration);
    framework = framework
        .bucket(
            "nospam",
            BucketBuilder::new_global().delay(3).time_span(10).limit(3),
        )
        .await;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MESSAGES;

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .event_handler(events::Handler)
        .application_id(application_id)
        .await?;

    cache::fill(
        client.data.clone(),
        &prefix,
        bot_id,
        client.shard_manager.clone(),
    )
    .await?;
    if let Ok(plog) = env::var("PANIC_LOG") {
        let default_panic = std::panic::take_hook();
        let http = client.http.clone();

        std::panic::set_hook(Box::new(move |info| {
            let http = http.clone();
            if let Ok(plog_parse) = plog.parse::<u64>() {
                let panic_str = info.to_string();
                tokio::spawn({
                    async move { manual_dispatch(http, plog_parse, panic_embed(panic_str)).await }
                });
            } else {
                warn!("Unable to parse channel id64 from PANIC_LOG, is it valid?");
            }
            default_panic(info);
        }));
    }

    let dbl = BotsListApi::new();
    if dbl.should_spawn() {
        dbl.spawn(client.http.clone(), client.data.clone());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
