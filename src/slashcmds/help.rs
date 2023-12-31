use serenity::all::{
    CommandInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;

use crate::{cache::ConfigCache, utls::constants::*};

pub async fn help(ctx: &Context, msg: &CommandInteraction) -> CommandResult {
    let data = ctx.data.read().await;
    let botinfo = data.get::<ConfigCache>().unwrap().read().await;
    let invite_link = botinfo.get("INVITE_LINK").unwrap();
    let dbl_link = botinfo.get("DISCORDBOTS_LINK").unwrap();
    let github_link = botinfo.get("GITHUB_LINK").unwrap();
    let stats_link = botinfo.get("STATS_LINK").unwrap();

    let hello_embed = CreateEmbed::new()
        .color(COLOR_OKAY)
        .description(format!(
            "Hello! I can compile code for you. To compile code, \
            use the `{0}compile` command. type `{0}help compile` for more information.",
            botinfo.get("BOT_PREFIX").unwrap()
        ))
        .thumbnail(ICON_HELP);

    let markdown_embed = CreateEmbed::new().color(COLOR_WARN).description(
        "If you are unfamiliar with Markdown, codeblocks can be created by \
         formatting your message as the following.\n\
         \\`\\`\\`\n\
         <code>\n\
         \\`\\`\\`",
    );

    let buttons = vec![
        CreateButton::new_link(invite_link).label("Invite me"),
        CreateButton::new_link(dbl_link).label("Vote for us"),
        CreateButton::new_link(github_link).label("GitHub"),
        CreateButton::new_link(stats_link).label("Stats"),
    ];

    let response = CreateInteractionResponseMessage::new()
        .add_embed(hello_embed)
        .add_embed(markdown_embed)
        .components(vec![CreateActionRow::Buttons(buttons)]);

    msg.create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await?;
    debug!("Command executed");
    Ok(())
}
