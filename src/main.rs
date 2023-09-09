use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use reqwest::Client;
use tokio::net::TcpListener;

mod api;
mod constants;
mod heroic;
mod proto;

use api::notification_pusher::NotificationPusherClient;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Provide access token (for getting user data)")]
    access_token: Option<String>,
    #[arg(long, help = "Provide refresh token (for creating game sessions)")]
    refresh_token: Option<String>,
    #[arg(long, help = "Provide user id")]
    user_id: Option<String>,

    #[arg(long = "from-heroic", help = "Load tokens from heroic")]
    heroic: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let logger_env = Env::new().filter("COMET_LOG").default_filter_or("info");
    env_logger::init_from_env(logger_env);

    let user_id: String;
    let access_token: String;
    let refresh_token: String;

    if args.heroic {
        let config = heroic::load_heroic_tokens();
        let config = config
            .fields
            .get(constants::GALAXY_CLIENT_ID)
            .expect("No Galaxy credentials");
        user_id = config
            .get("user_id")
            .expect("user_id not present in heroic config")
            .as_str()
            .unwrap()
            .to_owned();

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
        user_id = args.user_id.expect("User id is required");
        refresh_token = args.refresh_token.expect("Refresh token is required");
    }

    let listener = TcpListener::bind("127.0.0.1:9977")
        .await
        .expect("Failed to bind to port 9977");

    let (topic_sender, _) = tokio::sync::broadcast::channel::<Vec<u8>>(20);
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let pusher_shutdown = shutdown_token.clone(); // Handler for notifications-pusher
    let cloned_shutdown = shutdown_token.clone(); // Handler to share between main thread and sockets
    let reqwest_client = Client::builder()
        .user_agent(format!("Comet/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build reqwest client");

    let token_store: constants::TokenStorage = Arc::new(Mutex::new(HashMap::new()));
    let mut notification_pusher_client =
        NotificationPusherClient::new(&access_token, topic_sender.clone()).await;

    let pusher_handle = tokio::spawn(async move {
        notification_pusher_client
            .handle_loop(pusher_shutdown)
            .await
    });
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen to ctrl c signal");
        shutdown_token.cancel();
    });

    info!("Listening on port 9977");
    let socket_shutdown = cloned_shutdown.clone();
    loop {
        let acceptance = tokio::select! {
            accept = listener.accept() => {Some(accept)}
            _ = socket_shutdown.cancelled() => {None}
        };

        let acceptance = match acceptance {
            Some(acc) => acc,
            None => break,
        };

        if let Err(error) = acceptance {
            error!("Failed to accept the connection {:?}", error);
            continue;
        }

        let (socket, _addr) = acceptance.unwrap();

        // Spawn handler
        let socket_topic_receiver = topic_sender.subscribe();

        let cloned_client = reqwest_client.clone();
        let cloned_token_store = token_store.clone();
        let shutdown_handler = socket_shutdown.clone();
        tokio::spawn(async move {
            api::handlers::entry_point(
                socket,
                cloned_client,
                cloned_token_store,
                socket_topic_receiver,
                shutdown_handler,
            )
            .await
        });
    }

    // Ignore errors, we are exiting
    let _ = pusher_handle.await;
}
