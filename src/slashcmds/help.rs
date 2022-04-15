use serenity::{
    framework::standard::{CommandResult},
    client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::prelude::message_component::ButtonStyle
};

use crate::{
    cache::ConfigCache,
    utls::constants::*
};

pub async fn help(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {
    let data = ctx.data.read().await;
    let botinfo = data.get::<ConfigCache>().unwrap().read().await;
    let invite_link = botinfo.get("INVITE_LINK").unwrap();
    let dbl_link = botinfo.get("DISCORDBOTS_LINK").unwrap();
    let github_link = botinfo.get("GITHUB_LINK").unwrap();
    let stats_link = botinfo.get("STATS_LINK").unwrap();
    msg.create_interaction_response(&ctx.http, |resp| {
        resp.interaction_response_data(|data| {
            data
                .embed(|emb| {
                emb.color(COLOR_OKAY)
                    .description("Hello! I can compile code for you. To compile code, \
                    first post a code block containing code, right click the message, \
                    go to the Apps dropdown, and select the Compile option!")
                    .thumbnail(ICON_HELP)
                })

                .embed(|emb| {
                emb.color(COLOR_WARN)
                    .description("If you are unfamiliar with Markdown, codeblocks can be created by \
                                 formatting your message as the following.\n\
                                 \\`\\`\\`\n\
                                 <code>\n\
                                 \\`\\`\\`")
                })
                .components(|components| {
                    components.create_action_row(|row| {
                        row
                            .create_button(|btn| {
                                btn.label("Invite me")
                                    .style(ButtonStyle::Link)
                                    .url(invite_link)
                            })
                            .create_button(|btn| {
                                btn.label("Vote for us")
                                    .style(ButtonStyle::Link)
                                    .url(dbl_link)
                            })
                            .create_button(|btn| {
                                btn.label("GitHub")
                                    .style(ButtonStyle::Link)
                                    .url(github_link)
                            })
                            .create_button(|btn| {
                                btn.label("Stats")
                                    .style(ButtonStyle::Link)
                                    .url(stats_link)
                            })
                    })
                })
        })
    }).await?;
    debug!("Command executed");
    Ok(())
}
