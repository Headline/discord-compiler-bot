use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::error::Error;

use tokio::sync::RwLock;
use tokio::sync::Mutex;

use serenity::prelude::{TypeMap, TypeMapKey};
use serenity::model::id::UserId;
use serenity::client::bridge::gateway::ShardManager;

use crate::managers::stats::StatsManager;
use crate::utls::blocklist::Blocklist;

use lru_cache::LruCache;
use serenity::model::channel::Message;
use crate::managers::compilation::CompilationManager;

/** Caching **/

/// Contains bot configuration information provided mostly from environment variables
pub struct ConfigCache;
impl TypeMapKey for ConfigCache {
    type Value = Arc<RwLock<HashMap<&'static str, String>>>;
}

/// Main interface for compiler options for either Compiler Explorer or WandBox
pub struct CompilerCache;
impl TypeMapKey for CompilerCache {
    type Value = Arc<RwLock<CompilationManager>>;
}

/// Contains our top.gg api client for server count updates
pub struct DblCache;
impl TypeMapKey for DblCache {
    type Value = Arc<RwLock<dbl::Client>>;
}

/// Our endpoints for the in-house statistics tracing - see apis/dbl.rs
pub struct StatsManagerCache;
impl TypeMapKey for StatsManagerCache {
    type Value = Arc<Mutex<StatsManager>>;
}

/// Internal blocklist for abusive users or guilds
pub struct BlocklistCache;
impl TypeMapKey for BlocklistCache {
    type Value = Arc<RwLock<Blocklist>>;
}

/// Contains the shard manager - used to send global presence updates
pub struct ShardManagerCache;
impl TypeMapKey for ShardManagerCache {
    type Value = Arc<Mutex<ShardManager>>;
}

/// Message  cache to interact with our own messages after they are dispatched
pub struct MessageCache;
impl TypeMapKey for MessageCache {
    type Value = Arc<Mutex<LruCache<u64, Message>>>;
}

pub async fn fill(
    data: Arc<RwLock<TypeMap>>,
    prefix: &str,
    id: &UserId,
    shard_manager: Arc<tokio::sync::Mutex<ShardManager>>
) -> Result<(), Box<dyn Error>> {
    let mut data = data.write().await;

    // Lets map some common things in BotInfo
    let mut map = HashMap::<&str, String>::new();

    // optional additions
    let emoji_identifiers = ["SUCCESS_EMOJI_ID", "SUCCESS_EMOJI_NAME", "LOADING_EMOJI_ID", "LOADING_EMOJI_NAME", "LOGO_EMOJI_NAME", "LOGO_EMOJI_ID"];
    for id in emoji_identifiers{
        if let Ok(envvar) = env::var(id) {
            if !envvar.is_empty() {
                map.insert(id, envvar);
            }
        }
    }

    map.insert("GIT_HASH_LONG", String::from(env!("GIT_HASH_LONG")));
    map.insert("GIT_HASH_SHORT", String::from(env!("GIT_HASH_SHORT")));
    map.insert("JOIN_LOG", env::var("JOIN_LOG")?);
    map.insert("BOT_PREFIX", String::from(prefix));
    map.insert("BOT_ID", id.to_string());
    data.insert::<ConfigCache>(Arc::new(RwLock::new(map)));

    // Shard manager for universal presence
    data.insert::<ShardManagerCache>(shard_manager);

    // Message delete cache
    data.insert::<MessageCache>(Arc::new(tokio::sync::Mutex::new(LruCache::new(25))));

    // Compiler manager
    data.insert::<CompilerCache>(Arc::new(RwLock::new(CompilationManager::new().await?)));
    info!("Compilation manager loaded");

    // DBL
    let token = env::var("DBL_TOKEN")?;
    let client = dbl::Client::new(token)?;
    data.insert::<DblCache>(Arc::new(RwLock::new(client)));

    // Stats tracking
    let stats = StatsManager::new();
    if stats.should_track() {
        info!("Statistics tracking enabled");
    }
    data.insert::<StatsManagerCache>(Arc::new(Mutex::new(stats)));

    // Blocklist
    let blocklist = Blocklist::new();
    data.insert::<BlocklistCache>(Arc::new(RwLock::new(blocklist)));

    Ok(())
}
