use crate::utls::constants::{CODE_BLOCK_REGEX, C_LIKE_INCLUDE_REGEX, URL_ALLOW_LIST};

use serenity::framework::standard::CommandError;
use serenity::model::channel::{Attachment, Message};
use serenity::model::user::User;

use crate::managers::compilation::{CompilationManager, RequestHandler};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

// Allows us to convert some common aliases to other programming languages
pub fn shortname_to_qualified(language: &str) -> &str {
    match language {
        // Replace cpp with c++ since we removed the c pre-processor
        // support for wandbox. This is okay for godbolt requests, too.
        "cpp" | "hpp" | "h++" => "c++",
        "h" => "c",
        "rs" => "rust",
        "js" => "javascript",
        "ts" => "typescript",
        "csharp" | "cs" => "c#",
        "py" => "python",
        "bash" | "sh" => "bash script",
        "rb" => "ruby",
        "kt" => "kotlin",
        "golang" => "go",
        "fs" | "f#" => "fsharp",
        "hs" | "lhs" => "haskell",
        "jl" => "julia",
        "gvy" => "groovy",
        _ => language,
    }
}

#[derive(Debug, Default, Clone)]
pub struct ParserResult {
    pub url: String,
    pub stdin: String,
    pub target: String,
    pub code: String,
    pub options: Vec<String>,
    pub args: Vec<String>,
}

#[allow(clippy::while_let_on_iterator)]
pub async fn get_components(
    input: &str,
    author: &User,
    compilation_manager: Option<&Arc<RwLock<CompilationManager>>>,
    reply: &Option<Box<Message>>,
    ignore_lang: bool,
) -> Result<ParserResult, CommandError> {
    let mut result = ParserResult::default();

    // Find the index for where we should stop parsing user input
    let mut end_point: usize = input.len();
    if let Some(parse_stop) = input.find('\n') {
        end_point = parse_stop;
    }
    if let Some(index) = input.find('`') {
        // if the ` character is found before \n we should use the ` as our parse stop point
        if end_point == 0 || index < end_point {
            end_point = index;
        }
    }
    let mut args: Vec<&str> = input[..end_point].split_whitespace().collect();
    // ditch command str (;compile, ;asm)
    args.remove(0);

    // Check to see if we were given a valid target... if not we'll check
    // the syntax highlighting str later.
    if let Some(comp_mngr) = compilation_manager {
        let lang_lookup = comp_mngr.read().await;
        if let Some(param) = args.first() {
            let lower_param = param.trim().to_lowercase();
            let language = shortname_to_qualified(&lower_param);
            if !matches!(lang_lookup.resolve_target(language), RequestHandler::None) {
                args.remove(0);
                result.target = language.to_owned();
            }
        }
    } else {
        // no compilation manager, just assume target is supplied
        if !ignore_lang {
            if let Some(param) = args.first() {
                let lower_param = param.trim().to_lowercase();
                let language = shortname_to_qualified(&lower_param);
                args.remove(0);
                result.target = language.to_owned();
            }
        }
    }

    // looping every argument
    let mut iter = args.iter();
    while let Some(c) = iter.next() {
        if c.contains('\n') || c.contains('`') {
            break;
        }

        if *c == "<" {
            let link = match iter.next() {
                Some(link) => link,
                None => return Err(CommandError::from("'<' operator requires a url\n\nUsage: `;compile c++ < http://foo.bar/code.txt`"))
            };
            result.url = link.trim().to_string();
        } else if *c == "|" {
            let mut input: String = String::new();
            while let Some(stdin) = iter.next() {
                if stdin.contains("```") {
                    break;
                }
                if *stdin == "<" {
                    return Err(CommandError::from(
                        "`|`` operator should be last, unable to continue",
                    ));
                }
                input.push_str(stdin);
                input.push(' ');
            }

            result.stdin = input.trim().to_owned();
        } else {
            result.options.push(c.trim().to_string());
        }
    }

    let cmdline_args;
    if let Some(codeblock_start) = input.find('`') {
        if end_point < codeblock_start {
            cmdline_args = String::from(input[end_point..codeblock_start].trim());
        } else {
            cmdline_args = String::default();
        }
    } else {
        cmdline_args = String::from(input[end_point..].trim());
    }
    result.args = shell_words::split(&cmdline_args)?;

    if find_code_block(&mut result, input, author).await? {
        if !result.url.is_empty() {
            let code = get_url_code(&result.url, author).await?;
            result.stdin = result.code;
            result.code = code;
        }
    } else if !result.url.is_empty() {
        let code = get_url_code(&result.url, author).await?;
        result.code = code;
    }
    // Unable to parse a code block from our executor's message, lets see if we have a
    // reply to grab some code from.
    else if let Some(replied_msg) = reply {
        let attachment = get_message_attachment(&replied_msg.attachments).await?;
        if !attachment.0.is_empty() {
            if !result.target.is_empty() {
                result.target = attachment.1;
            }
            result.code = attachment.0;
        }
        // no attachment in the reply, lets check for a code-block..
        else if !find_code_block(&mut result, &replied_msg.content, author).await? {
            return Err(CommandError::from(
                "You must attach a code-block containing code to your message or reply to a message that has one.",
            ));
        }
    } else {
        // We were really given nothing, lets fail now.
        return Err(CommandError::from(
            "You must attach a code-block containing code to your message or quote a message that has one.",
        ));
    }

    if result.target.is_empty() {
        return Err(CommandError::from("You must provide a valid language or compiler!\n\n;compile c++ \n\\`\\`\\`\nint main() {}\n\\`\\`\\`"));
    }

    Ok(result)
}

