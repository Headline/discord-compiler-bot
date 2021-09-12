use std::sync::Arc;

use std::env;

use crate::stats::structures::*;
use serenity::model::id::GuildId;

pub struct StatsManager {
    client: Arc<reqwest::Client>,
    url: String,
    pass: String,
    servers: u64,
    shards: u64,
    boot_count: Vec<u64>,
    leave_queue: u64,
    join_queue: u64
}

impl StatsManager {
    pub fn new() -> StatsManager {
        StatsManager {
            client: Arc::new(reqwest::Client::new()),
            url: env::var("STATS_API_LINK").unwrap_or_default(),
            pass: env::var("STATS_API_KEY").unwrap_or_default(),
            servers: 0,
            leave_queue: 0,
            join_queue: 0,
            shards: 0,
            boot_count: Vec::new()
        }
    }

    pub fn should_track(&self) -> bool {
        !self.url.is_empty() && !self.pass.is_empty()
    }

    pub async fn compilation(&self, language: &str, fail: bool) {
        let mut cmd = LanguageRequest::new(language, fail);
        self.send_request::<LanguageRequest>(&mut cmd).await;
    }

    pub async fn command_executed(&self, command: &str, guild: Option<GuildId>) {
        let mut cmd = CommandRequest::new(command, guild);
        self.send_request::<CommandRequest>(&mut cmd).await;
    }

    pub async fn post_servers(&mut self, amount: u64) {
        self.servers = amount;

        // in the connect phase it's entirely possible for our server count to be
        // zero while we receive a guild left or guild joined event, since they were
        // queued we can now modify the server count safely

        // join queue
        self.servers += self.join_queue;
        self.join_queue = 0;

        // leave queue
        self.servers -= self.leave_queue;
        self.leave_queue = 0;

        // update our stats
        let mut legacy = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn new_server(&mut self) {
        if self.servers < 1 { // not all shards have loaded in yet - queue the join for post_servers
            self.join_queue += 1;
            return
        }

        self.servers += 1;
        let mut legacy = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn leave_server(&mut self) {
        if self.servers < 1 { // not loaded in - queue leave for post_servers
            self.leave_queue += 1;
            return
        }

        self.servers -= 1;
        let mut legacy = LegacyRequest::new(Some(self.servers));
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub async fn post_request(&self) {
        let mut legacy = LegacyRequest::new(None);
        self.send_request::<LegacyRequest>(&mut legacy).await;
    }

    pub fn server_count(&self) -> u64 {
        self.servers
    }

    pub fn shard_count(&self) -> u64 {
        self.shards
    }

    pub fn add_shard(& mut self, server_count : u64) {
        self.shards += 1;
        self.boot_count.push(server_count);
    }

    pub fn get_boot_vec_sum(&self) -> u64 {
        self.boot_count.iter().sum()
    }

    async fn send_request<T: Sendable + std::marker::Sync>(&self, sendable: &mut T) {
        sendable.set_key(&self.pass);
        match sendable.send(self.client.clone(), &self.url).await {
            Ok(_) => (),
            Err(e) => warn!("Request failed to {}: {}", sendable.endpoint(), e),
        }
    }
}
