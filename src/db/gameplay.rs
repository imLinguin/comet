use crate::paths;
use log::debug;
use sqlx::{Error, SqlitePool};

pub const SETUP_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS `leaderboard` (`id` INTEGER PRIMARY KEY NOT NULL,`key` TEXT UNIQUE NOT NULL,`name` TEXT NOT NULL,`sort_method` TEXT CHECK ( sort_method IN ( 'SORT_METHOD_ASCENDING', 'SORT_METHOD_DESCENDING' ) ) NOT NULL,`display_type` TEXT CHECK ( display_type IN ( 'DISPLAY_TYPE_NUMERIC', 'DISPLAY_TYPE_TIME_SECONDS', 'DISPLAY_TYPE_TIME_MILLISECONDS' ) ) NOT NULL,`score` INTEGER NOT NULL DEFAULT 0,`rank` INTEGER NOT NULL DEFAULT 0,`force_update` INTEGER CHECK ( force_update IN ( 0, 1 ) ) NOT NULL DEFAULT 0,`changed` INTEGER CHECK ( changed IN ( 0, 1 ) ) NOT NULL, entry_total_count INTEGER NOT NULL DEFAULT 0, details TEXT NOT NULL DEFAULT "");
CREATE TABLE IF NOT EXISTS `achievement` (`id` INTEGER PRIMARY KEY NOT NULL,`key` TEXT UNIQUE NOT NULL,`name` TEXT NOT NULL,`description` TEXT NOT NULL,`visible_while_locked` INTEGER CHECK ( visible_while_locked IN ( 0, 1 ) ) NOT NULL,`unlock_time` TEXT,`image_url_locked` TEXT NOT NULL,`image_url_unlocked` TEXT NOT NULL,`changed` INTEGER CHECK ( changed IN ( 0, 1 ) ) NOT NULL, rarity REAL NOT NULL DEFAULT 0.0, rarity_level_description TEXT NOT NULL DEFAULT "", rarity_level_slug TEXT NOT NULL DEFAULT "");
CREATE TABLE IF NOT EXISTS `statistic` (`id` INTEGER PRIMARY KEY NOT NULL,`key` TEXT UNIQUE NOT NULL,`type` TEXT CHECK ( type IN ( 'INT', 'FLOAT', 'AVGRATE' ) ) NOT NULL,`increment_only` INTEGER CHECK ( increment_only IN ( 0, 1 ) ) NOT NULL,`changed` INTEGER CHECK ( changed IN ( 0, 1 ) ) NOT NULL);
CREATE INDEX IF NOT EXISTS `is_leaderboard_score_changed` on leaderboard (changed);
CREATE INDEX IF NOT EXISTS `is_achievement_changed` ON achievement (changed);
CREATE INDEX IF NOT EXISTS `is_statistic_changed` ON statistic (changed);
CREATE TABLE IF NOT EXISTS `game_info` (`time_played` INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS `int_statistic` (`id` INTEGER REFERENCES statistic ( id ) NOT NULL,`value` INTEGER NOT NULL DEFAULT 0,`default_value` INTEGER NOT NULL DEFAULT 0,`min_value` INTEGER,`max_value` INTEGER,`max_change` INTEGER);
CREATE TABLE IF NOT EXISTS `float_statistic` (`id` INTEGER REFERENCES statistic ( id ) NOT NULL,`value` REAL NOT NULL DEFAULT 0,`default_value` REAL NOT NULL DEFAULT 0,`min_value` REAL,`max_value` REAL,`max_change` REAL,`window` REAL DEFAULT NULL);
CREATE TABLE IF NOT EXISTS `database_info` (`key` TEXT PRIMARY KEY NOT NULL,`value` TEXT NOT NULL);
"#;

pub async fn setup_connection(client_id: &str, user_id: &str) -> Result<SqlitePool, Error> {
    let databases_path = paths::GAMEPLAY_STORAGE.join(client_id).join(user_id);
    let database_file = databases_path.join("gameplay.db");
    if !databases_path.exists() {
        let _ = tokio::fs::create_dir_all(&databases_path).await;
    }

    if !database_file.exists() {
        let _ = tokio::fs::File::create(&database_file).await;
    }

    debug!("Setting up database at {:?}", database_file);
    let url = String::from("sqlite:") + database_file.to_str().unwrap();

    SqlitePool::connect(&url).await
}
