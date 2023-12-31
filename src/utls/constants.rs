use lazy_static::lazy_static;
use regex::Regex;

pub const COLOR_OKAY: u32 = 0x5dbcd2;
pub const COLOR_FAIL: u32 = 0xff7761;
pub const COLOR_WARN: u32 = 0xad7805;

pub const ICON_FAIL: &str = "https://i.imgur.com/LxxYrFj.png";
pub const ICON_VOTE: &str = "https://i.imgur.com/VXbdwSQ.png";
pub const ICON_HELP: &str = "https://i.imgur.com/TNzxfMB.png";
pub const ICON_INVITE: &str = "https://i.imgur.com/CZFt69d.png";
pub const COMPILER_ICON: &str = "http://i.michaelwflaherty.com/u/XedLoQWCVc.png";
pub const USER_AGENT: &str =
    const_format::formatcp!("discord-compiler-bot/{}", env!("CARGO_PKG_VERSION"));
pub const URL_ALLOW_LIST: [&str; 4] = [
    "pastebin.com",
    "gist.githubusercontent.com",
    "hastebin.com",
    "raw.githubusercontent.com",
];

pub const MAX_OUTPUT_LEN: usize = 250;
pub const MAX_ERROR_LEN: usize = 997;

// Boilerplate Regexes
lazy_static! {
    pub static ref JAVA_MAIN_REGEX: Regex =
        Regex::new("\"[^\"\n]*?\"|(?P<main>void[\\s]+?main[\\s]*?\\()").unwrap();
    pub static ref C_LIKE_MAIN_REGEX: Regex =
        Regex::new("\"[^\"\n]*?\"|(?P<main>main[\\s]*?\\()").unwrap();
    pub static ref CSHARP_MAIN_REGEX: Regex =
        Regex::new("\"[^\"\n]*?\"|(?P<main>static[\\s]+?void[\\s]+?Main[\\s]*?\\()").unwrap();
    pub static ref PHP_START_REGEX: Regex =
        Regex::new("\"[^\"\n]*?\"|(?P<php_start><\\?php)").unwrap();
}

// Other Regexes
lazy_static! {
    pub static ref GEORDI_MAIN_REGEX: Regex =
        Regex::new(r"(([a-zA-Z]*?)[\s]+main\((.*?)\)[\s]+\{[\s\S]*?\})").unwrap();
    pub static ref JAVA_PUBLIC_CLASS_REGEX: Regex =
        Regex::new("\"[^\"]*?\"|(?P<public>public)[\\s]+?class[\\s]*?").unwrap();
    pub static ref C_LIKE_INCLUDE_REGEX: Regex =
        Regex::new("\"[^\"]+\"|(?P<statement>#include\\s<(?P<url>.+?)>)").unwrap();
    pub static ref CODE_BLOCK_REGEX: Regex =
        Regex::new(r"```(?:(?P<language>[^\s`]*)\r?\n)?(?P<code>[\s\S]*?)```").unwrap();
}
