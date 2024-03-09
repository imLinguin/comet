use crate::api::handlers::context::HandlerContext;
use derive_getters::Getters;
use log::debug;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Getters, Debug)]
pub struct LeaderboardDefinition {
    id: String,
    client_id: String,
    key: String,
    name: String,
    sort_method: String,
    display_type: String,
    trusted: bool,
    locale: String,
    is_localized: bool,
}

#[derive(Deserialize)]
pub struct LeaderboardsResponse {
    pub items: Vec<LeaderboardDefinition>,
}

pub async fn get_leaderboards(
    context: &HandlerContext,
    reqwest_client: &Client,
) -> Result<Vec<LeaderboardDefinition>, reqwest::Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);
    let url = format!(
        "https://gameplay.gog.com/clients/{}/leaderboards",
        &client_id
    );

    let auth_header = String::from("Bearer ") + &token.access_token;
    let response = reqwest_client
        .get(url)
        .header("Authorization", &auth_header)
        .header("X-Gog-Lc", "en-US") // TODO: Handle languages
        .send()
        .await?;

    let response_data: LeaderboardsResponse = response.json().await?;

    debug!("Got {} leaderboards", response_data.items.len());

    Ok(response_data.items)
}
