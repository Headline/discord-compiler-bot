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
use crate::utls::discordhelpers;

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

// Pandas#3**2 on serenity disc, tyty
pub fn build_reaction(emoji_id: u64, emoji_name: &str) -> ReactionType {
    ReactionType::Custom {
        animated: false,
        id: EmojiId::from(emoji_id),
        name: Some(String::from(emoji_name)),
    }
}

pub fn build_compilation_embed(author: &User, res: & mut CompilationResult) -> CreateEmbed {
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
        let str = conform_external_str(&res.compiler_all);
        embed.field("Compiler Output", format!("```{}\n```", str), false);
    }
    if !res.program_all.is_empty() {
        let str = conform_external_str(&res.program_all);
        embed.field("Program Output", format!("```\n{}\n```", str), false);
    }
    if !res.url.is_empty() {
        embed.field("URL", &res.url, false);
    }

    embed.title("Compilation Results");
    embed.footer(|f| {
        f.text(format!(
            "Requested by: {} | Powered by wandbox.org",
            author.tag()
        ))
    });
    embed
}

// Certain compiler outputs use unicode control characters that
// make the user experience look nice (colors, etc). This ruins
// the look of the compiler messages in discord, so we strip them out
//
// Here we also limit the text to 1000 chars, this prevents discord from
// rejecting our embeds for being to long if someone decides to spam.
pub fn conform_external_str(input: &str) -> String {
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
    if str.len() > MAX_OUTPUT_LEN {
        str.chars().take(MAX_OUTPUT_LEN).collect()
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

            let compliant_str = discordhelpers::conform_external_str(&errs);
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
    embed.field("Guild", guild, true);
    let mut code = String::from(input_code);
    if code.len() > 800 {
        code = code[..800].to_owned();
    }
    embed.field("Code", format!("```{}\n{}\n```", lang, code), false);

    embed
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
