use serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::CompilerCache;
use crate::utls::constants::COLOR_OKAY;
use crate::utls::discordhelpers::embeds;

#[command]
#[bucket = "nospam"]
pub async fn asmdoc(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let opcode = args
        .single::<String>()
        .map_err(|_| {
            CommandError::from(
                "Please supply an opcode to look up, i.e. `;asmdoc mov` or `;asmdoc adds arm32`",
            )
        })?
        .to_lowercase();

    // Optional second argument selects the instruction set
    let instruction_set = if args.is_empty() {
        String::from("amd64")
    } else {
        args.single::<String>()?.to_lowercase()
    };

    let data = ctx.data.read().await;
    let compiler_manager = data.get::<CompilerCache>().unwrap().read().await;
    let godbolt = compiler_manager.godbolt().ok_or_else(|| {
        CommandError::from("Compiler Explorer service is currently down, please try again later.")
    })?;

    let doc = godbolt
        .asm_doc(&instruction_set, &opcode)
        .await
        .map_err(|_| {
            CommandError::from(format!(
                "Unable to find documentation for opcode '{}' on instruction set '{}'.",
                opcode, instruction_set
            ))
        })?;

    let mut description = doc.tooltip.trim().to_string();
    if description.is_empty() {
        description = String::from("No description available.");
    }
    if description.len() > 2000 {
        description = description.chars().take(2000).collect();
        description.push('…');
    }

    let footer = CreateEmbedFooter::new(format!(
        "Requested by: {} | Powered by godbolt.org",
        msg.author.name
    ));
    let mut emb = CreateEmbed::new()
        .color(COLOR_OKAY)
        .title(format!("{} ({})", opcode.to_uppercase(), instruction_set))
        .description(description)
        .footer(footer);
    if let Some(url) = &doc.url {
        emb = emb.url(url);
    }

    embeds::dispatch_embed(&ctx.http, msg.channel_id, emb).await?;

    debug!("Command executed");
    Ok(())
}
