use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomEventType {
    AchievementUnlocked,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CustomEvent<T> {
    pub name: CustomEventType,
    pub client_id: String,
    pub data: T,
}
