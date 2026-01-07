#[derive(Clone)]
pub struct StatAchievementWorkaround {
    pub statistic_key: String,
    pub achievement_key: String,
    pub threshold: String, // Since we dont know the type of the stat we dont parse it
}