async fn get_url_code(url: &str, author: &User) -> Result<String, CommandError> {
    let url = match reqwest::Url::parse(url) {
        Err(e) => return Err(CommandError::from(format!("Error parsing url: {}", e))),
        Ok(url) => url,
    };

    let host = url.host();
    if host.is_none() {
        return Err(CommandError::from("Unable to find host"));
    }

    let host_str = host.unwrap().to_string();
    if !URL_ALLOW_LIST.contains(&host_str.as_str()) {
        warn!(
            "Blocked URL request to: {} by {} [{}]",
            host_str, author.id, author.name
        );
        return Err(CommandError::from("Unknown paste service. Please use pastebin.com, hastebin.com, or GitHub gists.\n\nAlso please be sure to use a 'raw text' link"));
    }

    let response = match reqwest::get(url).await {
        Ok(b) => b,
        Err(_e) => {
            return Err(CommandError::from(
                "GET request failed, perhaps your link is unreachable?",
            ))
        }
    };

    match response.text().await {
        Ok(t) => Ok(t),
        Err(_e) => Err(CommandError::from("Unable to grab resource")),
    }
}

pub async fn find_code_block(
    result: &mut ParserResult,
    haystack: &str,
    author: &User,
) -> Result<bool, CommandError> {
    let matches = CODE_BLOCK_REGEX.captures_iter(haystack);

    let mut captures = Vec::new();
    let list = matches.enumerate();
    for (_, cap) in list {
        captures.push(cap);
    }

    // support for stdin codeblocks
    let code_index; // index into captures where we might find our target lang
    match captures.len() {
        len if len > 1 => {
            result.stdin = String::from(captures[0].name("code").unwrap().as_str());
            result.code = String::from(captures[1].name("code").unwrap().as_str());

            code_index = 1;
        }
        1 => {
            result.code = String::from(captures[0].name("code").unwrap().as_str());

            code_index = 0;
        }
        _ => return Ok(false),
    }

    let code_copy = result.code.clone();
    let matches = C_LIKE_INCLUDE_REGEX.captures_iter(&code_copy).enumerate();
    for (_, cap) in matches {
        if let Some(statement) = cap.name("statement") {
            let include_stmt = statement.as_str();
            let url = cap.name("url").unwrap().as_str();
            if let Ok(code) = get_url_code(url, author).await {
                debug!("Replacing {} with {}", include_stmt, &code);
                result.code = result.code.replace(include_stmt, &code);
            }
        }
    }

    // if we still don't have our language target, lets try the language for syntax highlighting
    if result.target.is_empty() {
        if let Some(lang_match) = captures[code_index].name("language") {
            result.target = shortname_to_qualified(lang_match.as_str()).to_owned();
        }
    }

    Ok(true)
}

pub async fn get_message_attachment(
    attachments: &[Attachment],
) -> Result<(String, String), CommandError> {
    if !attachments.is_empty() {
        let attachment = attachments.first();
        if attachment.is_none() {
            return Ok((String::new(), String::new()));
        }
        let attached = attachment.unwrap();
        if attached.size > 512 * 1000 {
            // 512 KB seems enough
            return Err(CommandError::from(format!(
                "Uploaded file too large: `{} KB`",
                attached.size / 1000
            )));
        }
        return match reqwest::get(&attached.url).await {
            Ok(r) => {
                let bytes = r.bytes().await.unwrap();
                let cnt_type = content_inspector::inspect(&bytes);
                if cnt_type.is_binary() {
                    return Err(CommandError::from("Invalid file type"));
                }

                match String::from_utf8(bytes.to_vec()) {
                    Ok(str) => {
                        let mut extension = String::from("");
                        if let Some(ext) = Path::new(&attached.filename).extension() {
                            extension = ext.to_string_lossy().to_string();
                        }
                        Ok((str, extension))
                    }
                    Err(e) => Err(CommandError::from(format!(
                        "UTF8 Error occured while parsing file: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(CommandError::from(format!(
                "Failure when downloading attachment: {}",
                e
            ))),
        };
    }
    Ok((String::new(), String::new()))
}
