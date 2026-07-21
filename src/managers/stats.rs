use std::sync::Arc;
use std::time::Duration;

use serenity::prelude::TypeMap;
use tokio::sync::RwLock;

use crate::cache::{ConfigCache, DblCache, ShardManagerCache, StatsManagerCache};
use crate::utls::discordhelpers;

// how often the background task pushes server count to top.gg and presence
const STATS_FLUSH_INTERVAL: Duration = Duration::from_secs(500);

pub struct StatsManager {
    servers: u64,
    shards: u32,
    boot_count: Vec<u64>,
    leave_queue: u64,
    join_queue: u64,
    last_presence: u64,
}

impl StatsManager {
    pub fn new() -> StatsManager {
        StatsManager {
            servers: 0,
            leave_queue: 0,
            join_queue: 0,
            shards: 0,
            boot_count: Vec::new(),
            last_presence: 0,
        }
    }

    /// Updates server count from boot
    pub fn post_servers(&mut self, amount: u64) {
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

        self.last_presence = self.servers;
    }

    /// Registers a new server
    pub fn new_server(&mut self) {
        if self.servers < 1 {
            // not all shards have loaded in yet - queue the join for post_servers
            self.join_queue += 1;
            return;
        }
        self.servers += 1;
    }

    /// Registers a server leave
    pub fn leave_server(&mut self) {
        if self.servers < 1 {
            // not loaded in - queue leave for post_servers
            self.leave_queue += 1;
            return;
        }
        self.servers -= 1;
    }

    pub fn server_count(&self) -> u64 {
        self.servers
    }

    pub fn shard_count(&self) -> u32 {
        self.shards
    }

    pub fn add_shard(&mut self, server_count: u64) {
        self.shards += 1;
        self.boot_count.push(server_count);
    }

    pub fn get_boot_vec_sum(&self) -> u64 {
        self.boot_count.iter().sum()
    }

    fn take_dirty(&mut self) -> bool {
        if self.servers != self.last_presence {
            self.last_presence = self.servers;
            true
        } else {
            false
        }
    }

    pub fn spawn_flusher(data: Arc<RwLock<TypeMap>>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(STATS_FLUSH_INTERVAL);
            loop {
                interval.tick().await;
                StatsManager::flush(&data).await;
            }
        });
    }

    async fn flush(data: &Arc<RwLock<TypeMap>>) {
        let read = data.read().await;

        let (server_count, shard_count) = {
            let mut stats = read.get::<StatsManagerCache>().unwrap().lock().await;
            if !stats.take_dirty() {
                return;
            }
            (stats.server_count(), stats.shard_count())
        };

        if let Some(dbl_cache) = read.get::<DblCache>() {
            let id = {
                let info = read.get::<ConfigCache>().unwrap().read().await;
                info.get("BOT_ID").unwrap().parse::<u64>().unwrap()
            };
            let new_stats = dbl::types::ShardStats::Cumulative {
                server_count,
                shard_count: Some(shard_count as u64),
            };
            let dbl = dbl_cache.read().await.clone();
            if let Err(e) = dbl.update_stats(id, new_stats).await {
                warn!("Failed to post stats to dbl: {}", e);
            }
        }

        let shard_manager = read.get::<ShardManagerCache>().unwrap().lock().await;
        discordhelpers::send_global_presence(&shard_manager, server_count).await;
    }
}
