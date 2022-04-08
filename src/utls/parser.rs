use crate::utls::constants::URL_ALLOW_LIST;

use serenity::{
    model::user::User,
    framework::standard::CommandError
};

// Allows us to convert some common aliases to other programming languages
pub fn shortname_to_qualified(language : &str) -> &str {
    match language {
        // Replace cpp with c++ since we removed the c pre-processor
        // support for wandbox. This is okay for godbolt requests, too.
        "cpp" => "c++",
        "rs" => "rust",
        "js" => "javascript",
        "csharp" => "c#",
        "cs" => "c#",
        "py" => "python",
        _ => language
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

#[allow(dead_code)]
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
