use std::env;
use std::collections::HashMap;
use serenity::prelude::{TypeMapKey, TypeMap};
use std::sync::{Arc};
use tokio::sync::RwLock;

use wandbox::Wandbox;
use godbolt::Godbolt;
use serenity::futures::lock::Mutex;
use std::error::Error;

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
    type Value = Arc<Mutex<Vec<usize>>>;
}

pub async fn fill(data : Arc<RwLock<TypeMap>>, prefix : &str) -> Result<(), Box<dyn Error>>{
    let mut data = data.write().await;

    // Lets map some common things in BotInfo
    let mut map = HashMap::<&str, String>::new();
    map.insert("SUCCESS_EMOJI_ID", env::var("SUCCESS_EMOJI_ID")?);
    map.insert("SUCCESS_EMOJI_NAME", env::var("SUCCESS_EMOJI_NAME")?);
    map.insert("LOADING_EMOJI_ID", env::var("LOADING_EMOJI_ID")?);
    map.insert("LOADING_EMOJI_NAME", env::var("LOADING_EMOJI_NAME")?);
    map.insert("BOT_PREFIX", String::from(prefix));
    data.insert::<BotInfo>(Arc::new(RwLock::new(map)));

    // Wandbox
    let wbox = wandbox::Wandbox::new(None, None).await?;
    info!("WandBox cache loaded");
    data.insert::<WandboxInfo>(Arc::new(RwLock::new(wbox)));

    // Godbolt
    let godbolt = Godbolt::new().await?;
    info!("Godbolt cache loaded");
    data.insert::<GodboltInfo>(Arc::new(RwLock::new(godbolt)));

    // DBL
    let token = env::var("DBL_TOKEN")?;
    let client =  dbl::Client::new(token)?;
    data.insert::<DBLApi>(Arc::new(RwLock::new(client)));

    // DBL
    data.insert::<ShardServers>(Arc::new(Mutex::new(Vec::new())));
    Ok(())
}
