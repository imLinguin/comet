use crate::api::handlers::context::HandlerContext;
use clap::builder::Str;
use derive_getters::Getters;
use reqwest::{Client, Error};
use serde::Deserialize;
use serde::__private::de::borrow_cow_bytes;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum FieldValue {
    INT {
        value: i32,
        min_value: Option<i32>,
        max_value: Option<i32>,
        max_change: Option<i32>,
        default_value: Option<i32>,
    },
    FLOAT {
        value: f32,
        min_value: Option<f32>,
        max_value: Option<f32>,
        max_change: Option<f32>,
        default_value: Option<f32>,
    },
    AVGRATE {
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
    context: &HandlerContext,
    user_id: &str,
    reqwest_client: &Client,
) -> Result<Vec<Stat>, Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/stats",
        &client_id, user_id
    );
    let auth_header = String::from("Bearer ") + &token.access_token;
    let response = reqwest_client
        .get(url)
        .header("Authorization", &auth_header)
        .send()
        .await?;

    let stats_data = response.json::<StatsResponse>().await?;

    Ok(stats_data.items)
}
