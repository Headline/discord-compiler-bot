use std::error::Error;
use std::fmt;

use crate::utls::constants::URL_ALLOW_LIST;
use serenity::model::user::User;

#[derive(Debug)]
pub struct ParserError {
    details: String,
}
impl ParserError {
    fn new(msg: &str) -> ParserError {
        ParserError {
            details: msg.to_string(),
        }
    }
}
impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl Error for ParserError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct ParserResult {
    pub url: String,
    pub stdin: String,
    pub target: String,
    pub code: String,
    pub options: Vec<String>,
}

#[allow(clippy::while_let_on_iterator)]
pub async fn get_components(input: &str, author : &User) -> Result<ParserResult, ParserError> {
    let mut result = ParserResult {
        url: Default::default(),
        stdin: Default::default(),
        target: Default::default(),
        code: Default::default(),
        options: Default::default(),
    };

    // we grab the index for the first code block - this will help us
    // know when to stop parsing arguments
    let code_block: usize;
    if let Some(index) = input.find('`') {
        code_block = index;
    } else {
        code_block = input.len();
    }

    let mut args: Vec<&str> = input[..code_block].split(' ').collect();

    // ditch command str (;compile, ;asm)
    args.remove(0);

    if args.is_empty() {
        return Err(ParserError::new("You must provide a valid language or compiler!\n\n;compile c++ \n\\`\\`\\`\nint main() {}\n\\`\\`\\`"));
    }
    result.target = args.remove(0).trim().to_owned().to_lowercase();

    // Replace cpp with c++ since we removed the c pre-processor
    // support for wandbox. This is okay for godbolt requests, too.
    if result.target == "cpp" {
        result.target = String::from("c++");
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
                None => return Err(ParserError::new("'<' operator requires a url\n\nUsage: `;compile c++ < http://foo.bar/code.txt`"))
            };
            result.url = link.trim().to_string();
        } else if *c == "|" {
            let mut input: String = String::new();
            while let Some(stdin) = iter.next() {
                if stdin.contains("```") {
                    break;
                }
                if *stdin == "<" {
                    return Err(ParserError::new(
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
        let url = match reqwest::Url::parse(&result.url) {
            Err(e) => {
                return Err(ParserError::new(&format!("Error parsing url: {}", e)))
            },
            Ok(url) => url
        };

        let host = url.host().unwrap().to_string();
        if !URL_ALLOW_LIST.contains(&host.as_str()) {
            warn!("Blocked URL request to: {} by {} [{}]", host, author.id.0, author.tag());
            return Err(ParserError::new("Unknown paste service. Please use pastebin.com, hastebin.com, or GitHub gists.\n\nAlso please be sure to use a 'raw text' link"))
        }

        let response = match reqwest::get(&result.url).await {
            Ok(b) => b,
            Err(_e) => {
                return Err(ParserError::new(
                    "GET request failed, perhaps your link is unreachable?",
                ))
            }
        };

        let body = match response.text().await {
            Ok(t) => t,
            Err(_e) => return Err(ParserError::new("Unable to grab resource")),
        };

        result.code = body;
    } else {
        find_code_block(&mut result, input)?;
    }

    Ok(result)
}

fn find_code_block(result: &mut ParserResult, haystack: &str) -> Result<(), ParserError> {
    let re = regex::Regex::new(r"```(?:(?P<language>[^\s`]*)\r?\n)?(?P<code>[\s\S]*?)```").unwrap();
    let matches = re.captures_iter(haystack);

    let mut captures: Vec<&str> = Vec::new();
    let list = matches.enumerate();
    for (_, cap) in list {
        captures.push(cap.name("code").unwrap().as_str());
    }

    // support for stdin codeblocks
    // support for stdin codeblocks
    match captures.len() {
        len if len > 1 => {
            result.code = String::from(captures[1]);
            result.stdin = String::from(captures[0]);
        }
        1 => {
            result.code = String::from(captures[0]);
        }
        _ => {
            return Err(ParserError::new(
                "You must attach a code-block containing code to your message",
            ))
        }
    }
    Ok(())
}
