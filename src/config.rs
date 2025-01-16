use crate::paths;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Configuration {
    pub overlay: OverlayConfiguration,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct OverlayConfiguration {
    #[serde(default = "default_volume")]
    pub notification_volume: u32,
    pub position: OverlayPosition,
    pub notifications: OverlayNotifications,
}

impl Default for OverlayConfiguration {
    fn default() -> Self {
        Self {
            notification_volume: 50,
            position: Default::default(),
            notifications: Default::default(),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl std::fmt::Display for OverlayPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TopLeft => f.write_str("top_left"),
            Self::TopRight => f.write_str("top_right"),
            Self::BottomLeft => f.write_str("bottom_left"),
            Self::BottomRight => f.write_str("bottom_right"),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct OverlayNotifications {
    pub chat: OverlayNotificationConfig,
    pub friend_online: OverlayNotificationConfig,
    pub friend_invite: OverlayNotificationConfig,
    pub friend_game_start: OverlayNotificationConfig,
    pub game_invite: OverlayNotificationConfig,
}

#[derive(Deserialize, Debug)]
pub struct OverlayNotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
}

impl Default for OverlayNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_volume() -> u32 {
    50
}

pub fn load_config() -> Result<Configuration, Box<dyn std::error::Error + Send + Sync>> {
    let data = fs::read_to_string(paths::CONFIG_FILE.as_path())?;
    Ok(toml::from_str(&data)?)
}
