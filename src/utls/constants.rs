

//pub const COLOR_OKAY : i32 = 0x046604;
pub const COLOR_OKAY: u32 = 0x5dbcd2;
//pub const COLOR_FAIL : i32 = 0x660404;
pub const COLOR_FAIL: u32 = 0xff7761;
pub const ICON_FAIL: &str = "https://i.imgur.com/LxxYrFj.png";
pub const ICON_VOTE: &str = "https://i.imgur.com/VXbdwSQ.png";
pub const ICON_HELP: &str = "https://i.imgur.com/TNzxfMB.png";
pub const ICON_INVITE: &str = "https://i.imgur.com/CZFt69d.png";
//pub const COMPILER_EXPLORER_ICON: &str = "https://i.imgur.com/GIgATFr.png";
pub const COMPILER_ICON: &str = "http://i.michaelwflaherty.com/u/XedLoQWCVc.png";
pub const MAX_OUTPUT_LEN: usize = 250;
pub const MAX_ERROR_LEN: usize = 500;
pub const USER_AGENT : &str = const_format::formatcp!("discord-compiler-bot/{}", env!("CARGO_PKG_VERSION"));
pub const URL_ALLOW_LIST : [&str; 4] = ["pastebin.com", "gist.githubusercontent.com", "hastebin.com", "raw.githubusercontent.com"];