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
}

#[cfg(target_os = "windows")]
lazy_static! {
    static ref DATA_PATH: PathBuf = PathBuf::from(env::var("LOCALAPPDATA").unwrap()).join("comet");
}

lazy_static! {
    pub static ref GAMEPLAY_STORAGE: PathBuf = DATA_PATH.join("gameplay");
}
