use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;

#[derive(Serialize, Deserialize)]
pub struct HeroicAuthConfig {
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[cfg(target_os = "linux")]
fn get_config_path() -> Option<path::PathBuf> {
    let home_dir = env::var("HOME").unwrap();
    let home_dir = path::Path::new(&home_dir);

    let config_path = env::var("XDG_CONFIG_PATH")
        .unwrap_or_else(|_e| home_dir.join(".config").to_str().unwrap().to_owned());
    let config_path = path::Path::new(&config_path);

    let host_path = config_path.join("heroic/gog_store/auth.json");
    let flatpak_path =
        home_dir.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/auth.json");

    if host_path.exists() {
        Some(host_path)
    } else if flatpak_path.exists() {
        Some(flatpak_path)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn get_config_path() -> Option<path::PathBuf> {
    let appdata = env::var("APPDATA").unwrap();
    let appdata = path::Path::new(&appdata);

    Some(appdata.join("heroic/gog_store/auth.json"))
}

#[cfg(target_os = "macos")]
fn get_config_path() -> Option<path::PathBuf> {
    let app_support = env::var("HOME").unwrap();
    let app_support = path::Path::new(&app_support).join("Library/Application Support");
    Some(app_support.join("heroic/gog_store/auth.json"))
}

pub fn load_tokens() -> HeroicAuthConfig {
    let config_path = get_config_path().expect("No heroic's auth.json found");
    log::debug!("Loading Heroic credentials from {:?}", config_path);
    let data = fs::read(config_path).expect("Failed to read heroic auth file");
    let data = data.as_slice();

    let parsed: HeroicAuthConfig =
        serde_json::from_slice(data).expect("Heroic auth file is corrupted");
    parsed
}
