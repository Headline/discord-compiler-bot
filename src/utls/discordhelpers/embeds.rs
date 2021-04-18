use std::str;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::prelude::*,
    client::Context
};

use wandbox::*;

use crate::utls::constants::*;
use crate::utls::{discordhelpers};

pub async fn edit_message_embed(ctx : &Context, old : & mut Message, emb : CreateEmbed) {
    let _ = old.edit(ctx, |m| {
        m.embed(|e| {
            e.0 = emb.0;
            e
        });
        m
    }).await;
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
        let str = discordhelpers::conform_external_str(&res.compiler_all,  MAX_ERROR_LEN);
        embed.field("Compiler Output", format!("```{}\n```", str), false);
    }
    if !res.program_all.is_empty() {
        let str = discordhelpers::conform_external_str(&res.program_all, MAX_OUTPUT_LEN);
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

            let compliant_str = discordhelpers::conform_external_str(&errs, MAX_ERROR_LEN);
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

pub fn build_small_compilation_embed(author: &User, res: & mut CompilationResult) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    if res.status != "0" {
        embed.color(COLOR_FAIL);
    } else {
        embed.color(COLOR_OKAY);
    }

    if !res.compiler_all.is_empty() {
        let str = discordhelpers::conform_external_str(&res.compiler_all, MAX_ERROR_LEN);
        embed.field("Compiler Output", format!("```{}\n```", str), false);
    }
    if !res.program_all.is_empty() {
        let str = discordhelpers::conform_external_str(&res.program_all, MAX_OUTPUT_LEN);
        embed.description(format!("```\n{}\n```", str));
    }
    embed.footer(|f| {
        f.text(format!(
            "Requested by: {} | Powered by wandbox.org",
            author.tag()
        ))
    });

    embed
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

pub fn build_fail_embed(author: &User, err: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.color(COLOR_FAIL);
    embed.title("Critical error:");
    embed.description(err);
    embed.thumbnail(ICON_FAIL);
    embed.footer(|f| f.text(format!("Requested by: {}", author.tag())));
    embed
}
