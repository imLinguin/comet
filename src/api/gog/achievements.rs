use crate::api::handlers::context::HandlerContext;
use derive_getters::Getters;
use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Deserialize, Debug, Getters)]
pub struct Achievement {
    achievement_id: String,
    achievement_key: String,
    name: String,
    description: String,
    image_url_locked: String,
    image_url_unlocked: String,
    visible: bool,
    date_unlocked: Option<String>,
    rarity: f32,
    rarity_level_description: String,
    rarity_level_slug: String,
}

impl Achievement {
    pub fn new(
        achievement_id: String,
        achievement_key: String,
        name: String,
        description: String,
        image_url_locked: String,
        image_url_unlocked: String,
        visible: bool,
        date_unlocked: Option<String>,
        rarity: f32,
        rarity_level_description: String,
        rarity_level_slug: String,
    ) -> Self {
        Self {
            achievement_id,
            achievement_key,
            name,
            description,
            image_url_unlocked,
            image_url_locked,
            visible,
            date_unlocked,
            rarity,
            rarity_level_slug,
            rarity_level_description,
        }
    }
}

#[derive(Deserialize, Debug, Getters)]
pub struct AchievementsResponse {
    total_count: u32,
    limit: u32,
    page_token: String,
    items: Vec<Achievement>,
    achievements_mode: String,
}

pub async fn fetch_achievements(
    context: &HandlerContext,
    user_id: &str,
    reqwest_client: &Client,
) -> Result<(Vec<Achievement>, String), Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().clone().unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/achievements",
        &client_id, user_id
    );
    let auth_header = String::from("Bearer ") + &token.access_token;
    let response = reqwest_client
        .get(url)
        .header("Authorization", &auth_header)
        .header("X-Gog-Lc", "en-US") // TODO: Handle languages
        .send()
        .await?;

    let achievements_data = response.json::<AchievementsResponse>().await?;

    Ok((achievements_data.items, achievements_data.achievements_mode))
}
