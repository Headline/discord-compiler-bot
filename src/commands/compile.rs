use std::fmt::Write as _;
use std::time::Duration;

use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse, CreateMessage,
    EditMessage,
};
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::{Message, ReactionType};
use serenity::model::user::User;

use crate::cache::{CompilerCache, ConfigCache, LinkAPICache, MessageCache, MessageCacheEntry};
use crate::managers::compilation::{CompilationDetails, CompilationResult};
use crate::utls::discordhelpers::embeds;
use crate::utls::parser::ParserResult;
use crate::utls::{discordhelpers, parser};

const EXECUTE_BUTTON_TIMEOUT: Duration = Duration::from_secs(30);

#[command]
#[bucket = "nospam"]
pub async fn compile(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let result = handle_request(ctx, &msg.content, &msg.author, msg, false).await?;

    let link_button = build_link_button(ctx, &result.details).await;
    let offer_execute = result.details.success && !result.details.executed;

    let mut buttons = link_button.clone();
    if offer_execute {
        buttons.push(
            CreateButton::new(format!("execute:{}", msg.id.get()))
                .label("Execute")
                .style(ButtonStyle::Primary),
        );
    }

    let mut new_msg = discordhelpers::reply_to(msg, CreateMessage::new().embed(result.embed));
    if !buttons.is_empty() {
        new_msg = new_msg.components(vec![CreateActionRow::Buttons(buttons)]);
    }

    let sent = msg.channel_id.send_message(&ctx.http, new_msg).await?;

    // React with success/fail indicator
    discordhelpers::send_completion_react(ctx, &sent, result.details.success).await?;

    // Cache for edit tracking
    {
        let data = ctx.data.read().await;
        let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
        let mut entry = MessageCacheEntry::new(sent.clone(), msg.clone());
        entry.executed = result.details.executed;
        message_cache.insert(msg.id.get(), entry);
    }

    if offer_execute {
        await_execute_button(ctx, msg, sent, result.parse_result, link_button).await?;
    }

    debug!("Command executed");
    Ok(())
}

/// Build the "View on godbolt.org" link button if a shortened link is available
pub async fn build_link_button(ctx: &Context, details: &CompilationDetails) -> Vec<CreateButton> {
    let mut buttons = Vec::new();
    if let Some(b64) = &details.godbolt_base64 {
        let data = ctx.data.read().await;
        if let Some(link_cache) = data.get::<LinkAPICache>() {
            let long_url = format!("https://godbolt.org/clientstate/{}", b64);
            let link_cache_lock = link_cache.read().await;
            if let Some(short_url) = link_cache_lock.get_link(long_url).await {
                buttons.push(CreateButton::new_link(short_url).label("View on godbolt.org"));
            }
        }
    }
    buttons
}

/// Wait for the requester to press Execute, running the code if pressed.
/// The button is removed once we stop waiting.
async fn await_execute_button(
    ctx: &Context,
    request_msg: &Message,
    mut sent: Message,
    parse_result: ParserResult,
    link_button: Vec<CreateButton>,
) -> CommandResult {
    let interaction = sent
        .await_component_interaction(&ctx.shard)
        .author_id(request_msg.author.id)
        .timeout(EXECUTE_BUTTON_TIMEOUT)
        .await;

    let components = if link_button.is_empty() {
        Vec::new()
    } else {
        vec![CreateActionRow::Buttons(link_button)]
    };

    let Some(mci) = interaction else {
        sent.edit(&ctx.http, EditMessage::new().components(components))
            .await?;
        return Ok(());
    };

    mci.create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
        .await?;

    // mark as executed so edits re-execute rather than compile
    let compilation_manager = {
        let data = ctx.data.read().await;
        {
            let mut message_cache = data.get::<MessageCache>().unwrap().lock().await;
            if let Some(entry) = message_cache.get_mut(&request_msg.id.get()) {
                entry.executed = true;
            }
        }
        data.get::<CompilerCache>().unwrap().clone()
    };
    let result = {
        let compilation_manager_lock = compilation_manager.read().await;
        compilation_manager_lock
            .execute(&parse_result, &request_msg.author)
            .await
    };

    match result {
        Ok(CompilationResult { details, embed }) => {
            if let Ok(updated) = sent.channel_id.message(&ctx.http, sent.id).await {
                for reaction in &updated.reactions {
                    if reaction.me {
                        let _ = discordhelpers::delete_bot_reacts(
                            ctx,
                            &updated,
                            reaction.reaction_type.clone(),
                        )
                        .await;
                    }
                }
            }
            let _ = discordhelpers::send_completion_react(ctx, &sent, details.success).await;

            sent.edit(
                &ctx.http,
                EditMessage::new().embed(embed).components(components),
            )
            .await?;
        }
        Err(e) => {
            let embed = embeds::build_fail_embed(&request_msg.author, &e.to_string());
            sent.edit(
                &ctx.http,
                EditMessage::new().embed(embed).components(components),
            )
            .await?;
        }
    }

    Ok(())
}

/// Result of handle_request containing everything needed to display and track a compilation
pub struct HandleRequestResult {
    pub embed: CreateEmbed,
    pub details: CompilationDetails,
    pub parse_result: ParserResult,
}

/// Parse message, compile code (running it if `execute` is set), and return
/// result ready for display.
pub async fn handle_request(
    ctx: &Context,
    content: &str,
    author: &User,
    msg: &Message,
    execute: bool,
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
        if execute {
            compilation_manager_lock
                .execute(&parse_result, author)
                .await
        } else {
            compilation_manager_lock
                .compile(&parse_result, author)
                .await
        }
    };

    // Remove loading indicator
    let _ = discordhelpers::delete_bot_reacts(ctx, msg, loading_reaction).await;

    // Handle compilation errors
    let CompilationResult { details, embed } = result?;

    // Log compilation if configured
    log_compilation(ctx, msg, &parse_result, details.success).await;

    Ok(HandleRequestResult {
        embed,
        details,
        parse_result,
    })
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
