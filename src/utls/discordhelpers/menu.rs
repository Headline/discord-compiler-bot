use futures_util::StreamExt;
use serenity::all::{CreateActionRow, CreateButton, CreateMessage, EditMessage};
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use std::time::Duration;

pub struct Menu {
    ctx: Context,
    msg: Message,
    pages: Vec<CreateEmbed>,
    page: usize,
    components: Vec<CreateActionRow>,
}

impl Menu {
    pub fn new(ctx: &Context, msg: &Message, items: &[CreateEmbed]) -> Menu {
        Menu {
            ctx: ctx.clone(),
            msg: msg.clone(),
            pages: Vec::from(items),
            page: 0,
            components: Menu::build_components(),
        }
    }

    pub async fn run(&mut self) -> Result<(), CommandError> {
        let msg = CreateMessage::new()
            .embed(self.pages[self.page].clone())
            .components(self.components.clone());
        let mut m = self
            .msg
            .channel_id
            .send_message(&self.ctx.http, msg)
            .await?;

        let mut cib = m
            .await_component_interactions(&self.ctx.shard)
            .timeout(Duration::from_secs(60))
            .stream();
        while let Some(int) = cib.next().await {
            match int.data.custom_id.as_str() {
                "left" => {
                    if self.page > 0 {
                        self.page -= 1;
                    } else {
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

        let edit = EditMessage::new().components(Vec::new());

        let _ = m.edit(&self.ctx.http, edit).await;
        Ok(())
    }

    async fn update_msg(&self, msg: &mut Message) -> serenity::Result<()> {
        let edit = EditMessage::new().embed(self.pages[self.page].clone());

        msg.edit(&self.ctx.http, edit).await
    }

    fn build_components() -> Vec<CreateActionRow> {
        let left = CreateButton::new("left").label("⬅");

        let right = CreateButton::new("right").label("➡");

        vec![CreateActionRow::Buttons(vec![left, right])]
    }
}
