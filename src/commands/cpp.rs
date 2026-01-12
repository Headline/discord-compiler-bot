use serenity::all::{CreateActionRow, CreateButton, CreateMessage};
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::{Message, ReactionType};
use serenity::model::user::User;

use crate::cache::{CompilerCache, ConfigCache, LinkAPICache, MessageCache, MessageCacheEntry};
use crate::cppeval::eval::CppEval;
use crate::managers::compilation::CompilationDetails;
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::embeds::{EmbedOptions, ToEmbed};
use crate::utls::parser::ParserResult;

#[command]
#[aliases("c++")]
#[bucket = "nospam"]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let result = handle_request(ctx, &msg.content, &msg.author, msg).await?;

    let data = ctx.data.read().await;

    // Build message with optional godbolt link button
    let mut new_msg = CreateMessage::new().embed(result.embed);
    if let Some(b64) = &result.details.godbolt_base64 {
        if let Some(link_cache) = data.get::<LinkAPICache>() {
            let long_url = format!("https://godbolt.org/clientstate/{}", b64);
            let link_cache_lock = link_cache.read().await;
            if let Some(short_url) = link_cache_lock.get_link(long_url).await {
                let btn = CreateButton::new_link(short_url).label("View on godbolt.org");
                new_msg = new_msg.components(vec![CreateActionRow::Buttons(vec![btn])]);
            }
        }
    }

    let sent = msg.channel_id.send_message(&ctx.http, new_msg).await?;

    // Cache for edit tracking
    let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
    message_cache.insert(msg.id.get(), MessageCacheEntry::new(sent, msg.clone()));

    Ok(())
}

/// Result of handle_request containing everything needed to display and track a compilation
pub struct HandleRequestResult {
    pub embed: CreateEmbed,
    pub details: CompilationDetails,
}

/// Parse C++ expression, wrap it for evaluation, compile, and return result.
pub async fn handle_request(
    ctx: &Context,
    content: &str,
    author: &User,
    msg: &Message,
) -> Result<HandleRequestResult, CommandError> {
    let data = ctx.data.read().await;

    // Get loading reaction
    let loading_reaction = get_loading_reaction(&data).await?;

    // Parse the C++ expression
    let start = content
        .find(' ')
        .ok_or_else(|| CommandError::from("Invalid usage. View `;help cpp`"))?;
    let expression = content.split_at(start).1;

    // Evaluate and wrap the expression
    let mut eval = CppEval::new(expression);
    let wrapped_code = eval.evaluate()?;

    // Show loading indicator
    if msg
        .react(&ctx.http, loading_reaction.clone())
        .await
        .is_err()
    {
        return Err(CommandError::from(
            "Unable to react to message. Am I missing permissions to react or use external emoji?",
        ));
    }

    // Build a fake parse result for the compilation
    let parse_result = ParserResult {
        url: String::new(),
        stdin: String::new(),
        target: "gsnapshot".to_string(),
        code: wrapped_code,
        options: vec![String::from("-O3"), String::from("-std=gnu++26")],
        args: vec![],
    };

    // Compile using Godbolt (raw response needed for custom embed building)
    let compilation_manager = data.get::<CompilerCache>().unwrap().read().await;
    let result = compilation_manager.compile_godbolt_raw(&parse_result).await;

    // Remove loading indicator
    discordhelpers::delete_bot_reacts(ctx, msg, loading_reaction.clone()).await?;

    let (details, response) = result?;

    // Build embed from response
    let embed_options = EmbedOptions::new(false, details.clone());
    let embed = response.to_embed(author, &embed_options);

    Ok(HandleRequestResult { embed, details })
}

/// Get the configured loading reaction or default hourglass
async fn get_loading_reaction(
    data: &tokio::sync::RwLockReadGuard<'_, serenity::prelude::TypeMap>,
) -> Result<ReactionType, CommandError> {
    let config = data.get::<ConfigCache>().unwrap().read().await;

    if let Some(loading_id) = config.get("LOADING_EMOJI_ID") {
        let loading_name = config
            .get("LOADING_EMOJI_NAME")
            .expect("LOADING_EMOJI_NAME must be set if LOADING_EMOJI_ID is set")
            .clone();
        Ok(discordhelpers::build_reaction(
            loading_id.parse::<u64>()?,
            &loading_name,
        ))
    } else {
        Ok(ReactionType::Unicode(String::from("⏳")))
    }
}
