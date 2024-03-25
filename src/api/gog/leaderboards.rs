use crate::api::handlers::context::HandlerContext;
use derive_getters::Getters;
use log::debug;
use reqwest::{Client, Url};
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

pub async fn get_leaderboards<I, K, V>(
    context: &HandlerContext,
    reqwest_client: &Client,
    params: I,
) -> Result<Vec<LeaderboardDefinition>, reqwest::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);
    let url = format!(
        "https://gameplay.gog.com/clients/{}/leaderboards",
        &client_id
    );

    let new_url = Url::parse_with_params(&url, params).unwrap();

    let auth_header = String::from("Bearer ") + &token.access_token;
    let response = reqwest_client
        .get(new_url)
        .header("Authorization", &auth_header)
        .header("X-Gog-Lc", "en-US") // TODO: Handle languages
        .send()
        .await?;

    let response_data: LeaderboardsResponse = response.json().await?;

    debug!("Got {} leaderboards", response_data.items.len());

    Ok(response_data.items)
}

#[derive(Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: String,
    pub rank: u32,
    pub score: u32,
}

#[derive(Deserialize)]
pub struct LeaderboardEntriesResponse {
    pub items: Vec<LeaderboardEntry>,
    pub leaderboard_entry_total_count: u32,
}

pub async fn get_leaderboards_entries<I, K, V>(
    context: &HandlerContext,
    reqwest_client: &Client,
    leaderboard_id: u64,
    params: I,
) -> Result<LeaderboardEntriesResponse, reqwest::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/leaderboards/{}/entries",
        &client_id, leaderboard_id
    );

    let new_url = Url::parse_with_params(&url, params).unwrap();

    let auth_header = String::from("Bearer ") + &token.access_token;
    let response = reqwest_client
        .get(new_url)
        .header("Authorization", &auth_header)
        .header("X-Gog-Lc", "en-US") // TODO: Handle languages
        .send()
        .await?;

    let response = response.error_for_status()?;

    response.json().await
}
