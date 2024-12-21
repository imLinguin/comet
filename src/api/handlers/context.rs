use crate::constants::TokenStorage;
use crate::db;
use derive_getters::Getters;
use sqlx::SqlitePool;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};

pub struct State {
    is_online: bool,
    client_identified: bool,
    client_id: Option<String>,
    client_secret: Option<String>,
    updated_achievements: bool,
    updated_stats: bool,
    updated_leaderboards: bool,
}

#[derive(Getters)]
pub struct HandlerContext {
    socket: Mutex<TcpStream>,
    token_store: TokenStorage,
    #[getter(skip)]
    db_connection: Mutex<Option<SqlitePool>>,
    #[getter(skip)]
    state: Mutex<State>,
}

impl HandlerContext {
    pub fn new(socket: TcpStream, token_store: TokenStorage) -> Self {
        let state = Mutex::new(State {
            is_online: false,
            client_identified: false,
            client_id: None,
            client_secret: None,
            updated_achievements: false,
            updated_stats: false,
            updated_leaderboards: true,
        });
        Self {
            socket: Mutex::new(socket),
            token_store,
            db_connection: Mutex::new(None),
            state,
        }
    }

    pub async fn socket_mut(&self) -> MutexGuard<'_, TcpStream> {
        self.socket.lock().await
    }

    pub async fn socket_read_u16(&self) -> Result<u16, std::io::Error> {
        self.socket.lock().await.read_u16().await
    }

    pub async fn identify_client(&self, client_id: &str, client_secret: &str) {
        let mut state = self.state.lock().await;
        state.client_identified = true;
        state.client_id = Some(client_id.to_string());
        state.client_secret = Some(client_secret.to_string());
    }

    pub async fn set_online(&self) {
        self.state.lock().await.is_online = true
    }

    pub async fn set_offline(&self) {
        self.state.lock().await.is_online = false
    }

    pub async fn set_updated_achievements(&self, value: bool) {
        self.state.lock().await.updated_achievements = value
    }
    pub async fn set_updated_stats(&self, value: bool) {
        self.state.lock().await.updated_stats = value
    }
    pub async fn set_updated_leaderboards(&self, value: bool) {
        self.state.lock().await.updated_leaderboards = value
    }

    pub async fn setup_database(&self, client_id: &str, user_id: &str) -> Result<(), sqlx::Error> {
        let mut db_con = self.db_connection.lock().await;
        if db_con.is_some() {
            return Ok(());
        }
        let connection = db::gameplay::setup_connection(client_id, user_id).await?;
        *db_con = Some(connection);

        let pool = db_con.clone().unwrap();
        let mut connection = pool.acquire().await?;
        sqlx::query(db::gameplay::SETUP_QUERY)
            .execute(&mut *connection)
            .await?;

        // This may already exist, we don't care
        let _ = sqlx::query("INSERT INTO database_info VALUES ('language', $1) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(crate::LOCALE.as_str())
            .execute(&mut *connection)
            .await;

        Ok(())
    }

    pub async fn db_connection(&self) -> SqlitePool {
        let connection = self.db_connection.lock().await.clone();
        connection.unwrap()
    }

    pub async fn client_id(&self) -> Option<String> {
        self.state.lock().await.client_id.clone()
    }

    pub async fn client_secret(&self) -> Option<String> {
        self.state.lock().await.client_secret.clone()
    }

    pub async fn is_online(&self) -> bool {
        self.state.lock().await.is_online
    }

    pub async fn client_identified(&self) -> bool {
        self.state.lock().await.client_identified
    }

    pub async fn updated_achievements(&self) -> bool {
        self.state.lock().await.updated_achievements
    }
    pub async fn updated_stats(&self) -> bool {
        self.state.lock().await.updated_stats
    }
    pub async fn updated_leaderboards(&self) -> bool {
        self.state.lock().await.updated_leaderboards
    }
}
