use std::fmt::Write as _;
use std::{env, str};

use crate::apis::insights::InsightsResponse;
use crate::cache::LinkAPICache;
use crate::managers::compilation::CompilationDetails;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbedFooter, EditMessage};
use serenity::http::Http;
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    model::prelude::*,
};

use crate::utls::constants::*;
use crate::utls::discordhelpers;

#[derive(Default)]
pub struct EmbedOptions {
    pub is_assembly: bool,
    pub preprocessor: bool,
    pub compilation_info: CompilationDetails,
}

impl EmbedOptions {
    pub fn new(
        is_assembly: bool,
        preprocessor: bool,
        compilation_info: CompilationDetails,
    ) -> Self {
        EmbedOptions {
            is_assembly,
            preprocessor,
            compilation_info,
        }
    }
}

pub trait ToEmbed {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed;
}

impl ToEmbed for wandbox::CompilationResult {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed {
        let mut embed = CreateEmbed::new();

        if let Some(status) = self.status {
            if status != 0 {
                embed = embed.color(COLOR_FAIL);
            } else {
                embed = embed.color(COLOR_OKAY);
            }
        }

        if !self.signal.is_empty() {
            // If we received 'Signal', then the application successfully ran, but was timed out
            // by wandbox. We should skin this as successful as it causes confusion as to who
            // actually failed (ourselves vs wandbox)
            embed = embed.color(COLOR_OKAY);
        }
        if !self.compiler_message.is_empty() {
            let str =
                discordhelpers::conform_external_str(&self.compiler_message, MAX_ERROR_LEN, true);
            embed = embed.field("Compiler Output", format!("```{}\n```", str), false);
        }
        if !self.program_message.is_empty() {
            let str =
                discordhelpers::conform_external_str(&self.program_message, MAX_OUTPUT_LEN, true);
            embed = embed.field("Program Output", format!("```\n{}\n```", str), false);
        }
        if let Some(url) = self.url.as_deref().filter(|url| !url.is_empty()) {
            embed = embed.field("URL", url, false);
        }

        let mut text = author.name.clone();
        if !options.compilation_info.language.is_empty() {
            text = format!("{} | {}", text, options.compilation_info.language);
        }
        if !options.compilation_info.compiler.is_empty() {
            text = format!("{} | {}", text, options.compilation_info.compiler);
        }

        text = format!("{} | wandbox.org", text);

        let footer = CreateEmbedFooter::new(text);
        embed.footer(footer)
    }
}

impl ToEmbed for crate::apis::sourcepawn::SourcePawnResponse {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed {
        let mut embed = CreateEmbed::new();
        let overall = self.compile.success && self.run.as_ref().map(|r| r.success).unwrap_or(true);
        embed = embed.color(if overall { COLOR_OKAY } else { COLOR_FAIL });

        let compiler_msg = clean_spcomp_output(
            &format!("{}\n{}", self.compile.stdout, self.compile.stderr),
            // The size stats are only interesting for compile-only requests
            self.run.is_none() && !options.is_assembly,
        );

        if options.is_assembly {
            if !self.compile.success {
                return embed.field(
                    "Compilation Errors",
                    format!(
                        "```\n{}```",
                        discordhelpers::conform_external_str(&compiler_msg, MAX_ERROR_LEN, true)
                    ),
                    false,
                );
            }

            let asm_text = self.asm.map(|asm| asm.stdout).unwrap_or_default();
            let (pieces, remainder) = chunk_output(asm_text.lines());
            let (new_embed, output) =
                append_output_fields(embed, pieces, remainder, "Assembly Output", "");
            embed = new_embed;

            if !output {
                embed = embed
                    .title("Compilation successful")
                    .description("No assembly generated.");
            }
        } else {
            if !compiler_msg.is_empty() {
                let str = discordhelpers::conform_external_str(&compiler_msg, MAX_ERROR_LEN, true);
                embed = embed.field("Compiler Output", format!("```\n{}\n```", str), false);
            }

            match &self.run {
                Some(run) => {
                    let mut program = run.stdout.trim().to_string();
                    if !run.stderr.trim().is_empty() {
                        if !program.is_empty() {
                            program.push('\n');
                        }
                        program.push_str(run.stderr.trim());
                    }
                    if run.truncated {
                        if !program.is_empty() {
                            program.push('\n');
                        }
                        program.push_str("… (output truncated)");
                    }
                    if run.timed_out {
                        if !program.is_empty() {
                            program.push('\n');
                        }
                        program.push_str("Execution timed out.");
                    } else if !run.success {
                        if let Some(code) = run.exit_code.filter(|&code| code != 0) {
                            if !program.is_empty() {
                                program.push('\n');
                            }
                            program.push_str(&format!("Exited with code {}.", code));
                        }
                    }

                    if !program.is_empty() {
                        let str =
                            discordhelpers::conform_external_str(&program, MAX_OUTPUT_LEN, true);
                        embed = embed.field("Program Output", format!("```\n{}\n```", str), false);
                    } else {
                        embed = embed.title("Execution successful");
                    }
                }
                None => {
                    if compiler_msg.is_empty() {
                        embed = embed.title("Compilation successful");
                    }
                }
            }
        }

