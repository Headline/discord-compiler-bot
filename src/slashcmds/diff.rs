use serenity::{
    framework::standard::CommandResult,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::application_command::ApplicationCommandInteractionDataOptionValue,
    model::prelude::*, prelude::*,
};

use similar::ChangeTag;

use crate::{
    utls::constants::COLOR_FAIL, utls::constants::COLOR_OKAY, utls::parser::find_code_block,
    utls::parser::ParserResult,
};

pub async fn diff(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {
    let message1 = msg
        .data
        .options
        .get(0)
        .expect("Expected interaction option 0")
        .resolved
        .as_ref()
        .expect("Expected data option value");

    let message2 = msg
        .data
        .options
        .get(1)
        .expect("Expected interaction option 1")
        .resolved
        .as_ref()
        .expect("Expected data option value");

    let mut message1_parse = None;
    if let ApplicationCommandInteractionDataOptionValue::String(input) = message1 {
        message1_parse = input.parse::<u64>().ok();
    }
    let mut message2_parse = None;
    if let ApplicationCommandInteractionDataOptionValue::String(input) = message2 {
        message2_parse = input.parse::<u64>().ok();
    }

    if message1_parse.is_none() || message2_parse.is_none() {
        msg.create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|emb| {
                    emb.color(COLOR_FAIL).description(
                        "Invalid message ID specified!\n\n\
                        Right click a message and select 'Copy ID' at the bottom. If you cannot \
                        see this option then you must first enable Developer Mode by going to the \
                        User Settings > Advanced tab",
                    )
                })
                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        })
        .await?;
        return Ok(());
    }

    let message1_obj = ctx
        .http
        .get_message(msg.channel_id.0, message1_parse.unwrap())
        .await
        .ok();
    let message2_obj = ctx
        .http
        .get_message(msg.channel_id.0, message2_parse.unwrap())
        .await
        .ok();
    if message1_obj.is_none() || message2_obj.is_none() {
        msg.create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|emb| {
                    emb.color(COLOR_FAIL).description(
                        "Unable to find message.\n\nEnsure both messages belong to \
                        this channel and the Message IDs are correct.",
                    )
                })
                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        })
        .await?;
        return Ok(());
    }

    let content1 = message1_obj.unwrap().content;
    let content2 = message2_obj.unwrap().content;

    let mut fake_parse1 = ParserResult::default();
    let mut fake_parse2 = ParserResult::default();

    {
        let block1 = find_code_block(&mut fake_parse1, &content1, &msg.user);
        let block2 = find_code_block(&mut fake_parse2, &content2, &msg.user);

        if !block1.await? || !block2.await? {
            msg.create_interaction_response(&ctx.http, |resp| {
                resp.interaction_response_data(|data| {
                    data.embed(|emb| {
                        emb.color(COLOR_FAIL).description(
                            "Unable to find a code-block to match within a supplied message!",
                        )
                    })
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            })
            .await?;
            return Ok(());
        }
    }

    let code1 = fake_parse1.code.clone();
    let code2 = fake_parse2.code.clone();
    let diff = similar::TextDiff::from_lines(&code1, &code2);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }

    msg.create_interaction_response(&ctx.http, |resp| {
        resp.interaction_response_data(|data| {
            data.embed(|emb| {
                emb.color(COLOR_OKAY)
                    .title("Diff completed")
                    .description(format!("```diff\n{}\n```", output))
            })
        })
    })
    .await?;

    Ok(())
}
