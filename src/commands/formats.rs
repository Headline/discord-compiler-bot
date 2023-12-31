use std::fmt::Write as _;

use crate::cache::{CompilerCache, ConfigCache};
use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utls::constants::{COLOR_OKAY, ICON_HELP};
use crate::utls::discordhelpers::embeds;

#[command]
pub async fn formats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let prefix = {
        let botinfo_lock = data
            .get::<ConfigCache>()
            .expect("Expected BotInfo in global cache")
            .clone();
        let botinfo = botinfo_lock.read().await;
        botinfo.get("BOT_PREFIX").unwrap().clone()
    };

    let compiler_manager = data.get::<CompilerCache>().unwrap().read().await;
    if compiler_manager.gbolt.is_none() {
        return Err(CommandError::from(
            "Compiler Explorer service is currently down, please try again later.",
        ));
    }

    let mut emb = CreateEmbed::new()
        .thumbnail(ICON_HELP)
        .color(COLOR_OKAY)
        .title("Formatters:")
        .description(format!("Below is the list of all formatters currently supported, an valid example request can be `{}format rust`, or `{}format clang mozilla`", prefix, prefix));
    for format in &compiler_manager.gbolt.as_ref().unwrap().formats {
        let mut output = String::new();
        output.push_str("Styles:\n");
        if format.styles.is_empty() {
            output.push_str("    *(None)*\n");
        }
        for style in &format.styles {
            // output.push_str(&format!("    *- {}*\n", style));
            writeln!(output, "    *- {}*", style).unwrap();
        }
        emb = emb.field(&format.format_type, &output, false);
    }

    embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await?;

    return Ok(());
}
