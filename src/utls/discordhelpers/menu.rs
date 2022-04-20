use std::time::Duration;
use futures_util::StreamExt;
use serenity::builder::{CreateComponents, CreateEmbed};
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use serenity::model::interactions::message_component::ButtonStyle;

pub struct Menu {
    ctx : Context,
    msg : Message,
    pages : Vec<CreateEmbed>,
    page : usize,
    components : CreateComponents
}

impl Menu {
    pub fn new(ctx: &Context, msg: &Message, items: & Vec<CreateEmbed>) -> Menu {
        Menu {
            ctx: ctx.clone(),
            msg: msg.clone(),
            pages: items.clone(),
            page: 0,
            components: Menu::build_components(),
        }
    }

    pub async fn run(&mut self) -> Result<(), CommandError> {
        let mut m = self.msg.channel_id.send_message(&self.ctx.http, |msg| {
            msg.set_embed(self.pages[self.page].clone())
                .set_components(self.components.clone())
        }).await?;

        let cib = m.await_component_interactions(&self.ctx.shard)
            .timeout(Duration::from_secs(60));
        let mut cic = cib.build();
        while let Some(int) = cic.next().await {
            match int.data.custom_id.as_str() {
                "left" => {
                    if self.page > 0 {
                        self.page -= 1;
                    }
                    else {
                        self.page = self.pages.len() - 1;
                    }
                }
                "right" => {
                    self.page += 1;
                    if self.page == self.pages.len() {
                        self.page = 0;
                    }
                }
                _ => {}
            }
            int.defer(&self.ctx.http).await?;
            self.update_msg(&mut m).await?;
        }

        m.edit(&self.ctx.http, |edit| {
            edit.components(|cmps| {
                cmps.set_action_rows(Vec::new())
            })
        }).await?;

        Ok(())
    }

    async fn update_msg(&self, msg : &mut Message) -> serenity::Result<()> {
        msg.edit(&self.ctx.http, |m| {
            m.set_embed(self.pages[self.page].clone())
        }).await
    }

    fn build_components() -> CreateComponents {
        let mut c = CreateComponents::default();
        c.create_action_row(|row| {
            row
                .create_button(|btn| {
                    btn.style(ButtonStyle::Primary)
                        .label("⬅")
                        .custom_id("left")
                })
                .create_button(|btn| {
                    btn.style(ButtonStyle::Primary)
                        .label("➡")
                        .custom_id("right")
                })
        });
        c
    }
}