use std::str;
use std::sync::Arc;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::prelude::*,
};

use serenity_utils::menu::*;
use wandbox::*;

use crate::utls::constants::*;
use crate::utls::{discordhelpers, parser};
use tokio::sync::{MutexGuard};
use serenity::client::bridge::gateway::{ShardManager};
use crate::cache::{ConfigCache, GodboltCache};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use godbolt::{CompilationFilters, Godbolt};

pub fn build_menu_items(
    items: Vec<String>,
    items_per_page: usize,
    title: &str,
    avatar: &str,
    author: &str,
) -> Vec<CreateMessage<'static>> {
    let mut pages: Vec<CreateMessage> = Vec::new();
    let num_pages = items.len() / items_per_page;

    let mut current_page = 0;
    while current_page < num_pages + 1 {
        let start = current_page * items_per_page;
        let mut end = start + items_per_page;
        if end > items.len() {
            end = items.len();
        }
        let mut page = CreateMessage::default();
        page.embed(|e| {
            let mut description = String::new();
            for (i, item) in items[current_page * items_per_page..end].iter().enumerate() {
                if i > items_per_page {
                    break;
                }
                description.push_str(&format!(
                    "**{}**) {}\n",
                    current_page * items_per_page + i + 1,
                    item
                ))
            }
            e.color(COLOR_OKAY);
            e.title(title);
            e.description(description);
            e.footer(|f| {
                f.text(&format!(
                    "Requested by {} | Page {}/{}",
                    author,
                    current_page + 1,
                    num_pages + 1
                ))
            });
            e.thumbnail(avatar);
            e
        });

        pages.push(page);
        current_page += 1;
    }

    pages
}

pub fn build_menu_controls() -> MenuOptions {
    let controls = vec![
        Control::new(
            ReactionType::from('◀'),
            Arc::new(|m, r| Box::pin(prev_page(m, r))),
        ),
        Control::new(
            ReactionType::from('🛑'),
            Arc::new(|m, r| Box::pin(close_menu(m, r))),
        ),
        Control::new(
            ReactionType::from('▶'),
            Arc::new(|m, r| Box::pin(next_page(m, r))),
        ),
    ];

    // Let's create options for the menu.
    MenuOptions {
        controls,
        ..Default::default()
    }
}

pub async fn delete_and_close_menu(menu: &mut Menu<'_>, _reaction: Reaction) {
    let _ = menu
        .options
        .message
        .as_ref()
        .unwrap()
        .delete(&menu.ctx.http)
        .await;

    let _ = menu.msg.delete(&menu.ctx).await;
}

pub fn build_compile_controls() -> MenuOptions {
    let controls = vec![
        Control::new(
            ReactionType::from('◀'),
            Arc::new(|m, r| Box::pin(prev_page(m, r))),
        ),
        Control::new(
            ReactionType::from('🗑'),
            Arc::new(|m, r| Box::pin(delete_and_close_menu(m, r))),
        ),
        Control::new(
            ReactionType::from('▶'),
            Arc::new(|m, r| Box::pin(next_page(m, r))),
        ),
    ];

    // Let's create options for the menu.
    MenuOptions {
        controls,
        ..Default::default()
    }
}

pub fn get_page_count(result : &CompilationResult) -> usize {
    (result.program_all.chars().count()/MAX_OUTPUT_LEN) + 1
}

// Pandas#3**2 on serenity disc, tyty
pub fn build_reaction(emoji_id: u64, emoji_name: &str) -> ReactionType {
    ReactionType::Custom {
        animated: false,
        id: EmojiId::from(emoji_id),
        name: Some(String::from(emoji_name)),
    }
}

pub fn build_compilation_embed(author: &User, res: & mut CompilationResult, page_number : i32) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    if !res.status.is_empty() {
        if res.status != "0" {
            embed.color(COLOR_FAIL);
        } else {
            embed.field(
                "Status",
                format!("Finished with exit code: {}", &res.status),
                false,
            );
            embed.color(COLOR_OKAY);
        }
    }
    if !res.signal.is_empty() {
        embed.field("Signal", &res.signal, false);

        // If we received 'Signal', then the application successfully ran, but was timed out
        // by wandbox. We should skin this as successful, so we set status to 0 (success).
        // This is done to ensure that the checkmark is added at the end of the compile
        // command hook.
        embed.color(COLOR_OKAY);
        res.status = String::from('0');
    }
    if !res.compiler_all.is_empty() {
        let str = conform_external_str(&res.compiler_all, 0, MAX_ERROR_LEN);
        embed.field("Compiler Output", format!("```{}\n```", str), false);
    }
    if !res.program_all.is_empty() {
        let str = conform_external_str(&res.program_all, page_number, MAX_OUTPUT_LEN);
        embed.field("Program Output", format!("```\n{}\n```", str), false);
    }
    if !res.url.is_empty() {
        embed.field("URL", &res.url, false);
    }

    embed.title("Compilation Results");
    embed.footer(|f| {
        let count = get_page_count(res);
        if count > 1 {
            f.text(format!(
                "Requested by: {} | Powered by wandbox.org | {}/{}",
                author.tag(), page_number+1, count
            ))
        }
        else {
            f.text(format!(
                "Requested by: {} | Powered by wandbox.org",
                author.tag()
            ))
        }
    });
    embed
}

