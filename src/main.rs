mod utls;
mod apis;
mod commands;
mod cache;
mod events;
mod stats;

use std::{
    collections::HashSet,
    env,
    error::Error
};

use serenity::{
    framework::{
        StandardFramework,
        standard::macros::{group, hook},
    },
    http::Http,
};

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
    asm::hide::*
};
use crate::apis::dbl::BotsListAPI;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::framework::standard::CommandResult;
use serenity::client::bridge::gateway::GatewayIntents;

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
        .after(after)
        .group(&GENERAL_GROUP);
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

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    use crate::utls::discordhelpers::DiscordHelpers;
    use crate::cache::{Stats};
    if let Err(e) = command_result {
        let emb = DiscordHelpers::build_fail_embed( &msg.author, &format!("{}", e));
        let mut emb_msg = DiscordHelpers::embed_message(emb);
        if let Err(_) = msg.channel_id.send_message(&ctx.http, |_| &mut emb_msg).await {
            // missing permissions, just ignore...
        }
    }

    let data = ctx.data.read().await;
    let stats = data.get::<Stats>().unwrap().lock().await;
    if stats.should_track() {
        stats.command_executed(command_name).await;
    }
}