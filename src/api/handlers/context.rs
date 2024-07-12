use crate::constants::TokenStorage;
use crate::db;
use derive_getters::Getters;
use sqlx::SqlitePool;
use tokio::net::TcpStream;

#[derive(Getters)]
pub struct HandlerContext {
    is_online: bool,
    socket: TcpStream,
    token_store: TokenStorage,
    db_connected: bool,
    #[getter(skip)]
    db_connection: Option<SqlitePool>,
    client_identified: bool,
    client_id: Option<String>,
    client_secret: Option<String>,
    updated_achievements: bool,
    updated_stats: bool,
    updated_leaderboards: bool,
}

impl HandlerContext {
    pub fn new(socket: TcpStream, token_store: TokenStorage) -> Self {
        Self {
            is_online: false,
            socket,
            token_store,
            db_connected: false,
            db_connection: None,
            client_identified: false,
            client_id: None,
            client_secret: None,
            updated_achievements: false,
            updated_stats: false,
            updated_leaderboards: true,
        }
    }

    pub fn socket_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    pub fn identify_client(&mut self, client_id: &str, client_secret: &str) {
        self.client_identified = true;
        self.client_id = Some(client_id.to_string());
        self.client_secret = Some(client_secret.to_string());
    }

    pub fn set_online(&mut self) {
        self.is_online = true
    }

    pub fn set_offline(&mut self) {
        self.is_online = false
    }

    pub fn set_updated_achievements(&mut self, value: bool) {
        self.updated_achievements = value
    }
    pub fn set_updated_stats(&mut self, value: bool) {
        self.updated_stats = value
    }
    pub fn set_updated_leaderboards(&mut self, value: bool) {
        self.updated_leaderboards = value
    }

    pub async fn setup_database(
        &mut self,
        client_id: &str,
        user_id: &str,
    ) -> Result<(), sqlx::Error> {
        if self.db_connected {
            return Ok(());
        }
        let connection = db::gameplay::setup_connection(client_id, user_id).await?;
        self.db_connection = Some(connection);

        let pool = self.db_connection.clone().unwrap();
        let mut connection = pool.acquire().await?;
        sqlx::query(db::gameplay::SETUP_QUERY)
            .execute(&mut *connection)
            .await?;

        // This may already exist, we don't care
        let _ = sqlx::query("INSERT INTO database_info VALUES ('language', 'en-US')")
            .execute(&mut *connection)
            .await; // TODO: Handle languages

        self.db_connected = true;
        Ok(())
    }

    pub fn db_connection(&self) -> SqlitePool {
        let connection = self.db_connection.clone();
        connection.unwrap()
    }
}
