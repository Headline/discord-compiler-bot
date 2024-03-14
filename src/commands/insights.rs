use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::apis::insights::InsightsRequest;

use crate::cache::{ConfigCache, InsightsAPICache, MessageCache, MessageCacheEntry};
use crate::managers::compilation::CompilationDetails;
use crate::utls::discordhelpers::embeds::build_insights_response_embed;
use crate::utls::{discordhelpers, parser};

#[command]
pub async fn insights(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let (_details, embed) =
        handle_request(ctx.clone(), msg.content.clone(), msg.author.clone(), msg).await?;
    if let Ok(sent_msg) =
        discordhelpers::embeds::dispatch_embed(&ctx.http, msg.channel_id, embed).await
    {
        // add delete cache
        let data_read = ctx.data.read().await;
        let mut delete_cache = data_read.get::<MessageCache>().unwrap().lock().await;
        delete_cache.insert(msg.id.get(), MessageCacheEntry::new(sent_msg, msg.clone()));
    }

    debug!("Command executed");
    Ok(())
}

pub async fn handle_request(
    ctx: Context,
    content: String,
    author: User,
    msg: &Message,
) -> std::result::Result<(CompilationDetails, CreateEmbed), CommandError> {
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

    let parse_result =
        parser::get_components(&content, &author, None, &msg.referenced_message, true).await?;

    let req = InsightsRequest {
        code: parse_result.code,
        insights_options: if parse_result.options.is_empty() {
            vec![String::from("cpp20"), String::from("use-libcpp")]
        } else {
            parse_result.options
        },
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
    discordhelpers::delete_bot_reacts(&ctx, msg, loading_reaction).await?;

    if let Some(resp_obj) = resp {
        debug!("Insights response retval: {}", resp_obj.return_code);
        let details = CompilationDetails {
            language: "".to_string(),
            compiler: "".to_string(),
            base64: None,
            success: resp_obj.return_code == 0,
        };
        Ok((details, build_insights_response_embed(&author, resp_obj)))
    } else {
        Err(CommandError::from(
            "Unable to retrieve insights at this time! Please try again later.",
        ))
    }
}
