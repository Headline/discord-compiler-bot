use std::fmt::Write as _;
use std::{env, str};

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    model::prelude::*,
};

use wandbox::*;

use crate::utls::constants::*;
use crate::utls::discordhelpers;

#[derive(Default)]
pub struct EmbedOptions {
    pub is_assembly: bool,
    pub lang: String,
    pub compiler: String,
}
impl EmbedOptions {
    pub fn new(is_assembly: bool, lang: String, compiler: String) -> Self {
        EmbedOptions {
            is_assembly,
            lang,
            compiler,
        }
    }
}

pub trait ToEmbed {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed;
}

impl ToEmbed for wandbox::CompilationResult {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed {
        let mut embed = CreateEmbed::default();

        if !self.status.is_empty() {
            if self.status != "0" {
                embed.color(COLOR_FAIL);
            } else {
                embed.color(COLOR_OKAY);
            }
        }

        if !self.signal.is_empty() {
            // If we received 'Signal', then the application successfully ran, but was timed out
            // by wandbox. We should skin this as successful, so we set status to 0 (success).
            // This is done to ensure that the checkmark is added at the end of the compile
            // command hook.
            embed.color(COLOR_OKAY);
        }
        if !self.compiler_all.is_empty() {
            let str = discordhelpers::conform_external_str(&self.compiler_all, MAX_ERROR_LEN);
            embed.field("Compiler Output", format!("```{}\n```", str), false);
        }
        if !self.program_all.is_empty() {
            let str = discordhelpers::conform_external_str(&self.program_all, MAX_OUTPUT_LEN);
            embed.field("Program Output", format!("```\n{}\n```", str), false);
        }
        if !self.url.is_empty() {
            embed.field("URL", &self.url, false);
        }

        embed.footer(|f| {
            let mut text = author.tag();

            if !options.lang.is_empty() {
                text = format!("{} | {}", text, options.lang);
            }
            if !options.compiler.is_empty() {
                text = format!("{} | {}", text, options.compiler);
            }

            text = format!("{} | wandbox.org", text);
            f.text(text)
        });
        embed
    }
}

impl ToEmbed for godbolt::GodboltResponse {
    fn to_embed(self, author: &User, options: &EmbedOptions) -> CreateEmbed {
        let mut embed = CreateEmbed::default();

        if self.code == 0 {
            embed.color(COLOR_OKAY);
        } else {
            embed.color(COLOR_FAIL);

            // if it's an assembly request let's just handle the error case here.
            if options.is_assembly {
                let mut errs = String::new();
                for err_res in &self.stderr {
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

                let piece = discordhelpers::conform_external_str(&str, MAX_OUTPUT_LEN);
                embed.field(&title, format!("```x86asm\n{}\n```", &piece), false);
                output = true;
                i += 1;
            }
            if !append.is_empty() {
                let title = if i > 1 {
                    format!("Assembly Output Pt. {}", i)
                } else {
                    String::from("Assembly Output")
                };

                let str = discordhelpers::conform_external_str(&append, MAX_OUTPUT_LEN);
                embed.field(&title, format!("```x86asm\n{}\n```", &str), false);
                output = true;
            }

            if !output {
                embed.title("Compilation successful");
                embed.description("No assembly generated.");
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
                embed.field("Program Output", format!("```\n{}\n```", str), false);
                output = true;
            }
            if !stderr.is_empty() {
                output = true;
                let str = discordhelpers::conform_external_str(stderr, MAX_ERROR_LEN);
                embed.field("Compiler Output", format!("```\n{}\n```", str), false);
            }

            if !output {
                embed.title("Compilation successful");
                embed.description("No output.");
            }

            // Execution time can be displayed here, but I don't think it's useful enough
            // to show...
            //embed.field("Execution Time", format!("`{}ms`", self.execution_time), true);
        }

        let mut appendstr = author.tag();
        if let Some(time) = self.execution_time {
            appendstr = format!("{} | {}ms", appendstr, time);
        }
        if !options.lang.is_empty() {
            appendstr = format!("{} | {}", appendstr, options.lang);
        }
        if !options.compiler.is_empty() {
            appendstr = format!("{} | {}", appendstr, options.compiler);
        }

        embed.footer(|f| f.text(format!("{} | godbolt.org", appendstr)));
        embed
    }
}

pub async fn edit_message_embed(ctx: &Context, old: &mut Message, emb: CreateEmbed) {
    let _ = old
        .edit(ctx, |m| {
            m.embed(|e| {
                e.0 = emb.0;
                e
            });
            m
        })
        .await;
}

#[allow(dead_code)]
pub fn build_small_compilation_embed(author: &User, res: &mut CompilationResult) -> CreateEmbed {
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

pub fn panic_embed(panic_info: String) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Oopsie");
    embed.description(format!("```\n{}\n```", panic_info));
    embed
}

pub fn build_welcome_embed() -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let prefix = env::var("BOT_PREFIX").expect("Bot prefix is not set!");
    embed.title("Discord Compiler");
    embed.color(COLOR_OKAY);
    embed.thumbnail(COMPILER_ICON);
    embed.description("Thanks for inviting me to your discord server!");
    embed.field("Introduction", "I can take code that you give me and execute it, display generated assembly, or format it!", true);
    embed.field(
        "Example Request",
        format!("{}compile python\n```py\nprint('hello world')\n```", prefix),
        true,
    );
    embed.field("Learning Time!", "If you like reading the manuals of things, read our [getting started](https://github.com/Headline/discord-compiler-bot/wiki/Getting-Started) wiki or if you are confident type `;help` to view all commands.", false);
    embed.field("Support", "If you ever run into any issues please stop by our [support server](https://discord.com/invite/nNNEZ6s) and we'll give you a hand.", true);
    embed.footer(|f| {
        f.text(
            "powered by godbolt.org & wandbox.org // created by Michael Flaherty (Headline#9999)",
        )
    });
    embed
}

pub fn build_invite_embed(invite_link: &str) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Invite Link");
    embed.color(COLOR_OKAY);
    embed.thumbnail(ICON_INVITE);
    let description = format!(
        "Click the link below to invite me to your server!\n\n[Invite me!]({})",
        invite_link
    );
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
    if !success {
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

pub fn build_publish_embed() -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.color(COLOR_WARN).description(
        "This result will not be visible to others until you click the publish button.\n\n \
                    If you are unhappy with your results please start a new compilation request \
                    and dismiss this message.",
    );
    embed
}
