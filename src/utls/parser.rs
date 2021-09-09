use crate::utls::constants::URL_ALLOW_LIST;

use serenity::model::user::User;
use serenity::model::channel::{Message, Attachment};
use serenity::framework::standard::CommandError;

use tokio::sync::RwLock;
use std::sync::Arc;

//Traits for compiler lookup
pub trait LanguageResolvable {
    fn resolve(&self, language : &str) -> bool;
}

impl LanguageResolvable for wandbox::Wandbox {
    fn resolve(&self, language : &str) -> bool {
        self.is_valid_language(language) || self.is_valid_compiler_str(language)
    }
}
impl LanguageResolvable for godbolt::Godbolt {
    fn resolve(&self, language : &str) -> bool {
        self.resolve(language).is_some()
    }
}

// Allows us to convert some common aliases to other programming languages
pub fn shortname_to_qualified(language : &str) -> &str {
    match language {
        // Replace cpp with c++ since we removed the c pre-processor
        // support for wandbox. This is okay for godbolt requests, too.
        "cpp" => "c++",
        "rs" => "rust",
        "js" => "javascript",
        "csharp" => "c#",
        "py" => "python",
        _ => language
    }
}

#[derive(Debug, Default)]
pub struct ParserResult {
    pub url: String,
    pub stdin: String,
    pub target: String,
    pub code: String,
    pub options: Vec<String>,
}

#[allow(clippy::while_let_on_iterator)]
pub async fn get_components<T : LanguageResolvable>(input: &str, author : &User, target_api : &Arc<RwLock<T>>, reply : &Option<Box<Message>>) -> Result<ParserResult, CommandError> {

    let mut result = ParserResult::default();

    // we grab the index for the first code block - this will help us
    // know when to stop parsing arguments
    let code_block: usize;
    if let Some(index) = input.find('`') {
        code_block = index;
    } else {
        code_block = input.len();
    }

    let mut args: Vec<&str> = input[..code_block].split_whitespace().collect();

    // ditch command str (;compile, ;asm)
    args.remove(0);

    // Check to see if we were given a valid target... if not we'll check
    // the syntax highlighting str later.
    {
        let lang_lookup = target_api.read().await;
        if let Some(param) = args.get(0) {
            let lower_param = param.trim().to_lowercase();
            let language = shortname_to_qualified(&lower_param);
            if lang_lookup.resolve(&language) {
                args.remove(0);
                result.target = language.to_owned();
            }
        }
    }

    // looping every argument
    let mut iter = args.iter();
    while let Some(c) = iter.next() {
        if c.contains("```") {
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

    if !result.url.is_empty() {
        let code = get_url_code(&result.url, author).await?;
        result.code = code;
    }
    else if find_code_block(&mut result, input) {
        // If we find a code block from our executor's message, and it's also a reply
        // let's assume we found the stdin and what they're quoting is the code.
        // Anything else probably doesn't make sense.
        if let Some(replied_msg) = reply {
            result.stdin = result.code;
            result.code = String::default();

            let attachment = get_message_attachment(&replied_msg.attachments).await?;
            if !attachment.is_empty() {
                result.code = attachment;
            }
            else if !find_code_block(&mut result, &replied_msg.content) {
                return Err(CommandError::from(
                    "Cannot find code to compile assuming your code block is the program's stdin.",
                ))
            }
        }
    }
    else {
        // Unable to parse a code block from our executor's message, lets see if we have a
        // reply to grab some code from.
        if let Some(replied_msg) = reply {
            let attachment = get_message_attachment(&replied_msg.attachments).await?;
            if !attachment.is_empty() {
                result.code = attachment;
            }
            // no reply in the attachment, lets check for a code-block..
            else if !find_code_block(&mut result, &replied_msg.content) {
                return Err(CommandError::from(
                    "You must attach a code-block containing code to your message or quote a message that has one.",
                ))
            }
        }
        else { // We were really given nothing, lets fail now.
            return Err(CommandError::from(
                "You must attach a code-block containing code to your message or quote a message that has one.",
            ))
        }
    }

    if result.target.is_empty() {
        return Err(CommandError::from("You must provide a valid language or compiler!\n\n;compile c++ \n\\`\\`\\`\nint main() {}\n\\`\\`\\`"))
    }

    Ok(result)
}

async fn get_url_code(url : &str, author : &User) -> Result<String, CommandError> {
    let url = match reqwest::Url::parse(url) {
        Err(e) => {
            return Err(CommandError::from(format!("Error parsing url: {}", e)))
        },
        Ok(url) => url
    };

    let host = url.host();
    if host.is_none() {
        return Err(CommandError::from("Unable to find host"))
    }

    let host_str = host.unwrap().to_string();
    if !URL_ALLOW_LIST.contains(&host_str.as_str()) {
        warn!("Blocked URL request to: {} by {} [{}]", host_str, author.id.0, author.tag());
        return Err(CommandError::from("Unknown paste service. Please use pastebin.com, hastebin.com, or GitHub gists.\n\nAlso please be sure to use a 'raw text' link"))
    }

    let response = match reqwest::get(url).await {
        Ok(b) => b,
        Err(_e) => {
            return Err(CommandError::from(
                "GET request failed, perhaps your link is unreachable?",
            ))
        }
    };

    return match response.text().await {
        Ok(t) => Ok(t),
        Err(_e) => Err(CommandError::from("Unable to grab resource")),
    };
}

pub fn find_code_block(result: &mut ParserResult, haystack: &str) -> bool {
    let re = regex::Regex::new(r"```(?:(?P<language>[^\s`]*)\r?\n)?(?P<code>[\s\S]*?)```").unwrap();
    let matches = re.captures_iter(haystack);

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
        _ => {
            return false
        }
    }

    // if we still don't have our language target, lets try the language for syntax highlighting
    if result.target.is_empty() {
        if let Some(lang_match) = captures[code_index].name("language") {
            result.target = shortname_to_qualified(lang_match.as_str()).to_owned();
        }
    }

    true
}

pub async fn get_message_attachment(attachments : & Vec<Attachment>) -> Result<String, CommandError> {
    if !attachments.is_empty() {
        let attachment = attachments.get(0);
        if attachment.is_none() {
            return Ok(String::new());
        }
        let attached = attachment.unwrap();
        if attached.size > 512 * 1024  { // 512 KiB seems enough
            return Err(CommandError::from(format!("Uploaded file too large: `{} MiB`", attached.size)));
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
                        Ok(str)
                    }
                    Err(e) => {
                        Err(CommandError::from(format!("UTF8 Error occured while parsing file: {}", e)))
                    }
                }
            }
            Err(e) => {
                Err(CommandError::from(format!("Failure when downloading attachment: {}", e)))
            }
        }
    }
    Ok(String::new())
}