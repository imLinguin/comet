use std::{collections::HashMap, sync::Arc};

use clap::{Parser, Subcommand};
use env_logger::{Builder, Env, Target};
use log::{error, info, warn};
use reqwest::Client;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
#[macro_use]
extern crate lazy_static;
mod api;
mod constants;
mod db;
mod import_parsers;
mod paths;
mod proto;

use crate::api::notification_pusher::PusherEvent;
use crate::api::structs::{Token, UserInfo};
use api::notification_pusher::NotificationPusherClient;

static CERT: &[u8] = include_bytes!("../external/rootCA.pem");

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(about = "Preload achievements and statistics for offline usage")]
    Preload {
        client_id: String,
        client_secret: String,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Provide access token (for getting user data)")]
    access_token: Option<String>,
    #[arg(long, help = "Provide refresh token (for creating game sessions)")]
    refresh_token: Option<String>,
    #[arg(long, help = "Galaxy user id from /userData.json")]
    user_id: Option<String>,
    #[arg(long, help = "User name")]
    username: String,
    #[arg(
        long = "from-heroic",
        help = "Load tokens from heroic",
        global = true,
        group = "import"
    )]
    heroic: bool,
    #[arg(
        long = "from-lutris",
        help = "Load tokens from lutris",
        global = true,
        group = "import"
    )]
    #[cfg(target_os = "linux")]
    lutris: bool,

    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let env = Env::new().filter_or("COMET_LOG", "info");
    Builder::from_env(env).target(Target::Stderr).init();

    let (access_token, refresh_token, galaxy_user_id) =
        import_parsers::handle_credentials_import(&args);

    let certificate = reqwest::tls::Certificate::from_pem(CERT).unwrap();
    let reqwest_client = Client::builder()
        .user_agent(format!("Comet/{}", env!("CARGO_PKG_VERSION")))
        .add_root_certificate(certificate)
        .build()
        .expect("Failed to build reqwest client");

    let user_info = Arc::new(UserInfo {
        username: args.username,
        galaxy_user_id: galaxy_user_id.clone(),
    });

    let token_store: constants::TokenStorage = Arc::new(Mutex::new(HashMap::new()));
    let galaxy_token = Token::new(access_token.clone(), refresh_token.clone());
    let mut store_lock = token_store.lock().await;
    store_lock.insert(String::from(constants::GALAXY_CLIENT_ID), galaxy_token);
    drop(store_lock);

    let client_clone = reqwest_client.clone();
    tokio::spawn(async move {
        api::gog::components::get_peer(
            &client_clone,
            paths::REDISTS_STORAGE.clone(),
            api::gog::components::Platform::Windows,
        )
        .await
        .expect("Failed to get peer");
        #[cfg(target_os = "macos")]
        api::gog::components::get_peer(
            &client_clone,
            paths::REDISTS_STORAGE.clone(),
            api::gog::components::Platform::Mac,
        )
        .await;
    });

    if let Some(subcommand) = args.subcommand {
        match subcommand {
            SubCommand::Preload {
                client_id,
                client_secret,
            } => {
                let database = db::gameplay::setup_connection(&client_id, &galaxy_user_id)
                    .await
                    .expect("Failed to setup the database");

                if !db::gameplay::has_achievements(database.clone()).await
                    || !db::gameplay::has_statistics(database.clone()).await
                {
                    let mut connection = database.acquire().await.unwrap();
                    sqlx::query(db::gameplay::SETUP_QUERY)
                        .execute(&mut *connection)
                        .await
                        .expect("Failed to setup the database");
                    drop(connection);

                    let new_token = api::gog::users::get_token_for(
                        &client_id,
                        &client_secret,
                        &refresh_token,
                        &reqwest_client,
                    )
                    .await
                    .expect("Failed to obtain credentials");

                    let mut tokens = token_store.lock().await;
                    tokens.insert(client_id.clone(), new_token);
                    drop(tokens);

                    let new_achievements = api::gog::achievements::fetch_achievements(
                        &token_store,
                        &client_id,
                        &galaxy_user_id,
                        &reqwest_client,
                    )
                    .await;
                    let new_stats = api::gog::stats::fetch_stats(
                        &token_store,
                        &client_id,
                        &galaxy_user_id,
                        &reqwest_client,
                    )
                    .await;

                    if let Ok((achievements, mode)) = new_achievements {
                        db::gameplay::set_achievements(database.clone(), &achievements, &mode)
                            .await
                            .expect("Failed to write to the database");
                        info!("Got achievements");
                    } else {
                        error!("Failed to fetch achievements");
                    }
                    if let Ok(stats) = new_stats {
                        db::gameplay::set_statistics(database.clone(), &stats)
                            .await
                            .expect("Failed to write to the database");
                        info!("Got stats");
                    } else {
                        error!("Failed to fetch stats")
                    }
                } else {
                    info!("Already in database")
                }
            }
        }

        return;
    }

    let listener = TcpListener::bind("127.0.0.1:9977")
        .await
        .expect("Failed to bind to port 9977");

    let (topic_sender, _) = tokio::sync::broadcast::channel::<PusherEvent>(20);
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let pusher_shutdown = shutdown_token.clone(); // Handler for notifications-pusher
    let cloned_shutdown = shutdown_token.clone(); // Handler to share between main thread and sockets

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
