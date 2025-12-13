use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use clap::{Parser, Subcommand};
use comet::api::gog::overlay::OverlayPeerMessage;
use env_logger::{Builder, Env, Target};
use futures_util::future::join_all;
use log::{error, info, warn};
use reqwest::Client;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
mod import_parsers;

use comet::api;
use comet::constants;
use comet::db;
use comet::paths;

use comet::api::notification_pusher::NotificationPusherClient;
use comet::api::notification_pusher::PusherEvent;
use comet::api::structs::{Token, UserInfo};
use tokio::task::JoinHandle;

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(about = "Preload achievements and statistics for offline usage")]
    Preload {
        client_id: String,
        client_secret: String,
    },

    #[command(about = "Download overlay")]
    Overlay {
        #[arg(long, help = "Force the download of non-native overlay")]
        force: bool,
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

    #[arg(
        long = "from-wyvern",
        help = "Load tokens from wyvern",
        global = true,
        group = "import"
    )]
    #[cfg(target_os = "linux")]
    wyvern: bool,

    #[arg(
        short,
        long,
        global = true,
        help = "Make comet quit after every client disconnects. Use COMET_IDLE_WAIT environment variable to control the wait time (seconds)"
    )]
    quit: bool,

    #[command(subcommand)]
    subcommand: Option<SubCommand>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    #[cfg(debug_assertions)]
    let log_level = "debug";
    #[cfg(not(debug_assertions))]
    let log_level = "info";
    let env = Env::new().filter_or("COMET_LOG", log_level);
    Builder::from_env(env)
        .target(Target::Stderr)
        .filter_module("h2::codec", log::LevelFilter::Off)
        .init();

    log::debug!("Configuration file {:?}", *comet::CONFIG);
    log::info!("Preferred language: {}", comet::LOCALE.as_str());

    let (access_token, refresh_token, galaxy_user_id) =
        import_parsers::handle_credentials_import(&args);

    let certificate = reqwest::tls::Certificate::from_pem(comet::CERT).unwrap();
    let reqwest_client = Client::builder()
        .user_agent(format!("GOGGalaxyCommunicationService/2.0.13.27 (Windows_32bit) installation_source/gog Comet/{}", env!("CARGO_PKG_VERSION")))
        .add_root_certificate(certificate)
        .build()
        .expect("Failed to build reqwest client");

    let user_info = Arc::new(UserInfo {
        username: args.username,
        galaxy_user_id: galaxy_user_id.clone(),
    });

    let token_store: constants::TokenStorage = Arc::new(Mutex::new(HashMap::new()));
    let galaxy_token = Token::new(access_token.clone(), refresh_token.clone());
    {
        let mut store_lock = token_store.lock().await;
        store_lock.insert(String::from(constants::GALAXY_CLIENT_ID), galaxy_token);
    }

    let client_clone = reqwest_client.clone();
    tokio::spawn(async move {
        let mut retries = 0;
        loop {
            if retries > 10 {
                log::warn!("Failed to get peer libraries over 10 times, will not try again");
                return;
            }
            tokio::time::sleep(Duration::from_secs(retries * 5)).await;
            retries += 1;

            let result_win = api::gog::components::get_component(
                &client_clone,
                paths::REDISTS_STORAGE.clone(),
                api::gog::components::Platform::Windows,
                api::gog::components::Component::Peer,
            )
            .await;
            #[cfg(target_os = "macos")]
            let result_mac = api::gog::components::get_component(
                &client_clone,
                paths::REDISTS_STORAGE.clone(),
                api::gog::components::Platform::Mac,
                api::gog::components::Component::Peer,
            )
            .await;

            #[cfg(target_os = "macos")]
            if result_win.is_ok() && result_mac.is_ok() {
                break;
            } else {
                continue;
            }

            #[cfg(not(target_os = "macos"))]
            if result_win.is_ok() {
                break;
            }
        }
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

                if !db::gameplay::has_achievements(&database).await
                    || !db::gameplay::has_statistics(&database).await
                {
                    {
                        let mut connection = database.acquire().await.unwrap();
                        sqlx::query(db::gameplay::SETUP_QUERY)
                            .execute(&mut *connection)
                            .await
                            .expect("Failed to setup the database");
                    }

                    let new_token = api::gog::users::get_token_for(
                        &client_id,
                        &client_secret,
                        &refresh_token,
                        &reqwest_client,
                        false,
                    )
                    .await
                    .expect("Failed to obtain credentials");

                    {
                        let mut tokens = token_store.lock().await;
                        tokens.insert(client_id.clone(), new_token);
                    }

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
            SubCommand::Overlay { force } => {
                #[cfg(target_os = "linux")]
                if !force {
                    error!("There is no linux native overlay, to download a windows version use --force");
                    return;
                }
                #[cfg(not(target_os = "linux"))]
                if force {
                    warn!("The force flag has no effect on this platform");
                }

                let web = api::gog::components::get_component(
                    &reqwest_client,
                    paths::REDISTS_STORAGE.clone(),
                    api::gog::components::Platform::Windows,
                    api::gog::components::Component::Web,
                )
                .await;
                let overlay = api::gog::components::get_component(
                    &reqwest_client,
                    paths::REDISTS_STORAGE.clone(),
                    #[cfg(not(target_os = "macos"))]
                    api::gog::components::Platform::Windows,
                    #[cfg(target_os = "macos")]
                    api::gog::components::Platform::Mac,
                    api::gog::components::Component::Overlay,
                )
                .await;

                if let Err(err) = web {
                    error!("Unexpected error occured when downloading web component {err}");
                }

                if let Err(err) = overlay {
                    error!("Unexpected error occured when downloading overlay component {err}");
                }

                log::info!("Done");
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

    let (client_exit, mut con_exit_recv) = tokio::sync::mpsc::unbounded_channel::<bool>();
    let (overlay_ipc, _recv) = tokio::sync::broadcast::channel::<(u32, OverlayPeerMessage)>(32);

    let comet_idle_wait: u64 = match std::env::var("COMET_IDLE_WAIT") {
        Ok(wait) => wait.parse().unwrap_or(15),
        Err(_) => 15,
    };
    let mut ever_connected = false;
    let mut active_clients = 0;
    let mut handlers = Vec::new();
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
            _ = tokio::time::sleep(Duration::from_secs(comet_idle_wait)) => {
                handlers.retain(|handler: &JoinHandle<()>| !handler.is_finished());
                if active_clients == 0 && ever_connected {
                    socket_shutdown.cancel();
                    break
                }
                continue;
            },
            _ = con_exit_recv.recv() => { active_clients -= 1; continue; }
            _ = socket_shutdown.cancelled() => {break}
        };

        // Spawn handler
        let socket_topic_receiver = topic_sender.subscribe();

        let cloned_client = reqwest_client.clone();
        let cloned_token_store = token_store.clone();
        let shutdown_handler = socket_shutdown.clone();
        let socket_user_info = cloned_user_info.clone();
        let client_exit = client_exit.clone();
        let achievement_unlock_event = overlay_ipc.clone();
        active_clients += 1;
        ever_connected = args.quit;
        handlers.push(tokio::spawn(async move {
            api::handlers::entry_point(
                socket,
                cloned_client,
                cloned_token_store,
                socket_user_info,
                socket_topic_receiver,
                achievement_unlock_event,
                shutdown_handler,
            )
            .await;
            let _ = client_exit.send(true);
        }));
    }

    // Ignore errors, we are exiting
    let _ = pusher_handle.await;
    join_all(handlers).await;
}