// Certain compiler outputs use unicode control characters that
// make the user experience look nice (colors, etc). This ruins
// the look of the compiler messages in discord, so we strip them out
//
// Here we also limit the text to 1000 chars, this prevents discord from
// rejecting our embeds for being to long if someone decides to spam.
pub fn conform_external_str(input: &str, page_number : i32, max_len : usize) -> String {
    let mut str: String;
    if let Ok(vec) = strip_ansi_escapes::strip(input) {
        str = String::from_utf8_lossy(&vec).to_string();
    } else {
        str = String::from(input);
    }

    // while we're at it, we'll escape ` characters with a
    // zero-width space to prevent our embed from getting
    // messed up later
    str = str.replace("`", "\u{200B}`");

    // Conform our string.
    if str.len() > max_len {
        if page_number > 0 {
            let it = str.chars();
            let skip = it.skip(max_len*(page_number as usize));
            skip.take(max_len).collect()
        }
        else {
            str.chars().take(max_len).collect()
        }
    } else {
        str
    }
}

pub fn build_asm_embed(author: &User, res: &godbolt::CompilationResult) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    match res.asm_size {
        Some(size) => {
            embed.color(COLOR_OKAY);
            size
        }
        None => {
            embed.color(COLOR_FAIL);

            let mut errs = String::new();
            for err_res in &res.stderr {
                let line = format!("{}\n", &err_res.text);
                errs.push_str(&line);
            }

            let compliant_str = discordhelpers::conform_external_str(&errs, 0, MAX_ERROR_LEN);
            embed.field(
                "Compilation Errors",
                format!("```\n{}```", compliant_str),
                false,
            );
            return embed;
        }
    };

    let mut pieces: Vec<String> = Vec::new();
    let mut append: String = String::new();
    if let Some(vec) = &res.asm {
        for asm in vec {
            if let Some(text) = &asm.text {
                if append.len() + text.len() > 1000 {
                    pieces.push(append.clone());
                    append.clear()
                }
                append.push_str(&format!("{}\n", text));
            }
        }
    }

    let mut i = 1;
    for str in pieces {
        let title = format!("Assembly Output Pt. {}", i);
        embed.field(&title, format!("```x86asm\n{}\n```", &str), false);
        i += 1;
    }
    if !append.is_empty() {
        let title;
        if i > 1 {
            title = format!("Assembly Output Pt. {}", i);
        } else {
            title = String::from("Assembly Output")
        }
        embed.field(&title, format!("```x86asm\n{}\n```", &append), false);
    }

    embed.title("Assembly Results");
    embed.footer(|f| {
        f.text(format!(
            "Requested by: {} | Powered by godbolt.org",
            author.tag()
        ))
    });
    embed
}

pub async fn manual_dispatch(http: Arc<Http>, id: u64, emb: CreateEmbed) {
    match serenity::model::id::ChannelId(id)
        .send_message(&http, |m| {
            m.embed(|mut e| {
                e.0 = emb.0;
                e
            })
        })
        .await
    {
        Ok(m) => m,
        Err(e) => return error!("Unable to dispatch manually: {}", e),
    };
}

pub fn embed_message(emb: CreateEmbed) -> CreateMessage<'static> {
    let mut msg = CreateMessage::default();
    msg.embed(|e| {
        e.0 = emb.0;
        e
    });
    msg
}

pub fn build_dblvote_embed(tag: String) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.color(COLOR_OKAY);
    embed.description(format!("{} voted for us on top.gg!", tag));
    embed.thumbnail(ICON_VOTE);
    embed
}

pub fn build_invite_embed(invite_link : &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Invite Link");
    embed.color(COLOR_OKAY);
    embed.thumbnail(ICON_INVITE);
    let description = format!("Click the link below to invite me to your server!\n\n[Invite me!]({})", invite_link);
    embed.description(description);
    embed
}

