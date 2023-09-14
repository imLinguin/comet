pub mod gog;
pub mod handlers;
pub mod notification_pusher;

pub mod structs {
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct Token {
        pub access_token: String,
        pub refresh_token: String,
    }
    impl Token {
        pub fn new(access_token: String, refresh_token: String) -> Token {
            Token {
                access_token,
                refresh_token,
            }
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct UserInfo {
        pub username: String,
        #[serde(rename = "galaxyUserId")]
        pub galaxy_user_id: String,
    }
}
