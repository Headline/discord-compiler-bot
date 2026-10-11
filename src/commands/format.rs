use crate::cache::CompilerCache;
use crate::framework::{CommandError, CommandResult};
use crate::utls::discordhelpers;
use crate::utls::parser::{get_message_attachment, ParserResult};
use serenity::all::{CreateAttachment, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn format(ctx: &Context, msg: &Message, args: &str) -> CommandResult {
    let mut fmt = String::from("clangformat");
    let mut style = String::from("webkit");
    // do not include ``` codeblocks into arg parsing
    let head = args.split('`').next().unwrap_or_default();
    let mut parts = head.split_whitespace();
    if let Some(user_fmt) = parts.next() {
        fmt = user_fmt.to_owned();
        style = parts.next().unwrap_or_default().to_owned();
    }

    let data = ctx.data.read().await;
    let comp_mgr = data.get::<CompilerCache>().unwrap().read().await;
    let godbolt = comp_mgr.godbolt().ok_or_else(|| {
        CommandError::from("Compiler Explorer service is currently down, please try again later.")
    })?;

    // validate user input
    for format in &godbolt.formats {
        if format
            .format_type
            .to_ascii_lowercase()
            .contains(&fmt.to_ascii_lowercase())
        {
            // fmt is now valid - lets ensure case correctness
            fmt = format.format_type.clone();

            // if fmt has no styles - lets just empty the style string
            if format.styles.is_empty() {
                style = String::default();
            } else {
                // fmt does have styles - validate result if possible
                for fmtstyle in &format.styles {
                    if fmtstyle.to_ascii_lowercase().contains(&style) {
                        style = fmtstyle.to_string();
                    }
                }
            }
        }
    }

    let mut lang_code = String::new();
    let mut attachment_name = String::new();
    let code;

    if let Some(msgref) = &msg.referenced_message {
        let mut result = ParserResult::default();
        if crate::utls::parser::find_code_block(&mut result, &msgref.content, &msg.author).await? {
            lang_code = result.target.clone();
            code = result.code
        } else if !msgref.attachments.is_empty() {
            attachment_name = msgref.attachments[0].filename.clone();
            let (program_code, _) = get_message_attachment(&msgref.attachments).await?;
            code = program_code;
        } else {
            return Err(CommandError::from(
                "Referenced message has no code or attachment",
            ));
        }
    } else if !msg.attachments.is_empty() {
        attachment_name = msg.attachments[0].filename.clone();
        let (program_code, _) = get_message_attachment(&msg.attachments).await?;
        code = program_code;
    } else {
        let mut result = ParserResult::default();
        if crate::utls::parser::find_code_block(&mut result, &msg.content, &msg.author).await? {
            lang_code = result.target.clone();
            code = result.code
        } else {
            return Err(CommandError::from("Unable to find code to format!\n\nPlease reply to a message when executing this command or supply the code yourself in a code block or message attachment."));
        }
    }

    let answer;
    {
        let result = godbolt.format_code(&fmt, &style, &code, false, 4).await;
        match result {
            Ok(res) => {
                if res.exit != 0 {
                    return Err(CommandError::from(
                        "Formatter returned a non-zero exit code",
                    ));
                } else {
                    answer = res.answer;
                }
            }
            Err(err) => {
                return Err(CommandError::from(format!(
                    "An error occurred while formatting code: `{}`",
                    err
                )));
            }
        }
    }

    if !attachment_name.is_empty() {
        let attachment = CreateAttachment::bytes(answer.into_bytes(), attachment_name);

        let new_msg = discordhelpers::reply_to(
            msg,
            CreateMessage::new()
                .add_file(attachment)
                .content("Powered by godbolt.org"),
        );

        msg.channel_id.send_message(&ctx.http, new_msg).await?;
    } else {
        msg.reply(
            &ctx.http,
            format!(
                "\n```{}\n{}```\n*Powered by godbolt.org*",
                lang_code, answer
            ),
        )
        .await?;
    }
    Ok(())
}
