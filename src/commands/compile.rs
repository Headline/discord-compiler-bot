use std::fmt::Write as _;

use serenity::all::{CreateActionRow, CreateButton, CreateMessage};
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::{Message, ReactionType};
use serenity::model::user::User;

use crate::cache::{CompilerCache, ConfigCache, LinkAPICache, MessageCache, MessageCacheEntry};
use crate::managers::compilation::{CompilationDetails, CompilationResult};
use crate::utls::discordhelpers::embeds;
use crate::utls::{discordhelpers, parser};

#[command]
#[bucket = "nospam"]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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

    // React with success/fail indicator
    discordhelpers::send_completion_react(ctx, &sent, result.details.success).await?;

    // Cache for edit tracking
    let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
    message_cache.insert(msg.id.get(), MessageCacheEntry::new(sent, msg.clone()));

    debug!("Command executed");
    Ok(())
}

/// Result of handle_request containing everything needed to display and track a compilation
pub struct HandleRequestResult {
    pub embed: CreateEmbed,
    pub details: CompilationDetails,
}

/// Parse message, compile code, and return result ready for display.
pub async fn handle_request(
    ctx: &Context,
    content: &str,
    author: &User,
    msg: &Message,
) -> Result<HandleRequestResult, CommandError> {
    // Extract needed data and release lock immediately
    let (loading_reaction, compilation_manager) = {
        let data = ctx.data.read().await;
        let reaction = get_loading_reaction(&data).await?;
        let comp_mgr = data.get::<CompilerCache>().unwrap().clone();
        (reaction, comp_mgr)
    }; // TypeMap lock released here

    // Handle file attachments (may do HTTP request)
    let content = append_attachment_code(content, &msg.attachments).await?;

    // Parse the compilation request
    let parse_result = parser::get_components(
        &content,
        author,
        Some(&compilation_manager),
        &msg.referenced_message,
        false,
    )
    .await?;

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

    // Compile the code - this is the slow part, no locks held
    let result = {
        let compilation_manager_lock = compilation_manager.read().await;
        compilation_manager_lock
            .compile(&parse_result, author)
            .await
    };

    // Remove loading indicator
    let _ = discordhelpers::delete_bot_reacts(ctx, msg, loading_reaction).await;

    // Handle compilation errors
    let CompilationResult { details, embed } = result?;

    // Log compilation if configured
    log_compilation(ctx, msg, &parse_result, details.success).await;

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

/// Append code from message attachments to content
async fn append_attachment_code(
    content: &str,
    attachments: &[serenity::model::channel::Attachment],
) -> Result<String, CommandError> {
    let (code, ext) = parser::get_message_attachment(attachments).await?;
    if code.is_empty() {
        return Ok(content.to_string());
    }

    let mut result = content.to_string();
    writeln!(&mut result, "\n```{}\n{}\n```\n", ext, code).unwrap();
    Ok(result)
}

/// Log compilation to the configured channel if enabled
async fn log_compilation(
    ctx: &Context,
    msg: &Message,
    parse_result: &parser::ParserResult,
    success: bool,
) {
    let data = ctx.data.read().await;
    let config = data.get::<ConfigCache>().unwrap().read().await;

    if let Some(log_channel) = config.get("COMPILE_LOG") {
        if let Ok(channel_id) = log_channel.parse::<u64>() {
            let guild = msg
                .guild_id
                .map(|g| g.to_string())
                .unwrap_or_else(|| "<<DM>>".to_string());

            let embed = embeds::build_complog_embed(
                success,
                &parse_result.code,
                &parse_result.target,
                &msg.author.name,
                msg.author.id,
                &guild,
            );
            discordhelpers::manual_dispatch(ctx.http.clone(), channel_id, embed).await;
        }
    }
}
