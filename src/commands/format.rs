use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError, Delimiter};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::cache::{CompilerCache};
use crate::utls::parser::{ParserResult, get_message_attachment};
use godbolt::Godbolt;
use std::io::Write;

#[command]
pub async fn format(ctx: &Context, msg: &Message, mut args : Args) -> CommandResult {
    let mut fmt = String::from("clangformat");
    let mut style = String::from("webkit");
    if !args.is_empty() {
        // do not include ``` codeblocks into arg parsing.. lets just substr and replace args
        let idx = msg.content.find("`");
        if let Some(idx) = idx {
            let substr : String = msg.content.chars().take(idx).collect();
            args = Args::new(&substr, &[Delimiter::Single(' ')]);
            args.advance();
        }

        // kind of odd - but since we replaced args we try again...
        if !args.is_empty() {
            fmt = args.single::<String>()?.trim().to_owned();

            style = String::from("");
            if !args.is_empty() {
                style = args.single::<String>()?.trim().to_owned();
            }
        }
    }

    let data = ctx.data.read().await;
    let comp_mgr = data.get::<CompilerCache>().unwrap().read().await;
    let gbolt = &comp_mgr.gbolt;

    // validate user input
    for format in &gbolt.formats {
        if format.format_type.to_ascii_lowercase().contains(&fmt.to_ascii_lowercase()) {
            // fmt is now valid - lets ensure case correctness
            fmt = format.format_type.clone();

            // if fmt has no styles - lets just empty the style string
            if format.styles.is_empty() {
                style = String::default();
            }
            else { // fmt does have styles - validate result if possible
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
        if crate::utls::parser::find_code_block(& mut result, &msgref.content) {
            lang_code = result.target.clone();
            code = result.code
        }
        else {
            if msgref.attachments.len() > 0 {
                attachment_name = msgref.attachments[0].filename.clone();
                let (program_code, _) = get_message_attachment(&msg.attachments).await?;
                code = program_code;
            }
            else {
                return Err(CommandError::from("Referenced message has no code or attachment"));
            }
        }
    }
    else {
        if !msg.attachments.is_empty() {
            attachment_name = msg.attachments[0].filename.clone();
            let (program_code, _) = get_message_attachment(&msg.attachments).await?;
            code = program_code;
        } else {
            let mut result = ParserResult::default();
            if crate::utls::parser::find_code_block(& mut result, &msg.content) {
                lang_code = result.target.clone();
                code = result.code
            }
            else {
                return Err(CommandError::from("Unable to find code to format!\n\nPlease reply to a message when executing this command or supply the code yourself in a code block or message attachment."));
            }
        }
    }


    let answer;
    {
        let result = Godbolt::format_code(&fmt, &style, &code, false, 4).await;
        match result {
            Ok(res) => {
                if res.exit != 0 {
                    return Err(CommandError::from("Formatter returned a non-zero exit code"));
                } else {
                    answer = res.answer;
                }
            }
            Err(err) => {
                return Err(CommandError::from(format!("An error occurred while formatting code: `{}`", err)));
            }
        }
    }

    if !attachment_name.is_empty() {
        let _ = std::fs::create_dir("temp");
        let path = format!("temp/{}", attachment_name);
        let mut file = std::fs::File::create(&path)?;
        let _ = file.write_all(answer.as_bytes());
        let _ = file.flush();

        msg.channel_id.send_message(&ctx.http, |msg| msg.add_file(path.as_str()).content("Powered by godbolt.org")).await?;
        let _ = std::fs::remove_file(&path);
    }
    else {
        msg.reply(&ctx.http, format!("\n```{}\n{}```\n*Powered by godbolt.org*", lang_code, answer)).await?;
    }
    Ok(())
}