use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::Permissions;

pub const COLOR_OKAY: u32 = 0x5dbcd2;
pub const COLOR_FAIL: u32 = 0xff7761;
pub const COLOR_WARN: u32 = 0xad7805;

pub const ICON_FAIL: &str = "https://i.imgur.com/LxxYrFj.png";
pub const ICON_HELP: &str = "https://i.imgur.com/TNzxfMB.png";
pub const ICON_INVITE: &str = "https://i.imgur.com/CZFt69d.png";
pub const COMPILER_ICON: &str = "https://compiler.gg/icon.png";
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
pub const MAX_URL_CODE_LEN: usize = 512 * 1000;
pub const MAX_FILTER_LEN: usize = 64;

/// Discord's error code for "Missing Permissions"
pub const MISSING_PERMISSIONS_CODE: isize = 50013;

/// Everything the bot needs in a channel to run its commands
pub const REQUIRED_PERMISSIONS: Permissions = Permissions::VIEW_CHANNEL
    .union(Permissions::SEND_MESSAGES)
    .union(Permissions::EMBED_LINKS)
    .union(Permissions::ATTACH_FILES)
    .union(Permissions::ADD_REACTIONS)
    .union(Permissions::USE_EXTERNAL_EMOJIS)
    .union(Permissions::READ_MESSAGE_HISTORY);

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
