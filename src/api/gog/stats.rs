use crate::api::handlers::context::HandlerContext;
use crate::api::handlers::error::{MessageHandlingError, MessageHandlingErrorKind};
use crate::constants::TokenStorage;
use derive_getters::Getters;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum FieldValue {
    Int {
        value: i32,
        min_value: Option<i32>,
        max_value: Option<i32>,
        max_change: Option<i32>,
        default_value: Option<i32>,
    },
    Float {
        value: f32,
        min_value: Option<f32>,
        max_value: Option<f32>,
        max_change: Option<f32>,
        default_value: Option<f32>,
    },
    Avgrate {
        value: f32,
        min_value: Option<f32>,
        max_value: Option<f32>,
        max_change: Option<f32>,
        default_value: Option<f32>,
    },
}

#[derive(Deserialize, Debug, Getters)]
pub struct Stat {
    stat_id: String,
    stat_key: String,
    window: Option<f64>,
    increment_only: bool,
    #[serde(flatten)]
    values: FieldValue,
}

impl Stat {
    pub fn new(
        stat_id: String,
        stat_key: String,
        window: Option<f64>,
        increment_only: bool,
        values: FieldValue,
    ) -> Self {
        Self {
            stat_id,
            stat_key,
            window,
            increment_only,
            values,
        }
    }
}

#[derive(Deserialize, Debug)]
struct StatsResponse {
    total_count: u32,
    items: Vec<Stat>,
}

pub async fn fetch_stats(
    token_store: &TokenStorage,
    client_id: &str,
    user_id: &str,
    reqwest_client: &Client,
) -> Result<Vec<Stat>, MessageHandlingError> {
    let lock = token_store.lock().await;
    let token = lock
        .get(client_id)
        .ok_or(MessageHandlingError::new(
            MessageHandlingErrorKind::Unauthorized,
        ))?
        .clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/stats",
        client_id, user_id
    );
    let response = reqwest_client
        .get(url)
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    let stats_data = response
        .json::<StatsResponse>()
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    Ok(stats_data.items)
}

#[derive(Serialize)]
#[serde(untagged)]
enum UpdateStatRequestValueType {
    Float(f32),
    Int(i32),
}

#[derive(Serialize)]
struct UpdateStatRequest {
    value: UpdateStatRequestValueType,
}

impl UpdateStatRequest {
    pub fn new(value: UpdateStatRequestValueType) -> Self {
        Self { value }
    }
}

pub async fn update_stat(
    context: &HandlerContext,
    reqwest_client: &Client,
    user_id: &str,
    stat: &Stat,
) -> Result<(), Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().await.unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/stats/{}",
        &client_id,
        user_id,
        stat.stat_id()
    );
    let value_type = match stat.values {
        FieldValue::Float { value, .. } | FieldValue::Avgrate { value, .. } => {
            UpdateStatRequestValueType::Float(value)
        }
        FieldValue::Int { value, .. } => UpdateStatRequestValueType::Int(value),
    };
    let payload = UpdateStatRequest::new(value_type);
    let response = reqwest_client
        .post(url)
        .json(&payload)
        .bearer_auth(token.access_token)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}

pub async fn delete_stats(
    context: &HandlerContext,
    reqwest_client: &Client,
    user_id: &str,
) -> Result<(), Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().await.unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/stats",
        &client_id, user_id,
    );

    let response = reqwest_client
        .delete(url)
        .bearer_auth(token.access_token)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}