        let mut text = author.name.clone();
        if !options.compilation_info.language.is_empty() {
            text = format!("{} | {}", text, options.compilation_info.language);
        }
        if !options.compilation_info.compiler.is_empty() {
            text = format!("{} | {}", text, options.compilation_info.compiler);
        }

        embed.footer(CreateEmbedFooter::new(text))
    }
}

/// Drop spcomp's banner (and optionally its size stats) from compiler output
fn clean_spcomp_output(output: &str, keep_stats: bool) -> String {
    output
        .lines()
        .filter(|line| {
            !line.starts_with("SourcePawn Compiler") && !line.starts_with("Copyright (c)")
        })
        .filter(|line| {
            keep_stats
                || !(line.starts_with("Code size:")
                    || line.starts_with("Data size:")
                    || line.starts_with("Stack/heap size:")
                    || line.starts_with("Total requirements:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

impl ToEmbed for godbolt::CompilationResult {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed {
        let mut embed = CreateEmbed::new();
        if self.code == 0 {
            embed = embed.color(COLOR_OKAY);
        } else {
            embed = embed.color(COLOR_FAIL);

            // if it's an assembly request let's just handle the error case here.
            if options.is_assembly {
                let mut errs = String::new();
                for err_res in &self.stderr {
                    let line = format!("{}\n", &err_res.text);
                    errs.push_str(&line);
                }

                let compliant_str =
                    discordhelpers::conform_external_str(&errs, MAX_ERROR_LEN, true);
                return embed.field(
                    "Compilation Errors",
                    format!("```\n{}```", compliant_str),
                    false,
                );
            }
        };

        if options.is_assembly {
            // When the request asked for preprocessor output (with filtered headers),
            // show that cleaner source instead of the raw assembly. Fall back to the
            // assembly if no preprocessor output was produced (e.g. non-C/C++ targets).
            let pp_output = if options.preprocessor {
                self.pp_output
                    .as_ref()
                    .map(|pp| pp.output.trim())
                    .filter(|out| !out.is_empty())
            } else {
                None
            };

            let (pieces, remainder, base_title, fence) = match pp_output {
                Some(pp) => {
                    let (pieces, remainder) = chunk_output(pp.lines());
                    (pieces, remainder, "Preprocessor Output", "cpp")
                }
                None => {
                    let (pieces, remainder) = chunk_output(
                        self.asm
                            .iter()
                            .flatten()
                            .filter_map(|asm| asm.text.as_deref()),
                    );
                    (pieces, remainder, "Assembly Output", "x86asm")
                }
            };

            let (new_embed, output) =
                append_output_fields(embed, pieces, remainder, base_title, fence);
            embed = new_embed;

            if !output {
                embed = embed
                    .title("Compilation successful")
                    .description("No assembly generated.");
            }
        } else {
            let mut output = String::default();
            for line in &self.stdout {
                writeln!(output, "{}", line.text).unwrap();
            }

            let mut errs = String::default();
            if let Some(build_result) = &self.build_result {
                for line in &build_result.stderr {
                    writeln!(errs, "{}", line.text).unwrap();
                }
            }

            for line in &self.stderr {
                writeln!(errs, "{}", line.text).unwrap();
            }

            let stdout = output.trim();
            let stderr = errs.trim();
            let mut output = false;
            if !stdout.is_empty() {
                let str = discordhelpers::conform_external_str(stdout, MAX_OUTPUT_LEN, true);
                embed = embed.field("Program Output", format!("```\n{}\n```", str), false);
                output = true;
            }
            if !stderr.is_empty() {
                output = true;
                let str = discordhelpers::conform_external_str(stderr, MAX_ERROR_LEN, true);
                embed = embed.field("Compiler Output", format!("```\n{}\n```", str), false);
            }

            if !output {
                embed = embed.title("Compilation successful");
            }
        }

        let mut appendstr = author.name.clone();
        if let Some(time) = self.exec_time {
            appendstr = format!("{} | {}ms", appendstr, time);
        }
        if !options.compilation_info.language.is_empty() {
            appendstr = format!("{} | {}", appendstr, options.compilation_info.language);
        }
        if !options.compilation_info.compiler.is_empty() {
            appendstr = format!("{} | {}", appendstr, options.compilation_info.compiler);
        }

        let footer = CreateEmbedFooter::new(format!("{} | godbolt.org", appendstr));
        embed.footer(footer)
    }
}

/// Split lines into <=1000-char chunks at line boundaries, returning the
/// completed chunks plus the trailing remainder. This keeps each embed field
/// within Discord's size limits when output spans multiple fields.
fn chunk_output<'a>(lines: impl Iterator<Item = &'a str>) -> (Vec<String>, String) {
    let mut pieces: Vec<String> = Vec::new();
    let mut append = String::new();
    for line in lines {
        if append.len() + line.len() > 1000 {
            pieces.push(std::mem::take(&mut append));
        }
        writeln!(append, "{}", line).unwrap();
    }
    (pieces, append)
}

/// Render chunked output as one or more code-fence embed fields, using the
/// "<title> Pt. N" scheme when the output spans multiple chunks. Returns the
/// updated embed and whether any field was added.
fn append_output_fields(
    mut embed: CreateEmbed,
    pieces: Vec<String>,
    remainder: String,
    base_title: &str,
    fence: &str,
) -> (CreateEmbed, bool) {
    let mut output = false;
    let mut i = 1;
    for piece in pieces {
        let title = format!("{} Pt. {}", base_title, i);
        let body = piece.replace('`', "\u{200B}`");
        embed = embed.field(&title, format!("```{}\n{}\n```", fence, body), false);
        output = true;
        i += 1;
    }
    if !remainder.is_empty() {
        let title = if i > 1 {
            format!("{} Pt. {}", base_title, i)
        } else {
            base_title.to_string()
        };
        let body = remainder.replace('`', "\u{200B}`");
        embed = embed.field(title, format!("```{}\n{}\n```", fence, body), false);
        output = true;
    }
    (embed, output)
}

pub async fn edit_message_embed(
    ctx: &Context,
    old: &mut Message,
    emb: &mut CreateEmbed,
    compilation_details: Option<CompilationDetails>,
) -> serenity::Result<()> {
    let mut url = None;
    if let Some(details) = compilation_details {
        let data = ctx.data.read().await;
        if let Some(link_cache) = data.get::<LinkAPICache>() {
            if let Some(b64) = details.godbolt_base64 {
                let long_url = format!("https://godbolt.org/clientstate/{}", b64);
                let link_cache_lock = link_cache.read().await;
                url = link_cache_lock.get_link(long_url).await;
            }
        }
    }

    let mut btns = Vec::new();

    if let Some(shorturl) = url {
        btns.push(CreateButton::new_link(shorturl).label("View on godbolt.org"));
    }

    let edit = {
        if btns.is_empty() {
            EditMessage::default()
                .embed(emb.clone())
                .components(Vec::new())
        } else {
            EditMessage::default()
                .components(vec![CreateActionRow::Buttons(btns)])
                .embed(emb.clone())
        }
    };

    old.edit(ctx, edit).await?;
    Ok(())
}

pub fn build_insights_response_embed(author: &User, res: InsightsResponse) -> CreateEmbed {
    let error = res.return_code != 0;
    let footer = CreateEmbedFooter::new(format!(
        "Requested by: {} | Powered by cppinsights.io",
        author.name
    ));

    CreateEmbed::default()
        .color(if error { COLOR_FAIL } else { COLOR_OKAY })
        .description(format!(
            "```cpp\n{}```",
            if error { res.stderr } else { res.stdout }
        ))
        .footer(footer)
}

pub fn embed_message(emb: CreateEmbed) -> CreateMessage {
    CreateMessage::default().embed(emb)
}

pub async fn dispatch_embed(
    http: &Http,
    channel: ChannelId,
    emb: CreateEmbed,
) -> serenity::Result<Message> {
    let emb_msg = embed_message(emb);
    channel.send_message(&http, emb_msg).await
}

/// Send an embed as a silent reply to the invoking message
pub async fn reply_embed(
    http: &Http,
    msg: &Message,
    emb: CreateEmbed,
) -> serenity::Result<Message> {
    let emb_msg = discordhelpers::reply_to(msg, embed_message(emb));
    msg.channel_id.send_message(&http, emb_msg).await
}

pub fn build_dblvote_embed(tag: String) -> CreateEmbed {
    CreateEmbed::new()
        .color(COLOR_OKAY)
        .description(format!("{} voted for us on top.gg!", tag))
        .thumbnail(ICON_VOTE)
}

pub fn panic_embed(panic_info: String) -> CreateEmbed {
    CreateEmbed::new()
        .title("Oopsie")
        .description(format!("```\n{}\n```", panic_info))
}

pub fn build_welcome_embed() -> CreateEmbed {
    let footer =
        CreateEmbedFooter::new("powered by godbolt.org & wandbox.org // created by @headline");
    let prefix = env::var("BOT_PREFIX").expect("Bot prefix is not set!");

    CreateEmbed::new()
        .title("Discord Compiler")
        .thumbnail(COMPILER_ICON)
        .color(COLOR_OKAY)
        .description("Thanks for inviting me to your discord server!")
        .field("Introduction", "I can take code that you give me and execute it, display generated assembly, or format it!", true)
        .field(
            "Example Request",
            format!("{}compile python\n```py\nprint('hello world')\n```", prefix),
            true,
        )
        .field("Learning Time!", format!("If you like reading the manuals of things, read our [getting started](https://github.com/Headline/discord-compiler-bot/wiki/1.-Getting-Started) wiki or if you are confident type `{0}help` to view all commands.", prefix), false)
        .field("Support", "If you ever run into any issues please stop by our [support server](https://discord.com/invite/nNNEZ6s) and we'll give you a hand.", true)
        .footer(footer)
}

pub fn build_invite_embed(invite_link: &str) -> CreateEmbed {
    let description = format!(
        "Click the link below to invite me to your server!\n\n[Invite me!]({})",
        invite_link
    );

    CreateEmbed::new()
        .title("Invite Link")
        .color(COLOR_OKAY)
        .thumbnail(ICON_INVITE)
        .description(description)
}

pub fn build_join_embed(guild: &Guild) -> CreateEmbed {
    let mut emb = CreateEmbed::default()
        .title("Guild joined")
        .color(COLOR_OKAY)
        .field("Name", guild.name.clone(), true)
        .field("Members", guild.member_count.to_string(), true)
        .field("Channels", guild.channels.len().to_string(), true)
        .field("Guild ID", guild.id.to_string(), true);

    if let Some(icon) = guild.icon_url() {
        emb = emb.thumbnail(icon);
    }

    emb
}

pub fn build_leave_embed(guild: &GuildId) -> CreateEmbed {
    CreateEmbed::new()
        .title("Guild left")
        .color(COLOR_FAIL)
        .field("ID", format!("{}", guild.get()), true)
}

pub fn build_complog_embed(
    success: bool,
    input_code: &str,
    lang: &str,
    tag: &str,
    id: UserId,
    guild: &str,
) -> CreateEmbed {
    let embed = CreateEmbed::new()
        .color(if success { COLOR_OKAY } else { COLOR_FAIL })
        .title("Compilation requested")
        .field("Language", lang, true)
        .field("Author", tag, true)
        .field("Author ID", id.to_string(), true)
        .field("Guild", guild, true);

    let mut code = String::from(input_code);
    if code.len() > MAX_OUTPUT_LEN {
        code = code.chars().take(MAX_OUTPUT_LEN).collect()
    }

    embed.field("Code", format!("```{}\n{}\n```", lang, code), false)
}

pub fn build_fail_embed(author: &User, err: &str) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(format!("Requested by: {}", author.name));

    CreateEmbed::new()
        .color(COLOR_FAIL)
        .title("Critical error:")
        .description(err)
        .thumbnail(ICON_FAIL)
        .footer(footer)
}
