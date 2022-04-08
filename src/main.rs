#![type_length_limit="1146253"]

mod apis;
mod cache;
mod commands;
mod events;
mod stats;
mod utls;
mod cppeval;
mod managers;

use std::{collections::HashSet, env, error::Error};

use serenity::{
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    client::bridge::gateway::GatewayIntents,
    client::Context,
    framework::standard::{Args, CommandResult},
    model::channel::Message,
    framework::standard::{macros::command},
    model::interactions::message_component::ButtonStyle
};

use crate::{
    apis::dbl::BotsListApi,
    utls::constants::COLOR_WARN,
    cache::ConfigCache
};

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

#[command]
#[aliases("cpp", "asm", "help", "botinfo", "format", "invite", "ping")]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let botinfo = data.get::<ConfigCache>().unwrap().read().await;
    let github_link = botinfo.get("GITHUB_LINK").unwrap();

    msg.channel_id.send_message(&ctx.http, |new| {
        new.reference_message(msg).embed(|emb| {
            emb.color(COLOR_WARN)
                .title("A message from the author")
                .description("Hello,\n\
                As you may already know, Discord is limiting the ability of large bots to view message content. \
                This means that we will no longer be able to respond to requests as we used to, our entire command \
                handling system has been rewritten, and we no longer operate using the prefix ';'. \
                \n\n\
                By August 31st, all requests must now either originate from slash commands (see /help) or through what's called \
                \"Message Commands\". You should already be familiar with slash commands, but message commands \
                can be accessed by right clicking a message and hovering over \"Apps\". This is where you will find \
                our Compile, Format, and Assembly commands from now on. \
                \n\n\
                Since this format is new for all of us I'm asking our users to bear with us these coming weeks \
                while we work out any new issues introduced. If you'd like to report an issue or suggestion you may do so \
                by selecting one of the buttons below. \
                \n\n\
                Thank you for your patience while we migrate to this new system \
                \n\
                -Headline")
        })
        .allowed_mentions(|mnts| {
            mnts.replied_user(false)
        })
            .components(|cmps| {
                cmps.create_action_row(|row| {
                    row.create_button(|btn| {
                        btn.label("Support Server")
                            .url("https://discord.com/invite/ExraTaJ")
                            .style(ButtonStyle::Link)
                    })
                    .create_button(|btn| {
                        btn.label("GitHub Page")
                            .url(github_link)
                            .style(ButtonStyle::Link)
                    })
                })
            })
    }).await?;
    Ok(())
}

#[group]
#[commands(compile)]
struct General;

/** Spawn bot **/
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenv::dotenv() {
        error!("Unable to find .env configuration file: {}", e);
    }

    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN")?;
    let application_id = env::var("APPLICATION_ID");
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
            let id = env::var("BOT_ID").expect("Unable to find BOT_ID environment variable");
            let bot_id = id.parse::<u64>().expect("Invalid bot id");
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

    let prefix = env::var("BOT_PREFIX")?;
    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(&prefix))
        .before(events::before)
        .after(events::after)
        .group(&GENERAL_GROUP)
        .bucket("nospam", |b| b.delay(3).time_span(10).limit(3))
        .await
        .on_dispatch_error(events::dispatch_error);

    let mut cb = serenity::Client::builder(token)
        .framework(framework)
        .event_handler(events::Handler)
        .intents(GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS);

    if let Ok(app_id) = application_id {
        cb = cb.application_id(app_id.parse::<u64>()?);
    }

    let mut client = cb.await?;

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
