use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::cache::{MessageCache, CompilerCache, ConfigCache, MessageCacheEntry};
use crate::utls::discordhelpers::embeds;
use crate::utls::discordhelpers;
use crate::cppeval::eval::CppEval;
use crate::utls::parser::ParserResult;
use serenity::builder::CreateEmbed;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use crate::utls::constants::{COLOR_OKAY, COLOR_WARN};
use crate::utls::discordhelpers::embeds::ToEmbed;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;

pub async fn cpp(ctx: &Context, msg: &ApplicationCommandInteraction) -> CommandResult {
    if msg.data.options.is_empty() {
        msg.create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|emb| {
                    emb
                        .description("You are seeing this message because you haven't specified an input.\n\n \
                           This coomand allows you to quickly compile and execute c++ snippets using geordi-like syntax.\nSee section 2.1 of http://eel.is/geordi/#syntax")
                        .title("/cpp command")
                        .color(COLOR_OKAY)
                        .field("Example 1", "{{ int a = 4; if (a > 3) {{ cout << \"true\"; }} }}", false)
                        .field("Example 2", "<< (4*12) << \"Hello world!\"", false)
                        .field("Example 3", "<< f(2); int f(int a) {{ return a*12; }}", false)
                        .field("Example 4", "int main() {{ cout << \"Main\"; f(); }} void f() {{ cout << \"f()\"; }}", false)
                })
            })
        }).await?;
        return Ok(())
    }
    msg.create_interaction_response(&ctx.http, |resp| {
        resp.kind(InteractionResponseType::DeferredChannelMessageWithSource)
    }).await?;

    let geordi_input = msg.data.options
        .get(0)
        .expect("Expected interaction option 0")
        .resolved
        .as_ref()
        .expect("Expected data option value");

    if let ApplicationCommandInteractionDataOptionValue::String(input) = geordi_input {
        let mut eval = CppEval::new(input);
        let out = eval.evaluate()?;

        let fake_parse = ParserResult {
            url: "".to_string(),
            stdin: "".to_string(),
            target: "g101".to_string(),
            code: out,
            options: vec![String::from("-O2"), String::from("-std=gnu++2a")],
            args: vec![]
        };

        let data_read = ctx.data.read().await;
        let compiler_lock = data_read.get::<CompilerCache>().unwrap().read().await;
        let result = compiler_lock.compiler_explorer(&fake_parse).await?;

        msg.edit_original_interaction_response(&ctx.http, |resp| {
            resp.add_embed(result.to_embed(&msg.user, false))
        }).await?;
    }
    Ok(())
}