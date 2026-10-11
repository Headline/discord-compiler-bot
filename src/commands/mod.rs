pub mod asm;
pub mod asmdoc;
pub mod block;
pub mod botinfo;
pub mod compile;
pub mod compilers;
pub mod cpp;
pub mod execute;
pub mod format;
pub mod formats;
pub mod help;
pub mod insights;
pub mod invite;
pub mod languages;
pub mod libraries;
pub mod ping;
pub mod unblock;

use crate::framework::Command;

// adapts an async command function into a framework::CommandFn
macro_rules! run {
    ($f:path) => {
        |ctx, msg, args| Box::pin($f(ctx, msg, args))
    };
}

pub static COMMANDS: &[Command] = &[
    Command {
        names: &["botinfo"],
        owners_only: false,
        rate_limited: false,
        run: run!(botinfo::botinfo),
    },
    Command {
        names: &["compile"],
        owners_only: false,
        rate_limited: true,
        run: run!(compile::compile),
    },
    Command {
        names: &["execute", "exec", "exe"],
        owners_only: false,
        rate_limited: true,
        run: run!(execute::execute),
    },
    Command {
        names: &["languages"],
        owners_only: false,
        rate_limited: false,
        run: run!(languages::languages),
    },
    Command {
        names: &["compilers"],
        owners_only: false,
        rate_limited: true,
        run: run!(compilers::compilers),
    },
    Command {
        names: &["libraries"],
        owners_only: false,
        rate_limited: true,
        run: run!(libraries::libraries),
    },
    Command {
        names: &["ping"],
        owners_only: false,
        rate_limited: false,
        run: run!(ping::ping),
    },
    Command {
        names: &["help"],
        owners_only: false,
        rate_limited: false,
        run: run!(help::help),
    },
    Command {
        names: &["asm"],
        owners_only: false,
        rate_limited: true,
        run: run!(asm::asm),
    },
    Command {
        names: &["asmdoc"],
        owners_only: false,
        rate_limited: true,
        run: run!(asmdoc::asmdoc),
    },
    Command {
        names: &["block"],
        owners_only: true,
        rate_limited: false,
        run: run!(block::block),
    },
    Command {
        names: &["unblock"],
        owners_only: true,
        rate_limited: false,
        run: run!(unblock::unblock),
    },
    Command {
        names: &["invite"],
        owners_only: false,
        rate_limited: false,
        run: run!(invite::invite),
    },
    Command {
        names: &["cpp", "c++"],
        owners_only: false,
        rate_limited: true,
        run: run!(cpp::cpp),
    },
    Command {
        names: &["formats"],
        owners_only: false,
        rate_limited: false,
        run: run!(formats::formats),
    },
    Command {
        names: &["format"],
        owners_only: false,
        rate_limited: false,
        run: run!(format::format),
    },
    Command {
        names: &["insights"],
        owners_only: false,
        rate_limited: false,
        run: run!(insights::insights),
    },
];
