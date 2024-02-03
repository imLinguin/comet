use crate::constants::TokenStorage;
use crate::db;
use sqlx::SqlitePool;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

pub struct HandlerContext {
    socket: TcpStream,
    shutdown_token: CancellationToken,
    token_store: TokenStorage,
    db_connected: bool,
    db_connection: Option<SqlitePool>,
    client_identified: bool,
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl HandlerContext {
    pub fn new(
        socket: TcpStream,
        shutdown_token: CancellationToken,
        token_store: TokenStorage,
    ) -> Self {
        Self {
            socket,
            shutdown_token,
            token_store,
            db_connected: false,
            db_connection: None,
            client_identified: false,
            client_id: None,
            client_secret: None,
        }
    }

    pub fn socket(&self) -> &TcpStream {
        &self.socket
    }

    pub fn socket_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    pub fn token_store(&self) -> &TokenStorage {
        &self.token_store
    }

    pub fn shutdown_token(&self) -> &CancellationToken {
        &self.shutdown_token
    }

    pub fn identify_client(&mut self, client_id: &str, client_secret: &str) {
        self.client_identified = true;
        self.client_id = Some(client_id.to_string());
        self.client_secret = Some(client_secret.to_string());
    }

    pub async fn setup_database(
        &mut self,
        client_id: &str,
        user_id: &str,
    ) -> Result<(), sqlx::Error> {
        let connection = db::gameplay::setup_connection(client_id, user_id).await?;
        self.db_connection = Some(connection);

        let pool = self.db_connection.clone().unwrap();
        let mut connection = pool.acquire().await?;
        sqlx::query(db::gameplay::SETUP_QUERY)
            .execute(&mut *connection)
            .await?;

        self.db_connected = true;
        Ok(())
    }
}
