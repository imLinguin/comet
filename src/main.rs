use std::sync::{Arc, Mutex};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};

// PROTOBUF
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}
mod api;
mod constants;
mod heroic;

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
            .to_string();
        access_token = config
            .get("access_token")
            .expect("access_token not present in heroic config")
            .to_string();
        refresh_token = config
            .get("refresh_token")
            .expect("refresh_token not present in heroic config")
            .to_string();
    } else {
        access_token = args.access_token.expect("Access token is required");
        user_id = args.user_id.expect("User id is required");
        refresh_token = args.refresh_token.expect("Refresh token is required");
    }

    let listener = TcpListener::bind("127.0.0.1:9977")
        .await
        .expect("Failed to bind to port 9977");

    info!("Listening on port 9977");

    let notification_pusher_client = Arc::new(Mutex::new(NotificationPusherClient::new(&access_token).await));
    let mut sockets: Vec<Arc<TcpStream>> = Vec::new();
    

    loop {
        let acceptance = listener.accept().await;

        if let Err(error) = acceptance {
            error!("Failed to accept the connection {error:?}");
            continue;
        }

        let (socket, _addr) = acceptance.unwrap();

        let socket = Arc::new(socket);

        sockets.push(socket.clone());
    }
}
