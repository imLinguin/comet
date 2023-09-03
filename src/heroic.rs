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
fn get_heroic_config_path() -> path::PathBuf {
    let home_dir = env::var("HOME").unwrap();
    let home_dir = path::Path::new(&home_dir);

    let config_path = env::var("XDG_CONFIG_PATH")
        .unwrap_or_else(|_e| home_dir.join(".config").to_str().unwrap().to_owned());
    let config_path = path::Path::new(&config_path);

    let flatpak_path =
        home_dir.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/auth.json");

    if flatpak_path.exists() {
        flatpak_path
    } else {
        config_path.join("heroic/gog_store/auth.json")
    }
}

#[cfg(target_os = "windows")]
fn get_heroic_config_path() -> path::PathBuf {
    let appdata = env::var("APPDATA").unwrap();
    let appdata = path::Path::new(&appdata);

    appdata.join("heroic/gog_store/auth.json")
}

pub fn load_heroic_tokens() -> HeroicAuthConfig {
    let config_path = get_heroic_config_path();
    let data = fs::read(config_path).expect("Failed to read heroic auth file");
    let data = data.as_slice();

    let parsed: HeroicAuthConfig =
        serde_json::from_slice(data).expect("Heroic auth file is corrupted");
    parsed
}
