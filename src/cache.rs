use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::Mutex;
use tokio::sync::RwLock;

use serenity::prelude::{TypeMap, TypeMapKey};

use crate::managers::stats::StatsManager;
use crate::utls::blocklist::Blocklist;

use crate::apis::insights::InsightsAPI;
use crate::apis::quick_link::LinkAPI;
use crate::managers::command::CommandManager;
use crate::managers::compilation::CompilationManager;
use lru_cache::LruCache;
use serenity::all::{ApplicationId, CommandInteraction, ShardManager};
use serenity::model::channel::Message;

/* Caching */

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
    type Value = Mutex<Arc<ShardManager>>;
}

/// Contains the quick link api - used for godbolt button
pub struct LinkAPICache;
impl TypeMapKey for LinkAPICache {
    type Value = Arc<RwLock<LinkAPI>>;
}

/// Contains the cpp insights api - used for ;insights
pub struct InsightsAPICache;
impl TypeMapKey for InsightsAPICache {
    type Value = Arc<Mutex<InsightsAPI>>;
}

#[derive(Clone)]
pub struct MessageCacheEntry {
    pub our_msg: Message,
    pub original_msg: Message,
}

impl MessageCacheEntry {
    pub fn new(our_msg: Message, original_msg: Message) -> Self {
        MessageCacheEntry {
            our_msg,
            original_msg,
        }
    }
}

/// Message  cache to interact with our own messages after they are dispatched
pub struct MessageCache;
impl TypeMapKey for MessageCache {
    type Value = Arc<Mutex<LruCache<u64, MessageCacheEntry>>>;
}

/// Holds the Command Manager which handles command registration logic
pub struct CommandCache;
impl TypeMapKey for CommandCache {
    type Value = Arc<RwLock<CommandManager>>;
}

pub struct DiffCommandEntry {
    pub expired_timestamp: SystemTime,
    pub content: String,
    pub first_interaction: CommandInteraction,
}
impl DiffCommandEntry {
    pub fn new(content: &str, msg: &CommandInteraction) -> Self {
        DiffCommandEntry {
            content: content.to_owned(),
            expired_timestamp: SystemTime::now() + std::time::Duration::from_secs(30),
            first_interaction: msg.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expired_timestamp < SystemTime::now()
    }
}

/// Contains the first message used in the diff message command, w/ expiry timestamp
pub struct DiffCommandCache;
impl TypeMapKey for DiffCommandCache {
    type Value = Arc<Mutex<LruCache<u64, DiffCommandEntry>>>;
}

pub async fn fill(
    data: Arc<RwLock<TypeMap>>,
    prefix: &str,
    id: ApplicationId,
    shard_manager: Arc<ShardManager>,
) -> Result<(), Box<dyn Error>> {
    let mut data = data.write().await;

    // Lets map some common things in BotInfo
    let mut map = HashMap::<&str, String>::new();

    // optional additions
    let emoji_identifiers = [
        "SUCCESS_EMOJI_ID",
        "SUCCESS_EMOJI_NAME",
        "FAIL_EMOJI_NAME",
        "FAIL_EMOJI_ID",
        "LOADING_EMOJI_ID",
        "LOADING_EMOJI_NAME",
        "LOGO_EMOJI_NAME",
        "LOGO_EMOJI_ID",
    ];
    for id in &emoji_identifiers {
        if let Ok(envvar) = env::var(id) {
            if !envvar.is_empty() {
                map.insert(id, envvar);
            }
        }
    }

    map.insert("GIT_HASH_LONG", String::from(env!("GIT_HASH_LONG")));
    map.insert("GIT_HASH_SHORT", String::from(env!("GIT_HASH_SHORT")));

    if let Ok(jlog) = env::var("JOIN_LOG") {
        map.insert("JOIN_LOG", jlog);
    }
    if let Ok(clog) = env::var("COMPILE_LOG") {
        map.insert("COMPILE_LOG", clog);
    }

    map.insert("INVITE_LINK", env::var("INVITE_LINK")?);
    map.insert("DISCORDBOTS_LINK", env::var("DISCORDBOTS_LINK")?);
    map.insert("GITHUB_LINK", env::var("GITHUB_LINK")?);
    map.insert("STATS_LINK", env::var("STATS_LINK")?);
    map.insert("BOT_PREFIX", String::from(prefix));
    map.insert("BOT_ID", id.to_string());
    data.insert::<ConfigCache>(Arc::new(RwLock::new(map)));

    // Shard manager for universal presence
    data.insert::<ShardManagerCache>(Mutex::new(shard_manager));

    // Message delete cache
    data.insert::<MessageCache>(Arc::new(Mutex::new(LruCache::new(25))));

    // Compiler manager
    data.insert::<CompilerCache>(Arc::new(RwLock::new(CompilationManager::new().await?)));
    info!("Compilation manager loaded");

    // DBL
    if let Ok(token) = env::var("DBL_TOKEN") {
        let client = dbl::Client::new(token)?;
        data.insert::<DblCache>(Arc::new(RwLock::new(client)));
    }

    // DBL
    if let Ok(redirect_base) = env::var("QUICK_LINK_URL") {
        if let Ok(request_base) = env::var("QUICK_LINK_POST") {
            info!("Registered quick link api");
            let link_man = LinkAPI::new(&request_base, &redirect_base);
            data.insert::<LinkAPICache>(Arc::new(RwLock::new(link_man)));
        }
    }

    // Cpp insights
    let insights = InsightsAPI::new();
    data.insert::<InsightsAPICache>(Arc::new(Mutex::new(insights)));

    // Stats tracking
    let stats = StatsManager::new();
    if stats.should_track() {
        info!("Statistics tracking enabled");
    }
    data.insert::<StatsManagerCache>(Arc::new(Mutex::new(stats)));

    // Blocklist
    let blocklist = Blocklist::new();
    data.insert::<BlocklistCache>(Arc::new(RwLock::new(blocklist)));

    // Commands
    let commands = CommandManager::new();
    data.insert::<CommandCache>(Arc::new(RwLock::new(commands)));

    // Diff command message tracker
    data.insert::<DiffCommandCache>(Arc::new(Mutex::new(LruCache::new(10))));

    Ok(())
}
