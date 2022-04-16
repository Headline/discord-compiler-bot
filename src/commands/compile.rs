use serenity::framework::standard::{macros::command, Args, CommandResult};

use crate::cache::{MessageCache, MessageCacheEntry};
use crate::utls::{parser, discordhelpers};
use crate::utls::constants::COLOR_OKAY;
use crate::utls::discordhelpers::{embeds, is_success_embed};

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
    delete_cache.insert(msg.id.0, MessageCacheEntry::new(compilation_embed, msg.clone()));
    debug!("Command executed");
    Ok(())
}

pub async fn handle_request(ctx : Context, mut content : String, author : User, msg : &Message) -> Result<CreateEmbed, CommandError> {
    let data_read = ctx.data.read().await;
    let loading_reaction = {
        let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
        let botinfo = botinfo_lock.read().await;
        if let Some(loading_id) = botinfo.get("LOADING_EMOJI_ID") {
            let loading_name = botinfo.get("LOADING_EMOJI_NAME").expect("Unable to find loading emoji name").clone();
            discordhelpers::build_reaction(loading_id.parse::<u64>()?, &loading_name)
        }
        else {
            ReactionType::Unicode(String::from("⏳"))
        }
    };

    // Try to load in an attachment
    let (code, ext) = parser::get_message_attachment(&msg.attachments).await?;
    if !code.is_empty() {
        content.push_str(&format!("\n```{}\n{}\n```\n", ext, code));
    }

    // parse user input
    let compilation_manager = data_read.get::<CompilerCache>().unwrap();
    let parse_result = parser::get_components(&content, &author, Some(&compilation_manager), &msg.referenced_message).await?;

    // send out loading emote
    if let Err(_) = msg.react(&ctx.http, loading_reaction.clone()).await {
        return Err(CommandError::from("Unable to react to message, am I missing permissions to react or use external emoji?\n{}"));
    }

    // dispatch our req
    let compilation_manager_lock : RwLockReadGuard<CompilationManager> = compilation_manager.read().await;
    let awd = compilation_manager_lock.compile(&parse_result, &author).await;
    let result = match awd {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction.clone()).await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };
    
    // remove our loading emote
    let _ = discordhelpers::delete_bot_reacts(&ctx, &msg, loading_reaction).await;

    let is_success = is_success_embed(&result.1);
    let stats = data_read.get::<StatsManagerCache>().unwrap().lock().await;
    if stats.should_track() {
        stats.compilation(&result.0, !is_success).await;
    }

    let data = ctx.data.read().await;
    let config = data.get::<ConfigCache>().unwrap();
    let config_lock = config.read().await;

    if let Some(log) = config_lock.get("COMPILE_LOG") {
        if let Ok(id) = log.parse::<u64>() {
            let guild = if msg.guild_id.is_some() {msg.guild_id.unwrap().0.to_string()} else {"<<unknown>>".to_owned()};
            let emb = embeds::build_complog_embed(
                is_success,
                &parse_result.code,
                &parse_result.target,
                &msg.author.tag(),
                msg.author.id.0,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), id, emb).await;
        }
    }

    Ok(result.1)
}