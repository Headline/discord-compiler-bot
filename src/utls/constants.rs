

//pub const COLOR_OKAY : i32 = 0x046604;
pub const COLOR_OKAY: u32 = 0x5dbcd2;
//pub const COLOR_FAIL : i32 = 0x660404;
pub const COLOR_FAIL: u32 = 0xff7761;
pub const COLOR_WARN: u32 = 0xad7805;

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


/*
    Discord limits the size of the amount of compilers we can display to users, for some languages
    we'll just grab the first 25 from our API, for C & C++ we will create a curated list manually.

    If you'd like to see a change here feel free to pr to remove one, but justify it's removal.
 */
pub const CPP_ASM_COMPILERS : [[&str; 2]; 25] = [
    ["x86-64 clang (trunk)", "clang_trunk"],
    ["x86-64 clang 13.0.1", "clang1301"],
    ["x86-64 clang 14.0.0", "clang1400"],
    ["x86-64 clang 3.4.1", "clang341"],
    ["x86-64 clang 3.8", "clang380"],
    ["x86-64 clang 6.0.1", "clang601"],
    ["x86-64 clang 7.1.0", "clang710"],
    ["x86-64 clang 8.0.1", "clang801"],
    ["x86-64 clang 9.0.1", "clang901"],
    ["x86-64 gcc (trunk)", "gsnapshot"],
    ["x86-64 gcc 11.2", "g112"],
    ["x86-64 gcc 4.9.4", "g494"],
    ["x86-64 gcc 5.5", "g550"],
    ["x86-64 gcc 7.5", "g75"],
    ["x86-64 gcc 9.1", "g91"],
    ["x86-64 gcc 9.4", "g94"],
    ["x86 msvc v19.31", "vcpp_v19_31_x86"],
    ["x64 msvc v19.31", "vcpp_v19_31_x64"],
    ["ARM gcc 9.3 (linux)", "arm930"],
    ["ARM64 gcc 9.3", "arm64g930"],
    ["arm64 msvc v19.31", "vcpp_v19_31_arm64"],
    ["armv7-a clang 11.0.1", "armv7-clang1101"],
    ["armv8-a clang 10.0.1", "armv8-clang1001"],
    ["mips gcc 11.2.0", "mips1120"],
    ["mips64 gcc 11.2.0", "mips112064"],
];

pub const CPP_EXEC_COMPILERS : [[&str; 2]; 18] = [
    ["x86-64 clang (trunk)", "clang_trunk"],
    ["x86-64 clang 13.0.1", "clang1301"],
    ["x86-64 clang 14.0.0", "clang1400"],
    ["x86-64 clang 3.4.1", "clang341"],
    ["x86-64 clang 3.8", "clang380"],
    ["x86-64 clang 6.0.1", "clang601"],
    ["x86-64 clang 7.1.0", "clang710"],
    ["x86-64 clang 8.0.1", "clang801"],
    ["x86-64 clang 9.0.1", "clang901"],
    ["x86-64 gcc (trunk)", "gsnapshot"],
    ["x86-64 gcc 11.2", "g112"],
    ["x86-64 gcc 4.9.4", "g494"],
    ["x86-64 gcc 5.5", "g550"],
    ["x86-64 gcc 7.5", "g75"],
    ["x86-64 gcc 9.1", "g91"],
    ["x86-64 gcc 9.4", "g94"],
    ["x86 msvc v19.31", "vcpp_v19_31_x86"],
    ["x64 msvc v19.31", "vcpp_v19_31_x64"],
];

pub const C_ASM_COMPILERS : [[&str; 2]; 23] = [
    ["x86-64 clang (trunk)", "cclang_trunk"],
    ["x86-64 clang 13.0.1", "cclang1301"],
    ["x86-64 clang 14.0.0", "cclang1400"],
    ["x86-64 clang 3.4.1", "cclang341"],
    ["x86-64 clang 3.8", "cclang380"],
    ["x86-64 clang 3.8.1", "cclang381"],
    ["x86-64 clang 6.0.1", "cclang601"],
    ["x86-64 clang 7.1.0", "cclang710"],
    ["x86-64 clang 8.0.1", "cclang801"],
    ["x86-64 clang 9.0.1", "cclang901"],
    ["x86-64 gcc (trunk)", "cgsnapshot"],
    ["x86-64 gcc 11.2", "cg112"],
    ["x86-64 gcc 4.9.4", "cg494"],
    ["x86-64 gcc 5.4", "cg540"],
    ["x86-64 gcc 7.4", "cg74"],
    ["x86-64 gcc 9.1", "cg91"],
    ["x86-64 gcc 9.4", "cg94"],
    ["ARM gcc 9.3 (linux)", "carm930"],
    ["ARM64 gcc 9.3", "carm64g930"],
    ["armv7-a clang 11.0.0", "armv7-cclang1100"],
    ["armv8-a clang 10.0.1", "armv8-cclang1001"],
    ["mips gcc 11.2.0", "cmips1120"],
    ["mips64 gcc 11.2.0", "cmips112064"],
];

pub const C_EXEC_COMPILERS : [[&str; 2]; 23] = [
    ["x86-64 clang (trunk)", "cclang_trunk"],
    ["x86-64 clang 13.0.1", "cclang1301"],
    ["x86-64 clang 14.0.0", "cclang1400"],
    ["x86-64 clang 3.4.1", "cclang341"],
    ["x86-64 clang 3.8", "cclang380"],
    ["x86-64 clang 3.8.1", "cclang381"],
    ["x86-64 clang 6.0.1", "cclang601"],
    ["x86-64 clang 7.1.0", "cclang710"],
    ["x86-64 clang 8.0.1", "cclang801"],
    ["x86-64 clang 9.0.1", "cclang901"],
    ["x86-64 gcc (trunk)", "cgsnapshot"],
    ["x86-64 gcc 11.2", "cg112"],
    ["x86-64 gcc 4.9.4", "cg494"],
    ["x86-64 gcc 5.4", "cg540"],
    ["x86-64 gcc 7.4", "cg74"],
    ["x86-64 gcc 9.1", "cg91"],
    ["x86-64 gcc 9.4", "cg94"],
    ["ARM gcc 9.3 (linux)", "carm930"],
    ["ARM64 gcc 9.3", "carm64g930"],
    ["armv7-a clang 11.0.0", "armv7-cclang1100"],
    ["armv8-a clang 10.0.1", "armv8-cclang1001"],
    ["mips gcc 11.2.0", "cmips1120"],
    ["mips64 gcc 11.2.0", "cmips112064"],
];