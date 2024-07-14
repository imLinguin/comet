use serde::Deserialize;
use std::{env, fs, path};

#[derive(Deserialize)]
struct LutrisSection {
    cache_dir: Option<String>,
}

#[derive(Deserialize)]
struct LutrisConf {
    lutris: LutrisSection,
}

fn get_config_path() -> Option<path::PathBuf> {
    let home_dir = env::var("HOME").unwrap();
    let home_dir = path::Path::new(&home_dir);

    let config_path: path::PathBuf = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_e| home_dir.join(".config").to_str().unwrap().to_string())
        .into();
    let cache_path: path::PathBuf = env::var("XDG_CACHE_HOME")
        .unwrap_or_else(|_e| home_dir.join(".cache").to_str().unwrap().to_owned())
        .into();
    let lutris_conf = config_path.join("lutris/lutris.conf");
    let lutris_conf_data = fs::read_to_string(lutris_conf).or_else(|_| {
        // Fallback to flatpak's config
        fs::read_to_string(home_dir.join(".var/app/net.lutris.Lutris/config/lutris/lutris.conf"))
    });
    match lutris_conf_data {
        Ok(data) => {
            // Attempt to parse the config in search for custom cache_dir
            match serde_ini::from_str::<LutrisConf>(&data) {
                Ok(config_data) => {
                    if let Some(dir) = config_data.lutris.cache_dir {
                        let dir: path::PathBuf = dir.into();
                        let token_path = dir.join(".gog.token");
                        // Check if token config exists, if it doesn't return None
                        if token_path.exists() {
                            return Some(token_path);
                        } else {
                            return None;
                        }
                    } else {
                        log::debug!("Cache dir not specified in lutris config");
                    }
                }
                Err(err) => {
                    log::warn!("Failed to parse lutris config: {:?}", err);
                }
            }
        }
        Err(err) => log::warn!("Failed to read lutris.conf: {:?}", err),
    }

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

    let parsed: serde_json::Value =
        serde_json::from_slice(data).expect("Failed to parse lutris token file");
    parsed
}
