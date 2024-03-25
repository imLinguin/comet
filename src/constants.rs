pub static GALAXY_CLIENT_ID: &str = "46899977096215655";
pub static NOTIFICATIONS_PUSHER_SOCKET: &str = "wss://notifications-pusher.gog.com/";

use crate::api::structs::Token;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub type TokenStorage = Arc<Mutex<HashMap<String, Token>>>;
