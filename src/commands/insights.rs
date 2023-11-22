use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::apis::insights::InsightsRequest;

use crate::cache::{ConfigCache, InsightsAPICache};
use crate::utls::constants::{COLOR_FAIL, COLOR_OKAY};
use crate::utls::parser::ParserResult;
use crate::utls::{discordhelpers, parser};

#[command]
pub async fn insights(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data_read = ctx.data.read().await;
    let insights_lock = data_read.get::<InsightsAPICache>().unwrap();
    let botinfo_lock = data_read.get::<ConfigCache>().unwrap();
    let botinfo = botinfo_lock.read().await;

    let loading_reaction = {
        if let Some(loading_id) = botinfo.get("LOADING_EMOJI_ID") {
            let loading_name = botinfo
                .get("LOADING_EMOJI_NAME")
                .expect("Unable to find loading emoji name")
                .clone();
            discordhelpers::build_reaction(loading_id.parse::<u64>()?, &loading_name)
        } else {
            ReactionType::Unicode(String::from("⏳"))
        }
    };

    let mut parse_result = ParserResult::default();
    if !parser::find_code_block(&mut parse_result, &msg.content, &msg.author).await? {
        return Err(CommandError::from("Unable to find a codeblock to format!"));
    }
    let req = InsightsRequest {
        code: parse_result.code,
        insights_options: vec![String::from("cpp2c")], // hard coded version for now
    };

    if msg
        .react(&ctx.http, loading_reaction.clone())
        .await
        .is_err()
    {
        return Err(CommandError::from(
            "Unable to react to message, am I missing permissions to react or use external emoji?",
        ));
    }

    let resp = {
        let insights = insights_lock.lock().await;
        insights.get_insights(req).await
    };

    discordhelpers::delete_bot_reacts(ctx, msg, loading_reaction).await?;

    if let Some(resp_obj) = resp {
        let error = resp_obj.return_code != 0;
        let sent = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|emb| {
                    emb.color(if error { COLOR_FAIL } else { COLOR_OKAY })
                        .title("cppinsights.io")
                        .description(format!(
                            "```cpp\n{}\n```",
                            if error {
                                resp_obj.stderr
                            } else {
                                resp_obj.stdout
                            }
                        ))
                })
            })
            .await;

        if let Ok(sent_msg) = sent {
            discordhelpers::send_completion_react(ctx, &sent_msg, !error).await?;
        }
    } else {
        return Err(CommandError::from(
            "Unable to retrieve insights at this time! Please try again later.",
        ));
    }

    debug!("Command executed");
    Ok(())
}
