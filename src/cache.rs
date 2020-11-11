use serenity::prelude::{TypeMap, TypeMapKey};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::stats::statsmanager::StatsManager;
use crate::utls::blocklist::Blocklist;

use godbolt::Godbolt;
use serenity::futures::lock::Mutex;
use serenity::model::id::UserId;
use std::error::Error;
use wandbox::Wandbox;
use serenity::client::bridge::gateway::ShardManager;

/** Caching **/

/// Contains bot configuration information provided mostly from environment variables
pub struct ConfigCache;
impl TypeMapKey for ConfigCache {
    type Value = Arc<RwLock<HashMap<&'static str, String>>>;
}

/// The cache of all compilers/languages from wandbox - along with our bindings for their api
pub struct WandboxCache;
impl TypeMapKey for WandboxCache {
    type Value = Arc<RwLock<Wandbox>>;
}

/// Same as WandBox cache, but this time it's Matthew's toys
pub struct GodboltCache;
impl TypeMapKey for GodboltCache {
    type Value = Arc<RwLock<Godbolt>>;
}

/// Contains our top.gg api client for server count updates
pub struct DBLCache;
impl TypeMapKey for DBLCache {
    type Value = Arc<RwLock<dbl::Client>>;
}

/// Server count on boot, each shard has their own entry and server count
//TODO: This should die eventually
pub struct ServerCountCache;
impl TypeMapKey for ServerCountCache {
    type Value = Arc<Mutex<Vec<u64>>>;
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
    type Value = Arc<tokio::sync::Mutex<ShardManager>>;
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
    map.insert("SUCCESS_EMOJI_ID", env::var("SUCCESS_EMOJI_ID")?);
    map.insert("SUCCESS_EMOJI_NAME", env::var("SUCCESS_EMOJI_NAME")?);
    map.insert("LOADING_EMOJI_ID", env::var("LOADING_EMOJI_ID")?);
    map.insert("LOADING_EMOJI_NAME", env::var("LOADING_EMOJI_NAME")?);
    map.insert("GIT_HASH_LONG", String::from(env!("GIT_HASH_LONG")));
    map.insert("GIT_HASH_SHORT", String::from(env!("GIT_HASH_SHORT")));
    map.insert("JOIN_LOG", env::var("JOIN_LOG")?);
    map.insert("BOT_PREFIX", String::from(prefix));
    map.insert("BOT_ID", id.to_string());
    data.insert::<ConfigCache>(Arc::new(RwLock::new(map)));

    // Shard manager for universal presence
    data.insert::<ShardManagerCache>(shard_manager);

    // Wandbox
    let mut broken_compilers = std::collections::HashSet::new();
    broken_compilers.insert(String::from("ghc-head"));
    let mut broken_languages = std::collections::HashSet::new();
    broken_languages.insert(String::from("cpp"));
    let wbox = wandbox::Wandbox::new(Some(broken_compilers), Some(broken_languages)).await?;
    info!("WandBox cache loaded");
    data.insert::<WandboxCache>(Arc::new(RwLock::new(wbox)));

    // Godbolt
    let godbolt = Godbolt::new().await?;
    info!("Godbolt cache loaded");
    data.insert::<GodboltCache>(Arc::new(RwLock::new(godbolt)));

    // DBL
    let token = env::var("DBL_TOKEN")?;
    let client = dbl::Client::new(token)?;
    data.insert::<DBLCache>(Arc::new(RwLock::new(client)));

    // DBL
    data.insert::<ServerCountCache>(Arc::new(Mutex::new(Vec::new())));

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
