use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{MessageCache, CompilerCache, ConfigCache, MessageCacheEntry};
use crate::utls::discordhelpers::embeds;
use crate::utls::discordhelpers;
use crate::cppeval::eval::CppEval;
use crate::utls::parser::ParserResult;
use serenity::builder::CreateEmbed;
use crate::utls::discordhelpers::embeds::ToEmbed;

#[command]
#[aliases("c++")]
#[bucket = "nospam"]
pub async fn cpp(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}