pub fn build_join_embed(guild: &Guild) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Guild joined");
    embed.color(COLOR_OKAY);
    embed.field("Name", guild.name.clone(), true);
    embed.field("Members", guild.member_count, true);
    embed.field("Channels", guild.channels.len(), true);
    if let Some(icon) = guild.icon_url() {
        embed.thumbnail(icon);
    }
    embed.field("Region", guild.region.clone(), true);
    embed.field("Guild ID", guild.id, true);
    embed
}

pub fn build_leave_embed(guild: &GuildId) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Guild left");
    embed.color(COLOR_FAIL);
    embed.field("ID", format!("{}", guild.0), true);
    embed
}

pub fn build_complog_embed(
    success: bool,
    input_code: &str,
    lang: &str,
    tag: &str,
    id: u64,
    guild: &str,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    if success {
        embed.color(COLOR_FAIL);
    } else {
        embed.color(COLOR_OKAY);
    }
    embed.title("Compilation requested");
    embed.field("Language", lang, true);
    embed.field("Author", tag, true);
    embed.field("Author ID", id, true);
    embed.field("Guild", guild, true);
    let mut code = String::from(input_code);
    if code.len() > MAX_OUTPUT_LEN {
        code = code.chars().take(MAX_OUTPUT_LEN).collect()
    }
    embed.field("Code", format!("```{}\n{}\n```", lang, code), false);

    embed
}

pub async fn send_global_presence(shard_manager : &MutexGuard<'_, ShardManager>, sum : u64) {
    // update shard guild count & presence
    let presence_str = format!("in {} servers | ;invite", sum);

    let runners = shard_manager.runners.lock().await;
    for (_, v) in runners.iter() {
        v.runner_tx.set_presence(Some(Activity::playing(&presence_str)), OnlineStatus::Online);
    }
}

pub fn build_fail_embed(author: &User, err: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.color(COLOR_FAIL);
    embed.title("Critical error:");
    embed.description(err);
    embed.thumbnail(ICON_FAIL);
    embed.footer(|f| f.text(format!("Requested by: {}", author.tag())));
    embed
}

pub async fn handle_edit(ctx : &Context, content : String, author : User, mut old : Message) {
    let prefix = {
        let data = ctx.data.read().await;
        let info = data.get::<ConfigCache>().unwrap().read().await;
        info.get("BOT_PREFIX").unwrap().to_owned()
    };

    // try to clear reactions
   let _ = old.delete_reactions(&ctx).await;

    if content.starts_with(&format!("{}asm", prefix)) {
        if let Err(e) = handle_edit_asm(&ctx, content, author.clone(), old.clone()).await {
            let _ = old.edit(&ctx, |m| {
                let err = build_fail_embed(&author, &e.to_string());
                m.embed(|e| {
                    e.0 = err.0;
                    e
                });
                m
            }).await;

        }
    }
    else if content.starts_with(&format!("{}compile", prefix)) {
        if let Err(e) = handle_edit_compile(&ctx, content, author.clone(), old.clone()).await {
            let _ = old.edit(&ctx, |m| {
                let err = build_fail_embed(&author, &e.to_string());
                m.embed(|e| {
                    e.0 = err.0;
                    e
                });
                m
            }).await;
        }
    }
    else {
        let _ = old.edit(&ctx, |m| {
            let err = build_fail_embed(&author, "Invalid command for edit functionality!");
            m.embed(|e| {
                e.0 = err.0;
                e
            });
            m
        }).await;
    }
}

