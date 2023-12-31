use std::fmt::Write as _;
use std::{env, str};

use serenity::all::{CreateActionRow, CreateButton, CreateEmbedFooter, EditMessage};
use serenity::http::Http;
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    model::prelude::*,
};

use crate::apis::insights::InsightsResponse;
use crate::cache::LinkAPICache;
use crate::managers::compilation::CompilationDetails;

use crate::utls::constants::*;
use crate::utls::discordhelpers;

#[derive(Default)]
pub struct EmbedOptions {
    pub is_assembly: bool,
    pub compilation_info: CompilationDetails,
}

impl EmbedOptions {
    pub fn new(is_assembly: bool, compilation_info: CompilationDetails) -> Self {
        EmbedOptions {
            is_assembly,
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

        if !self.status.is_empty() {
            if self.status != "0" {
                embed = embed.color(COLOR_FAIL);
            } else {
                embed = embed.color(COLOR_OKAY);
            }
        }

        if !self.signal.is_empty() {
            // If we received 'Signal', then the application successfully ran, but was timed out
            // by wandbox. We should skin this as successful, so we set status to 0 (success).
            // This is done to ensure that the checkmark is added at the end of the compile
            // command hook.
            embed = embed.color(COLOR_OKAY);
        }
        if !self.compiler_all.is_empty() {
            let str = discordhelpers::conform_external_str(&self.compiler_all, MAX_ERROR_LEN);
            embed = embed.field("Compiler Output", format!("```{}\n```", str), false);
        }
        if !self.program_all.is_empty() {
            let str = discordhelpers::conform_external_str(&self.program_all, MAX_OUTPUT_LEN);
            embed = embed.field("Program Output", format!("```\n{}\n```", str), false);
        }
        if !self.url.is_empty() {
            embed = embed.field("URL", &self.url, false);
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

impl ToEmbed for godbolt::GodboltResponse {
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

                let compliant_str = discordhelpers::conform_external_str(&errs, MAX_ERROR_LEN);
                return embed.field(
                    "Compilation Errors",
                    format!("```\n{}```", compliant_str),
                    false,
                );
            }
        };

        if options.is_assembly {
            let mut pieces: Vec<String> = Vec::new();
            let mut append: String = String::new();
            if let Some(vec) = &self.asm {
                for asm in vec {
                    if let Some(text) = &asm.text {
                        if append.len() + text.len() > 1000 {
                            pieces.push(append.clone());
                            append.clear()
                        }
                        // append.push_str(&format!("{}\n", text));
                        writeln!(append, "{}", text).unwrap();
                    }
                }
            }

            let mut output = false;
            let mut i = 1;
            for str in pieces {
                let title = format!("Assembly Output Pt. {}", i);

                let piece = str.replace('`', "\u{200B}`");
                embed = embed.field(&title, format!("```x86asm\n{}\n```", &piece), false);
                output = true;
                i += 1;
            }
            if !append.is_empty() {
                let title = if i > 1 {
                    format!("Assembly Output Pt. {}", i)
                } else {
                    String::from("Assembly Output")
                };

                let str = append.replace('`', "\u{200B}`");
                embed = embed.field(title, format!("```x86asm\n{}\n```", &str), false);
                output = true;
            }

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
            if let Some(build_result) = self.build_result {
                if let Some(errors) = build_result.stderr {
                    for line in errors {
                        writeln!(errs, "{}", line.text).unwrap();
                    }
                }
            }

            for line in &self.stderr {
                writeln!(errs, "{}", line.text).unwrap();
            }

            let stdout = output.trim();
            let stderr = errs.trim();
            let mut output = false;
            if !stdout.is_empty() {
                let str = discordhelpers::conform_external_str(stdout, MAX_OUTPUT_LEN);
                embed = embed.field("Program Output", format!("```\n{}\n```", str), false);
                output = true;
            }
            if !stderr.is_empty() {
                output = true;
                let str = discordhelpers::conform_external_str(stderr, MAX_ERROR_LEN);
                embed = embed.field("Compiler Output", format!("```\n{}\n```", str), false);
            }

            if !output {
                embed = embed
                    .title("Compilation successful")
                    .description("No output.");
            }

            // Execution time can be displayed here, but I don't think it's useful enough
            // to show...
            //embed.field("Execution Time", format!("`{}ms`", self.execution_time), true);
        }

        let mut appendstr = author.name.clone();
        if let Some(time) = self.execution_time {
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

pub async fn edit_message_embed(
    ctx: &Context,
    old: &mut Message,
    emb: CreateEmbed,
    compilation_details: Option<CompilationDetails>,
) {
    let mut url = None;
    if let Some(details) = compilation_details {
        let data = ctx.data.read().await;
        if let Some(link_cache) = data.get::<LinkAPICache>() {
            if let Some(b64) = details.base64 {
                let long_url = format!("https://godbolt.org/clientstate/{}", b64);
                let link_cache_lock = link_cache.read().await;
                url = link_cache_lock.get_link(long_url).await
            }
        }
    }

    let mut btns = Vec::new();

    if let Some(shorturl) = url {
        btns.push(CreateButton::new_link(shorturl).label("View on godbolt.org"));
    }

    let edit = EditMessage::new()
        .embed(emb)
        .components(vec![CreateActionRow::Buttons(btns)]);
    let _ = old.edit(ctx, edit).await;
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
    let footer = CreateEmbedFooter::new(
        "powered by godbolt.org & wandbox.org // created by Michael Flaherty (@headline)",
    );
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
        .field("Learning Time!", format!("If you like reading the manuals of things, read our [getting started](https://github.com/Headline/discord-compiler-bot/wiki/Getting-Started) wiki or if you are confident type `{0}help` to view all commands.", prefix), false)
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
