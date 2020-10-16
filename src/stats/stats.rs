use std::sync::Arc;

use std::env;

use crate::stats::structures::*;

pub struct StatsManager {
    client : Arc<reqwest::Client>,
    url : String,
    pass : String,
    servers : usize
}

impl StatsManager {
    pub fn new() -> StatsManager {
        StatsManager {
            client: Arc::new(reqwest::Client::new()),
            url : env::var("STATS_API_LINK").unwrap_or_default(),
            pass : env::var("STATS_API_KEY").unwrap_or_default(),
            servers : 0
        }
    }

    pub fn should_track(&self) -> bool {
        return !self.url.is_empty() && !self.pass.is_empty();
    }

    pub async fn post_servers(&mut self, amount : usize) {
        self.servers = amount;
        let mut lang = LegacyRequest::new(Some(amount));
        self.send_request::<LegacyRequest>(&mut lang).await;
    }

    pub async fn new_server(&mut self) {
        self.servers += 1;
        let mut lang = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut lang).await;
    }

    pub async fn leave_server(&mut self) {
        self.servers -= 1;
        let mut lang = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut lang).await;
    }

    async fn send_request<T : Sendable + std::marker::Sync>(&self, sendable : & mut T) {
        sendable.set_key(&self.pass);
        match sendable.send(self.client.clone(), &self.url).await {
            Ok(_) => (),
            Err(e) => warn!("Request failed to {}: {}", sendable.endpoint(), e)
        }
    }
}