use crate::api::handlers::context::HandlerContext;
use derive_getters::Getters;
use log::debug;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Getters, Debug)]
pub struct LeaderboardDefinition {
    id: String,
    key: String,
    name: String,
    sort_method: String,
    display_type: String,
    //locale: String,
    //is_localized: bool,
}

impl LeaderboardDefinition {
    pub fn new(
        id: String,
        key: String,
        name: String,
        sort_method: String,
        display_type: String,
    ) -> Self {
        Self {
            id,
            key,
            name,
            sort_method,
            display_type,
        }
    }
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

    let response = reqwest_client
        .get(new_url)
        .bearer_auth(token.access_token)
        .header("X-Gog-Lc", crate::LOCALE.as_str())
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
    pub details: Option<String>,
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

    let response = reqwest_client
        .get(new_url)
        .bearer_auth(token.access_token)
        .header("X-Gog-Lc", crate::LOCALE.as_str())
        .send()
        .await?;

    let response = response.error_for_status()?;

    response.json().await
}

#[derive(Serialize)]
struct LeaderboardScoreUpdate {
    pub score: i32,
    pub force: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Deserialize)]
pub struct LeaderboardScoreUpdateResponse {
    pub old_rank: u32,
    pub new_rank: u32,
    pub leaderboard_entry_total_count: u32,
}

pub async fn post_leaderboard_score(
    context: &HandlerContext,
    reqwest_client: &Client,
    user_id: &str,
    leaderboard_id: i64,
    score: i32,
    force_update: bool,
    details: Option<String>,
) -> Result<LeaderboardScoreUpdateResponse, reqwest::Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/leaderboards/{}",
        &client_id, user_id, leaderboard_id
    );

    let payload = LeaderboardScoreUpdate {
        score,
        force: force_update,
        details,
    };

    let response = reqwest_client
        .post(url)
        .json(&payload)
        .bearer_auth(token.access_token)
        .send()
        .await?;

    let response = response.error_for_status()?;
    let data = response.json().await?;
    Ok(data)
}

#[derive(Serialize)]
struct CreateLeaderboardPayload {
    pub key: String,
    pub name: String,
    pub sort_method: String,
    pub display_type: String,
}

pub async fn create_leaderboard(
    context: &HandlerContext,
    reqwest_client: &Client,
    key: String,
    name: String,
    sort_method: String,
    display_type: String,
) -> Result<String, reqwest::Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let payload = CreateLeaderboardPayload {
        key,
        name,
        sort_method,
        display_type,
    };

    let url = format!(
        "https://gameplay.gog.com/clients/{}/leaderboards",
        client_id
    );

    let response = reqwest_client
        .post(url)
        .json(&payload)
        .bearer_auth(token.access_token)
        .send()
        .await?;
    let response = response.error_for_status()?;

    let definition: LeaderboardDefinition = response.json().await?;

    Ok(definition.id)
}
