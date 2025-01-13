use crate::api::handlers::context::HandlerContext;
use crate::api::handlers::error::{MessageHandlingError, MessageHandlingErrorKind};
use crate::constants::TokenStorage;
use derive_getters::Getters;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Getters)]
pub struct Achievement {
    pub achievement_id: String,
    pub achievement_key: String,
    pub name: String,
    pub description: String,
    pub image_url_locked: String,
    pub image_url_unlocked: String,
    pub visible: bool,
    pub date_unlocked: Option<String>,
    pub rarity: f32,
    pub rarity_level_description: String,
    pub rarity_level_slug: String,
}

impl Achievement {
    #[allow(clippy::too_many_arguments)]
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
    token_store: &TokenStorage,
    client_id: &str,
    user_id: &str,
    reqwest_client: &Client,
) -> Result<(Vec<Achievement>, String), MessageHandlingError> {
    let lock = token_store.lock().await;
    let token = lock
        .get(client_id)
        .ok_or(MessageHandlingError::new(
            MessageHandlingErrorKind::Unauthorized,
        ))?
        .clone();
    drop(lock);

    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/achievements",
        client_id, user_id
    );
    let response = reqwest_client
        .get(url)
        .bearer_auth(token.access_token)
        .header("X-Gog-Lc", crate::LOCALE.as_str())
        .send()
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    let achievements_data = response
        .json::<AchievementsResponse>()
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    Ok((achievements_data.items, achievements_data.achievements_mode))
}

#[derive(Serialize)]
struct SetAchievementRequest {
    date_unlocked: Option<String>,
}

impl SetAchievementRequest {
    pub fn new(date_unlocked: Option<String>) -> Self {
        Self { date_unlocked }
    }
}

pub async fn set_achievement(
    context: &HandlerContext,
    reqwest_client: &Client,
    user_id: &str,
    achievement_id: &str,
    date_unlocked: Option<String>,
) -> Result<(), Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().await.unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);
    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/achievements/{}",
        &client_id, user_id, achievement_id
    );
    let body = SetAchievementRequest::new(date_unlocked);

    let response = reqwest_client
        .post(url)
        .json(&body)
        .bearer_auth(token.access_token)
        .send()
        .await?;
    response.error_for_status()?;
    Ok(())
}

pub async fn delete_achievements(
    context: &HandlerContext,
    reqwest_client: &Client,
    user_id: &str,
) -> Result<(), Error> {
    let lock = context.token_store().lock().await;
    let client_id = context.client_id().await.unwrap();
    let token = lock.get(&client_id).unwrap().clone();
    drop(lock);
    let url = format!(
        "https://gameplay.gog.com/clients/{}/users/{}/achievements",
        &client_id, user_id
    );

    let response = reqwest_client
        .delete(url)
        .bearer_auth(token.access_token)
        .send()
        .await?;
    response.error_for_status()?;

    Ok(())
}
