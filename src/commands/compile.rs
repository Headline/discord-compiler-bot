use serenity::framework::standard::{macros::command, Args, CommandResult};

use crate::cache::{MessageCache};
use crate::utls::{parser, discordhelpers};
use crate::utls::constants::COLOR_OKAY;
use crate::utls::discordhelpers::{embeds, is_success_embed};

use std::env;
use tokio::sync::RwLockReadGuard;

use serenity::framework::standard::CommandError;
use serenity::builder::{CreateEmbed};
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::user::User;

use crate::cache::{ConfigCache, StatsManagerCache, CompilerCache};
use crate::managers::compilation::CompilationManager;

#[command]
#[bucket = "nospam"]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;

    // Handle wandbox request logic
    let embed = handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;

    // Send our final embed
    let mut message = embeds::embed_message(embed);
    let compilation_embed = msg
        .channel_id
        .send_message(&ctx.http, |_| &mut message)
        .await?;

    // Success/fail react
    let compilation_successful = compilation_embed.embeds[0].colour.0 == COLOR_OKAY;
    discordhelpers::send_completion_react(ctx, &compilation_embed, compilation_successful).await?;

    let mut delete_cache = data_read.get::<MessageCache>().unwrap().lock().await;
    delete_cache.insert(msg.id.0, compilation_embed);
    debug!("Command executed");
    Ok(())
}

pub async fn handle_request(ctx : Context, mut content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
    let data_read = ctx.data.read().await;
    let loading_react =  {
        let reaction;
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        if let Some(loading_id) = botinfo.get("LOADING_EMOJI_ID") {
            let loading_name = botinfo.get("LOADING_EMOJI_ID").expect("Unable to find loading emoji name").clone();
            reaction = discordhelpers::build_reaction(loading_id.parse::<u64>()?, &loading_name);
        }
        else {
            reaction = ReactionType::Unicode(String::from("⏳"));
        }

        reaction
    };

    // Try to load in an attachment
    let attached = parser::get_message_attachment(&msg.attachments).await?;
    if !attached.is_empty() {
        content.push_str(&format!("\n```\n{}\n```\n", attached));
    }

    // parse user input
    let compilation_manager = data_read.get::<CompilerCache>().unwrap();
    let parse_result = parser::get_components(&content, &author, &compilation_manager, &msg.referenced_message).await?;

    // send out loading emote
    let reaction = match msg.react(&ctx.http, loading_react).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    // dispatch our req
    let compilation_manager_lock : RwLockReadGuard<CompilationManager> = compilation_manager.read().await;
    let awd = compilation_manager_lock.compile(&parse_result, &author).await;
    let result = match awd {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // remove our loading emote
    if msg.delete_reaction_emoji(&ctx.http, reaction.emoji.clone()).await
        .is_err()
    {
        return Err(CommandError::from(
            "Unable to remove reactions!\nAm I missing permission to manage messages?",
        ));
    }

    let stats = data_read.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.compilation(&result.0, !is_success_embed(&result.1)).await;
    }

    let mut guild = String::from("<unknown>");
    if let Some(g) = msg.guild_id {
        guild = g.to_string()
    }
    if let Ok(log) = env::var("COMPILE_LOG") {
        if let Ok(id) = log.parse::<u64>() {
            let emb = embeds::build_complog_embed(
                is_success_embed(&result.1),
                &parse_result.code,
                &parse_result.target,
                &author.tag(),
                author.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }

    Ok(result.1)
}