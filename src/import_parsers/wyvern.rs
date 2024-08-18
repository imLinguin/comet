use derive_getters::Dissolve;
use serde::Deserialize;
use std::env;
use std::fs::read_to_string;
use std::path;

#[derive(Deserialize, Dissolve)]
pub struct WyvernTokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct WyvernConfig {
    pub token: WyvernTokenData,
}

fn get_config_path() -> Option<path::PathBuf> {
    let home_dir = env::var("HOME").unwrap();
    let home_dir = path::Path::new(&home_dir);

    let config_path: path::PathBuf = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_e| home_dir.join(".config").to_str().unwrap().to_string())
        .into();

    let wyvern_config_path = config_path.join("wyvern/wyvern.toml");

    if wyvern_config_path.exists() {
        Some(wyvern_config_path)
    } else {
        None
    }
}

pub fn load_tokens() -> WyvernConfig {
    let config_path = get_config_path().expect("Wyvern toml doesn't exist");
    let data = read_to_string(config_path).expect("Failed to read wyvern.toml");
    toml::from_str(&data).expect("Failed to parse wyvern config")
}
