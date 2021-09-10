use std::sync::Arc;

use std::env;

use crate::stats::structures::*;
use serenity::model::id::GuildId;
use lru_cache::LruCache;

pub struct StatsManager {
    client: Arc<reqwest::Client>,
    url: String,
    pass: String,
    servers: u64,
    shards: u64,
    boot_count: Vec<u64>,
    leave_queue: u64,
    join_queue: u64,
    settings_cache: LruCache<u64, SettingsResponse>
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct SettingsResponse {
    pub prefix : String,
    pub maxlen : i32,
    #[serde(rename = "outputStyle")]
    pub output_style: String,
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
            boot_count: Vec::new(),
            settings_cache: LruCache::new(10)
        }
    }

    pub fn clear_user(&mut self, id : u64) {
        self.settings_cache.remove(&id);
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

    pub async fn get_settings(&mut self, guild: Option<GuildId>) -> Option<SettingsResponse> {
        debug!("Getting settings");
        if let Some(id) = guild {
            if let Some(settings) = self.settings_cache.get_mut(&id.0) {
                debug!("Found from cache: {}", id.0);
                return Some(settings.clone())
            }
            else { // not in cache :(
                let mut req = SettingsRequest::new(id);
                if let Some(response) = self.send_request::<SettingsRequest>(& mut req).await {
                    match response.json::<SettingsResponse>().await {
                        Ok(prefix_resp) => {
                            self.settings_cache.insert(id.0, prefix_resp.clone());
                            debug!("Inserting from cache: {:?}", prefix_resp.clone());
                            return Some(prefix_resp);
                        }
                        Err(e) => {
                            error!("Unable to parse settings response: {}", e);
                        }
                    }
                }
            }
        }
        None
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

    async fn send_request<T: Sendable + std::marker::Sync>(&self, sendable: &mut T) -> Option<reqwest::Response> {
        sendable.set_key(&self.pass);
        return match sendable.send(self.client.clone(), &self.url).await {
            Ok(r) => Some(r),
            Err(e) => {
                warn!("Request failed to {}: {}", sendable.endpoint(), e);
                None
            },
        }
    }
}
