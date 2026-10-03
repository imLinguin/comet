use reqwest::Client;
use serde::Deserialize;

use crate::{api::handlers::error::MessageHandlingError, constants::TokenStorage};

#[derive(Deserialize)]
pub struct SessionsResponse {
    pub time_sum: u32,
}

pub async fn fetch_sessions(
    token_store: &TokenStorage,
    client_id: &str,
    user_id: &str,
    reqwest_client: &Client,
) -> Result<u32, MessageHandlingError> {
    let token = {
        let lock = token_store.lock().await;
        lock.get(client_id)
            .ok_or(MessageHandlingError::unauthorized())?
            .clone()
    };

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/sessions",
        client_id, user_id
    );

    let response = reqwest_client
        .get(url)
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(MessageHandlingError::network)?;

    let sessions_res: SessionsResponse = response
        .json()
        .await
        .map_err(MessageHandlingError::network)?;

    Ok(sessions_res.time_sum)
}