pub async fn handle_edit_compile(ctx : &Context, content : String, author : User, mut old : Message) -> CommandResult {
    let success_id;
    let success_name;
    let loading_id;
    let loading_name;
    {
        let data_read = ctx.data.read().await;
        let botinfo_lock = data_read
            .get::<ConfigCache>()
            .expect("Expected ConfigCache in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        success_id = botinfo
            .get("SUCCESS_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        success_name = botinfo.get("SUCCESS_EMOJI_NAME").unwrap().clone();
        loading_id = botinfo
            .get("LOADING_EMOJI_ID")
            .unwrap()
            .clone()
            .parse::<u64>()
            .unwrap();
        loading_name = botinfo.get("LOADING_EMOJI_NAME").unwrap().clone();
    }

    use serenity::{
        framework::standard::{CommandError},
    };
    use crate::cache::WandboxCache;

    // aquire lock to our wandbox cache
    let data_read = ctx.data.read().await;
    let wandbox_lock = match data_read.get::<WandboxCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nWandbox cache is uninitialized, please file a bug.",
            ));
        }
    };

    // parse user input
    let parse_result = parser::get_components(&content, &author, wandbox_lock).await?;

    // build user input
    let mut builder = CompilationBuilder::new();
    builder.code(&parse_result.code);
    builder.target(&parse_result.target);
    builder.stdin(&parse_result.stdin);
    builder.save(true);
    builder.options(parse_result.options);

    let wbox = wandbox_lock.read().await;

    // build request
    match builder.build(&wbox) {
        Ok(()) => (),
        Err(e) => {
            return Err(CommandError::from(format!(
                "An internal error has occurred while building request.\n{}",
                e
            )));
        }
    };

    // lets see if we can manually fix botched java compilations...
    // for wandbox, "public class" is invalid, so lets do a quick replacement
    if builder.lang == "java" {
        builder.code(&parse_result.code.replacen("public class", "class", 1));
    }

    // send out loading emote
    let reaction = match old
        .react(
            &ctx.http,
            discordhelpers::build_reaction(loading_id, &loading_name),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    // dispatch our req
    let mut result = match builder.dispatch().await {
        Ok(r) => r,
        Err(e) => {
            // we failed, lets remove the loading react so it doesn't seem like we're still processing
            old.delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
                .await?;

            return Err(CommandError::from(format!("{}", e)));
        }
    };

    // remove our loading emote
    match old
        .delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
        .await
    {
        Ok(()) => (),
        Err(_e) => {
            return Err(CommandError::from(
                "Unable to remove reactions!\nAm I missing permission to manage messages?",
            ));
        }
    }

    let page_count = discordhelpers::get_page_count(&result);
    if page_count > 1 {
        return Err(CommandError::from("Large paginated outputs requires a new request."));
    }
    else { // single page - display normally.
        let emb = discordhelpers::build_compilation_embed(&author, &mut result, 0);
        let _ = old.edit(&ctx, |m| {
            m.embed(|e| {
                e.0 = emb.0;
                e
            });
            m
        }).await;


        // Success/fail react
        let reaction;
        if result.status == "0" {
            reaction = discordhelpers::build_reaction(success_id, &success_name);
        } else {
            reaction = ReactionType::Unicode(String::from("❌"));
        }
        old.react(&ctx.http, reaction).await?;
    }

    Ok(())
}
pub async fn handle_edit_asm(ctx : &Context, content : String, author : User, mut old : Message) -> CommandResult {
    use serenity::{
        framework::standard::{CommandError},
    };

    // aquire lock to our godbolt cache
    let data_read = ctx.data.read().await;
    let godbolt_lock = match data_read.get::<GodboltCache>() {
        Some(l) => l,
        None => {
            return Err(CommandError::from(
                "Internal request failure\nGodbolt cache is uninitialized, please file a bug.",
            ));
        }
    };

    // parse user input
    let result = match parser::get_components(&content, &author, godbolt_lock).await {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!("{}", e)));
        }
    };

    let godbolt = godbolt_lock.read().await;
    let c = match godbolt.resolve(&result.target) {
        Some(c) => c,
        None => {
            return Err(CommandError::from(format!(
                "Unable to find valid compiler or language '{}'\n",
                &result.target
            )));
        }
    };

    // send out loading emote
    let reaction = match old
        .react(
            &ctx.http,
            discordhelpers::build_reaction(752440820036272139, "compiler_loading2"),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(CommandError::from(format!(" Unable to react to message, am I missing permissions to react or use external emoji?\n{}", e)));
        }
    };

    let filters = CompilationFilters {
        binary: None,
        comment_only: Some(true),
        demangle: Some(true),
        directives: Some(true),
        execute: None,
        intel: Some(true),
        labels: Some(true),
        library_code: None,
        trim: Some(true),
    };

    let response =
        match Godbolt::send_request(&c, &result.code, &result.options.join(" "), &filters).await {
            Ok(resp) => resp,
            Err(e) => {
                // we failed, lets remove the loading react before leaving so it doesn't seem like we're still processing
                old.delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
                    .await?;
                return Err(CommandError::from(format!(
                    "Godbolt request failed!\n\n{}",
                    e
                )));
            }
        };

    // remove our loading emote
    match old
        .delete_reaction_emoji(&ctx.http, reaction.emoji.clone())
        .await
    {
        Ok(()) => (),
        Err(_e) => {
            return Err(CommandError::from(
                "Unable to remove reactions!\nAm I missing permission to manage messages?",
            ));
        }
    }

    let emb = discordhelpers::build_asm_embed(&author, &response);
    let _ = old.edit(&ctx, |m| {
        m.embed(|e| {
            e.0 = emb.0;
            e
        });
        m
    }).await;

    let reaction;
    if response.asm_size.is_some() {
        reaction = discordhelpers::build_reaction(764356794352009246, "checkmark2");
    } else {
        reaction = ReactionType::Unicode(String::from("❌"));
    }

    old.react(&ctx.http, reaction).await?;
    Ok(())
}