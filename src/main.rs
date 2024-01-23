use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use env_logger::{Builder, Env, Target};
use log::{error, info, warn};
use reqwest::Client;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod api;
mod constants;
mod heroic;
mod proto;

use crate::api::structs::{Token, UserInfo};
use api::notification_pusher::NotificationPusherClient;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Provide access token (for getting user data)")]
    access_token: Option<String>,
    #[arg(long, help = "Provide refresh token (for creating game sessions)")]
    refresh_token: Option<String>,
    #[arg(long, help = "Galaxy user id from /userData.json")]
    user_id: String,
    #[arg(long, help = "User name")]
    username: String,
    #[arg(long = "from-heroic", help = "Load tokens from heroic")]
    heroic: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let env = Env::new().filter_or("COMET_LOG", "info");
    Builder::from_env(env).target(Target::Stderr).init();

    let access_token: String;
    let refresh_token: String;

    if args.heroic {
        let config = heroic::load_heroic_tokens();
        let config = config
            .fields
            .get(constants::GALAXY_CLIENT_ID)
            .expect("No Galaxy credentials");

        access_token = config
            .get("access_token")
            .expect("access_token not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();
        refresh_token = config
            .get("refresh_token")
            .expect("refresh_token not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();
    } else {
        access_token = args.access_token.expect("Access token is required");
        refresh_token = args.refresh_token.expect("Refresh token is required");
    }

    let reqwest_client = Client::builder()
        .user_agent(format!("Comet/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build reqwest client");

    let user_info = Arc::new(UserInfo {
        username: args.username,
        galaxy_user_id: args.user_id,
    });

    let listener = TcpListener::bind("127.0.0.1:9977")
        .await
        .expect("Failed to bind to port 9977");

    let (topic_sender, _) = tokio::sync::broadcast::channel::<Vec<u8>>(20);
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let pusher_shutdown = shutdown_token.clone(); // Handler for notifications-pusher
    let cloned_shutdown = shutdown_token.clone(); // Handler to share between main thread and sockets

    let token_store: constants::TokenStorage = Arc::new(Mutex::new(HashMap::new()));
    let galaxy_token = Token::new(access_token.clone(), refresh_token.clone());
    let mut store_lock = token_store.lock().await;
    store_lock.insert(String::from(constants::GALAXY_CLIENT_ID), galaxy_token);
    drop(store_lock);

    let notifications_pusher_topic_sender = topic_sender.clone();
    let pusher_handle = tokio::spawn(async move {
        let mut notification_pusher_client = NotificationPusherClient::new(
            &access_token,
            notifications_pusher_topic_sender,
            pusher_shutdown,
        )
        .await;
        notification_pusher_client.handle_loop().await;
        warn!("Notification pusher exiting");
    });
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen to ctrl c signal");
        shutdown_token.cancel();
    });

    info!("Listening on port 9977");
    let socket_shutdown = cloned_shutdown.clone();
    let cloned_user_info = user_info.clone();
    loop {
        let (socket, _addr) = tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok(accept) => accept,
                    Err(error) => {
                        error!("Failed to accept the connection {:?}", error);
                        continue;
                    }
                }
            }
            _ = socket_shutdown.cancelled() => {break}
        };

        // Spawn handler
        let socket_topic_receiver = topic_sender.subscribe();

        let cloned_client = reqwest_client.clone();
        let cloned_token_store = token_store.clone();
        let shutdown_handler = socket_shutdown.clone();
        let socket_user_info = cloned_user_info.clone();
        tokio::spawn(async move {
            api::handlers::entry_point(
                socket,
                cloned_client,
                cloned_token_store,
                socket_user_info,
                socket_topic_receiver,
                shutdown_handler,
            )
            .await
        });
    }

    // Ignore errors, we are exiting
    let _ = pusher_handle.await;
}
