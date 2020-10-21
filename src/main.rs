mod utls;
mod apis;
mod commands;
mod cache;
mod events;
mod stats;

use serenity::{
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    http::Http,
    client::bridge::gateway::GatewayIntents
};

use std::{
    collections::HashSet,
    env,
    error::Error
};

use crate::apis::dbl::BotsListAPI;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

/** Command Registration **/
use crate::commands::{
    ping::*,
    botinfo::*,
    compile::*,
    languages::*,
    compilers::*,
    help::*,
    asm::ASM_COMMAND
};

#[group]
#[commands(botinfo,compile,languages,compilers,ping,help,asm)]
struct General;

/** Spawn bot **/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN")?;
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
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    info!("Registering owner(s): {}", owners.iter().map(|o| format!("{}", o.0)).collect::<Vec<String>>().join(", "));

    let prefix = env::var("BOT_PREFIX")?;
    let framework = StandardFramework::new()
        .configure(|c| c
        .owners(owners)
        .prefix(&prefix))
        .after(events::after)
        .group(&GENERAL_GROUP)
        .bucket("nospam", |b| b.delay(3).time_span(10).limit(3)).await
        .on_dispatch_error(events::dispatch_error);
    let mut client = serenity::Client::new(token)
        .framework(framework)
        .event_handler(events::Handler)
        .add_intent(GatewayIntents::GUILDS)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .add_intent(GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .await?;

    cache::fill(client.data.clone(), &prefix, &bot_id).await?;

    let dbl = BotsListAPI::new();
    if dbl.should_spawn() {
        dbl.spawn(client.cache_and_http.http.clone(), client.data.clone());
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
