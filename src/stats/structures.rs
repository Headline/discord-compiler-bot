use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::Response;
use serde::*;
use serenity::async_trait;
use serenity::model::id::GuildId;
use std::sync::Arc;

#[async_trait]
pub trait Sendable: Serialize {
    fn endpoint(&self) -> &'static str;
    fn set_key(&mut self, key: &str);
    async fn send(
        &self,
        client: Arc<reqwest::Client>,
        url: &str,
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}/{}", url, self.endpoint());
        debug!("Sending request to: {}", &url);
        client
            .post(&url)
            .json(&self)
            .header(USER_AGENT, "godbolt-rust-crate")
            .header(ACCEPT, "application/json; charset=utf-8")
            .send()
            .await
    }
}

#[derive(Serialize)]
pub struct CommandRequest {
    key: String,
    command: String,
    guild: String,
}
impl CommandRequest {
    pub fn new(command: &str, guild: Option<GuildId>) -> CommandRequest {
        let mut guild_str = String::default();
        if let Some(g) = guild {
            guild_str = g.0.to_string();
        }
        CommandRequest {
            key: String::from(""),
            command: String::from(command),
            guild: guild_str,
        }
    }
}
impl Sendable for CommandRequest {
    #[inline]
    fn endpoint(&self) -> &'static str {
        "insert/command"
    }

    fn set_key(&mut self, key: &str) {
        self.key = String::from(key);
    }
}

#[derive(Serialize)]
pub struct LanguageRequest {
    key: String,
    language: String,
    fail: bool,
}
impl LanguageRequest {
    pub fn new(language: &str, fail: bool) -> LanguageRequest {
        LanguageRequest {
            key: String::from(""),
            language: String::from(language),
            fail,
        }
    }
}
impl Sendable for LanguageRequest {
    #[inline]
    fn endpoint(&self) -> &'static str {
        "insert/language"
    }
    fn set_key(&mut self, key: &str) {
        self.key = String::from(key);
    }
}

#[derive(Serialize)]
pub struct LegacyRequest {
    key: String,
    #[serde(rename = "type")]
    request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<u64>,
}
impl LegacyRequest {
    pub fn new(amount: Option<u64>) -> LegacyRequest {
        let request_type = if amount.is_some() {
            "servers"
        } else {
            "request"
        };

        LegacyRequest {
            key: String::from(""),
            request_type: String::from(request_type),
            amount,
        }
    }
}
impl Sendable for LegacyRequest {
    #[inline]
    fn endpoint(&self) -> &'static str {
        "insert/legacy"
    }
    fn set_key(&mut self, key: &str) {
        self.key = String::from(key);
    }
}
