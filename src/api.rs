pub mod gog;
pub mod handlers;
pub mod notification_pusher;

pub mod structs {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct Token {
        pub access_token: String,
        pub refresh_token: String,
        #[serde(skip, default = "Utc::now")]
        pub obtain_time: DateTime<Utc>,
    }
    impl Token {
        pub fn new(access_token: String, refresh_token: String) -> Self {
            Self {
                access_token,
                refresh_token,
                obtain_time: Utc::now(),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct UserInfo {
        pub username: String,
        #[serde(rename = "galaxyUserId")]
        pub galaxy_user_id: String,
    }

    #[derive(PartialEq)]
    pub enum DataSource {
        Online,
        Local,
    }

    pub enum IDType {
        Unassigned(u64),
        Lobby(u64),
        User(u64),
    }

    impl IDType {
        /// Parse entity id to this enum
        pub fn parse(id: u64) -> Self {
            let flag = id >> 56;
            let new_value = id << 8 >> 8;
            match flag {
                1 => Self::Lobby(new_value),
                2 => Self::User(new_value),
                _ => IDType::Unassigned(new_value),
            }
        }
        /// Return entity id with magic flag
        pub fn value(&self) -> u64 {
            match self {
                Self::Unassigned(id) => *id,
                Self::Lobby(id) => 1 << 56 | id,
                Self::User(id) => 2 << 56 | id,
            }
        }
        /// Return underlying entity id
        pub fn inner(&self) -> u64 {
            match self {
                Self::Unassigned(id) | Self::Lobby(id) | Self::User(id) => *id,
            }
        }
    }
}
