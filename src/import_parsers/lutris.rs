use std::{env, fs, path};

fn get_config_path() -> Option<path::PathBuf> {
    let home_dir = env::var("HOME").unwrap();
    let home_dir = path::Path::new(&home_dir);

    let cache_path = env::var("XDG_CACHE_HOME").unwrap_or_else(|_e| home_dir.join(".cache").to_str().unwrap().to_owned());
    let cache_path  = path::Path::new(&cache_path);

    let host_path = cache_path.join("lutris/.gog.token");
    let flatpak_path = home_dir.join(".var/app/net.lutris.Lutris/cache/lutris/.gog.token");

    if host_path.exists() {
        Some(host_path)
    } else if flatpak_path.exists() {
        Some(flatpak_path)
    } else {
        None
    }
}

pub fn load_tokens() -> serde_json::Value {
    let config_path = get_config_path().expect("No lutris tokens found");
    log::debug!("Loading Lutris credentials from {:?}", config_path);
    let data = fs::read(config_path).expect("Failed to read lutris token file");
    let data = data.as_slice();

    let parsed: serde_json::Value = serde_json::from_slice(data).expect("Failed to parse lutris token file");
    parsed
}