use derive_getters::Getters;
use crate::constants::TokenStorage;
use crate::db;
use sqlx::SqlitePool;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

#[derive(Getters)]
pub struct HandlerContext {
    is_online: bool,
    socket: TcpStream,
    shutdown_token: CancellationToken,
    token_store: TokenStorage,
    db_connected: bool,
    #[getter(skip)]
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
            is_online: false,
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

    pub fn db_connection(&self) -> SqlitePool {
        let connection = self.db_connection.clone();
        connection.unwrap()
    }
}
