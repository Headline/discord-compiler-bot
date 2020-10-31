use std::sync::Arc;

use std::env;

use crate::stats::structures::*;

pub struct StatsManager {
    client: Arc<reqwest::Client>,
    url: String,
    pass: String,
    servers: u64,
}

impl StatsManager {
    pub fn new() -> StatsManager {
        StatsManager {
            client: Arc::new(reqwest::Client::new()),
            url: env::var("STATS_API_LINK").unwrap_or_default(),
            pass: env::var("STATS_API_KEY").unwrap_or_default(),
            servers: 0,
        }
    }

    pub fn should_track(&self) -> bool {
        !self.url.is_empty() && !self.pass.is_empty()
    }

    pub async fn compilation(&self, language: &str, fail: bool) {
        let mut cmd = LanguageRequest::new(language, fail);
        self.send_request::<LanguageRequest>(&mut cmd).await;
    }

    pub async fn command_executed(&self, command: &str) {
        let mut cmd = CommandRequest::new(command);
        self.send_request::<CommandRequest>(&mut cmd).await;
    }

    pub async fn post_servers(&mut self, amount: u64) {
        self.servers = amount;
        let mut legacy = LegacyRequest::new(Some(amount));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn new_server(&mut self) {
        self.servers += 1;
        let mut legacy = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn leave_server(&mut self) {
        self.servers -= 1;
        let mut legacy = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn post_request(&self) {
        let mut legacy = LegacyRequest::new(None);
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    async fn send_request<T: Sendable + std::marker::Sync>(&self, sendable: &mut T) {
        sendable.set_key(&self.pass);
        match sendable.send(self.client.clone(), &self.url).await {
            Ok(_) => (),
            Err(e) => warn!("Request failed to {}: {}", sendable.endpoint(), e),
        }
    }
}
