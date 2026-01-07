use std::env;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
lazy_static! {
    static ref DATA_PATH: PathBuf = match env::var("XDG_DATA_HOME") {
        Ok(path) => {
            PathBuf::from(path).join("comet")
        }
        Err(_) => {
            let home = env::var("HOME").unwrap();
            PathBuf::from(home).join(".local/share/comet")
        }
    };
    static ref CONFIG_PATH: PathBuf = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => {
            PathBuf::from(path).join("comet")
        }
        Err(_) => {
            let home = env::var("HOME").unwrap();
            PathBuf::from(home).join(".config/comet")
        }
    };
}

#[cfg(target_os = "windows")]
lazy_static! {
    static ref DATA_PATH: PathBuf = PathBuf::from(env::var("LOCALAPPDATA").unwrap()).join("comet");
    static ref CONFIG_PATH: PathBuf = PathBuf::from(env::var("APPDATA").unwrap()).join("comet");
}

#[cfg(target_os = "macos")]
lazy_static! {
    static ref DATA_PATH: PathBuf =
        PathBuf::from(env::var("HOME").unwrap()).join("Library/Application Support/comet");
    static ref CONFIG_PATH: PathBuf = DATA_PATH.clone();
}

lazy_static! {
    pub static ref GAMEPLAY_STORAGE: PathBuf = DATA_PATH.join("gameplay");
    pub static ref REDISTS_STORAGE: PathBuf = DATA_PATH.join("redist");
    pub static ref WORKAROUNDS: PathBuf = DATA_PATH.join("workarounds");
    pub static ref CONFIG_FILE: PathBuf = CONFIG_PATH.join("config.toml");
}
