pub const GALAXY_CLIENT_ID: &str = "46899977096215655";
pub const NOTIFICATIONS_PUSHER_SOCKET: &str = "wss://notifications-pusher.gog.com/";

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub type TokenStorage = Arc<Mutex<HashMap<String, String>>>;
