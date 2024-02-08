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
}
