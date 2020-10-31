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

/** Caching **/
pub struct BotInfo;
impl TypeMapKey for BotInfo {
    type Value = Arc<RwLock<HashMap<&'static str, String>>>;
}

pub struct WandboxInfo;
impl TypeMapKey for WandboxInfo {
    type Value = Arc<RwLock<Wandbox>>;
}
pub struct GodboltInfo;
impl TypeMapKey for GodboltInfo {
    type Value = Arc<RwLock<Godbolt>>;
}

pub struct DBLApi;
impl TypeMapKey for DBLApi {
    type Value = Arc<RwLock<dbl::Client>>;
}

pub struct ShardServers;
impl TypeMapKey for ShardServers {
    type Value = Arc<Mutex<Vec<u64>>>;
}

pub struct Stats;
impl TypeMapKey for Stats {
    type Value = Arc<Mutex<StatsManager>>;
}

pub struct BlockListInfo;
impl TypeMapKey for BlockListInfo {
    type Value = Arc<RwLock<Blocklist>>;
}

pub async fn fill(
    data: Arc<RwLock<TypeMap>>,
    prefix: &str,
    id: &UserId,
) -> Result<(), Box<dyn Error>> {
    let mut data = data.write().await;

    // Lets map some common things in BotInfo
    let mut map = HashMap::<&str, String>::new();
    map.insert("SUCCESS_EMOJI_ID", env::var("SUCCESS_EMOJI_ID")?);
    map.insert("SUCCESS_EMOJI_NAME", env::var("SUCCESS_EMOJI_NAME")?);
    map.insert("LOADING_EMOJI_ID", env::var("LOADING_EMOJI_ID")?);
    map.insert("LOADING_EMOJI_NAME", env::var("LOADING_EMOJI_NAME")?);
    map.insert("JOIN_LOG", env::var("JOIN_LOG")?);
    map.insert("BOT_PREFIX", String::from(prefix));
    map.insert("BOT_ID", id.to_string());
    data.insert::<BotInfo>(Arc::new(RwLock::new(map)));

    // Wandbox
    let mut broken_compilers = std::collections::HashSet::new();
    broken_compilers.insert(String::from("ghc-head"));
    let mut broken_languages = std::collections::HashSet::new();
    broken_languages.insert(String::from("cpp"));
    let wbox = wandbox::Wandbox::new(Some(broken_compilers), Some(broken_languages)).await?;
    info!("WandBox cache loaded");
    data.insert::<WandboxInfo>(Arc::new(RwLock::new(wbox)));

    // Godbolt
    let godbolt = Godbolt::new().await?;
    info!("Godbolt cache loaded");
    data.insert::<GodboltInfo>(Arc::new(RwLock::new(godbolt)));

    // DBL
    let token = env::var("DBL_TOKEN")?;
    let client = dbl::Client::new(token)?;
    data.insert::<DBLApi>(Arc::new(RwLock::new(client)));

    // DBL
    data.insert::<ShardServers>(Arc::new(Mutex::new(Vec::new())));

    // Stats tracking
    let stats = StatsManager::new();
    if stats.should_track() {
        info!("Statistics tracking enabled");
    }
    data.insert::<Stats>(Arc::new(Mutex::new(stats)));


    // Blocklist
    let blocklist = Blocklist::new();
    data.insert::<BlockListInfo>(Arc::new(RwLock::new(blocklist)));

    Ok(())
}
