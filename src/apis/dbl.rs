use std::env;
use std::sync::Arc;

use tokio::sync::RwLock;

use serenity::{http::Http, prelude::TypeMap};
use warp::{
    body::BodyDeserializeError,
    http::StatusCode,
    {path, Filter, Rejection, Reply},
};

use dbl::types::Webhook;
use futures_util::future;

use crate::cache::DblCache;
use crate::utls::discordhelpers;
use crate::utls::discordhelpers::embeds::*;

pub struct BotsListApi {
    password: String,
    port: u16,
    vote_channel: u64,
}

impl BotsListApi {
    pub fn new() -> BotsListApi {
        let webhookpass = env::var("DBL_WEBHOOK_PASSWORD").unwrap_or_default();
        let webhookport = env::var("DBL_WEBHOOK_PORT").unwrap_or_default();
        let port = webhookport.parse::<u16>().unwrap_or_default();
        let vote_channel = env::var("VOTE_CHANNEL").unwrap_or_default();
        let channel_id = vote_channel.parse::<u64>().unwrap_or_default();

        BotsListApi {
            password: webhookpass,
            port,
            vote_channel: channel_id,
        }
    }

    pub fn should_spawn(&self) -> bool {
        self.port != 0 && !self.password.is_empty() && self.vote_channel != 0
    }

    pub fn spawn(self, http: Arc<Http>, data: Arc<RwLock<TypeMap>>) {
        tokio::spawn(async move {
            BotsListApi::start_webhook(
                http,
                data,
                self.vote_channel,
                self.password.clone(),
                self.port,
            )
            .await
        });
    }

    async fn start_webhook(
        http: Arc<Http>,
        data: Arc<RwLock<TypeMap>>,
        vote_channel: u64,
        pass: String,
        port: u16,
    ) {
        let filter = warp::header::<String>("authorization")
            .and_then(move |value| {
                if value == pass {
                    future::ok(())
                } else {
                    future::err(warp::reject::custom(Unauthorized))
                }
            })
            .untuple_one();

        let webhook = warp::post()
            .and(path!("dblwebhook"))
            .and(filter)
            .and(warp::body::json())
            .map(move |hook: Webhook| {
                let user_id = hook.user.0;
                let data = data.clone();
                let http: Arc<Http> = http.clone();
                BotsListApi::send_vote(user_id, vote_channel, http, data);

                warp::reply()
            })
            .recover(custom_error);

        info!("Starting webhook");
        warp::serve(webhook).run(([0, 0, 0, 0], port)).await;
    }

    fn send_vote(user_id: u64, vote_channel: u64, http: Arc<Http>, data: Arc<RwLock<TypeMap>>) {
        tokio::spawn(async move {
            let read = data.read().await;
            let client_lock = read.get::<DblCache>().expect("Unable to find dbl data");
            let awd = client_lock.read().await;

            let usr = match awd.user(user_id).await {
                Ok(u) => u,
                Err(err) => return warn!("Unable to retrieve user info: {}", err),
            };

            let tag = format!("{}#{}", usr.username, usr.discriminator);
            let emb = build_dblvote_embed(tag);
            discordhelpers::manual_dispatch(http.clone(), vote_channel, emb).await;
        });
    }
}

async fn custom_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.find::<BodyDeserializeError>().is_some() {
        Ok(warp::reply::with_status(
            warp::reply(),
            StatusCode::BAD_REQUEST,
        ))
    } else if err.find::<Unauthorized>().is_some() {
        Ok(warp::reply::with_status(
            warp::reply(),
            StatusCode::UNAUTHORIZED,
        ))
    } else {
        Err(err)
    }
}

#[derive(Debug)]
struct Unauthorized;

impl warp::reject::Reject for Unauthorized {}

impl std::fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unauthorized")
    }
}

impl std::error::Error for Unauthorized {}
