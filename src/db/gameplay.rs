use crate::api::gog::achievements::Achievement;
use crate::api::gog::stats::{FieldValue, Stat};
use crate::api::handlers::context::HandlerContext;
use crate::paths;
use log::info;
use sqlx::sqlite::SqliteRow;
use sqlx::{Acquire, Error, Row, SqlitePool};

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

    info!("Setting up database at {:?}", database_file);
    let url = String::from("sqlite:") + database_file.to_str().unwrap();

    SqlitePool::connect(&url).await
}

pub async fn has_statistics(context: &HandlerContext) -> bool {
    let database = context.db_connection();
    let mut connection = database.acquire().await;
    if let Err(_) = connection {
        return false;
    }
    let mut connection = connection.unwrap();
    let res = sqlx::query("SELECT * FROM database_info WHERE key='stats_retrieved'")
        .fetch_one(&mut *connection)
        .await;

    match res {
        Ok(result) => {
            let value = result
                .try_get("value")
                .unwrap_or("0")
                .parse::<u8>()
                .unwrap();
            !result.is_empty() && value != 0
        }
        Err(_) => false,
    }
}

pub async fn get_statistics(
    context: &HandlerContext,
    only_changed: bool,
) -> Result<Vec<Stat>, Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;
    let mut stats: Vec<Stat> = Vec::new();
    let int_stats = sqlx::query(
        r#"SELECT s.id, s.key, s.increment_only,
        i.value, i.default_value, i.min_value, i.max_value, i.max_change
        FROM int_statistic AS i
        JOIN statistic AS s
        ON s.id = i.id
        WHERE ($1=1 AND s.changed=1) OR ($1=0 AND 1)"#,
    )
    .bind(only_changed as u8)
    .fetch_all(&mut *connection)
    .await?;
    let float_stats = sqlx::query(
        r#"SELECT s.id, s.key, s.type, s.increment_only,
        f.value, f.default_value, f.min_value, f.max_value, f.max_change, f.window
        FROM float_statistic AS f
        JOIN statistic AS s
        ON s.id = f.id
        WHERE ($1=1 AND s.changed=1) OR ($1=0 AND 1)"#,
    )
    .bind(only_changed as u8)
    .fetch_all(&mut *connection)
    .await?;

    for int_stat in int_stats {
        let id: i64 = int_stat.try_get("id").unwrap();
        let key: String = int_stat.try_get("key").unwrap();
        let increment_only: u8 = int_stat.try_get("increment_only").unwrap();
        let values = FieldValue::INT {
            value: int_stat.try_get("value").unwrap(),
            default_value: int_stat.try_get("default_value").unwrap(),
            min_value: int_stat.try_get("min_value").unwrap(),
            max_value: int_stat.try_get("max_value").unwrap(),
            max_change: int_stat.try_get("max_change").unwrap(),
        };
        let new_stat = Stat::new(id.to_string(), key, None, increment_only == 1, values);
        stats.push(new_stat)
    }

    for float_stat in float_stats {
        let id: i64 = float_stat.try_get("id").unwrap();
        let key: String = float_stat.try_get("key").unwrap();
        let increment_only: u8 = float_stat.try_get("increment_only").unwrap();
        let window: Option<f64> = float_stat.try_get("window").unwrap();
        let value_type: String = float_stat.try_get("type").unwrap();
        let values: FieldValue = match value_type.as_str() {
            "FLOAT" => FieldValue::FLOAT {
                value: float_stat.try_get("value").unwrap(),
                default_value: float_stat.try_get("default_value").unwrap(),
                min_value: float_stat.try_get("min_value").unwrap(),
                max_value: float_stat.try_get("max_value").unwrap(),
                max_change: float_stat.try_get("max_change").unwrap(),
            },
            "AVGRATE" => FieldValue::AVGRATE {
                value: float_stat.try_get("value").unwrap(),
                default_value: float_stat.try_get("default_value").unwrap(),
                min_value: float_stat.try_get("min_value").unwrap(),
                max_value: float_stat.try_get("max_value").unwrap(),
                max_change: float_stat.try_get("max_change").unwrap(),
            },
            _ => panic!("Unsupported value type"),
        };
        let new_stat = Stat::new(id.to_string(), key, window, increment_only == 1, values);
        stats.push(new_stat)
    }

    Ok(stats)
}

pub async fn set_statistics(context: &HandlerContext, stats: &Vec<Stat>) -> Result<(), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;
    let mut transaction = connection.begin().await?;

    sqlx::query("DELETE FROM int_statistic; DELETE FROM float_statistic; DELETE FROM statistic;")
        .execute(&mut *transaction)
        .await?;

    for stat in stats {
        let stat_id = stat.stat_id().parse::<i64>().unwrap();
        let stat_type = match stat.values() {
            FieldValue::INT { .. } => "INT",
            FieldValue::FLOAT { .. } => "FLOAT",
            FieldValue::AVGRATE { .. } => "AVGRATE",
        };
        sqlx::query("INSERT INTO statistic VALUES ($1, $2, $3, $4, 0)")
            .bind(stat_id)
            .bind(stat.stat_key())
            .bind(stat_type)
            .bind(stat.increment_only().to_owned() as u8)
            .execute(&mut *transaction)
            .await?;

        match stat.values() {
            FieldValue::INT {
                value,
                default_value,
                max_value,
                min_value,
                max_change,
            } => {
                sqlx::query("INSERT INTO int_statistic VALUES ($1, $2, $3, $4, $5, $6)")
                    .bind(stat_id)
                    .bind(value)
                    .bind(default_value.unwrap_or_else(|| 0))
                    .bind(min_value)
                    .bind(max_value)
                    .bind(max_change)
                    .execute(&mut *transaction)
                    .await?;
            }

            FieldValue::FLOAT {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            }
            | FieldValue::AVGRATE {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            } => {
                sqlx::query("INSERT INTO float_statistic VALUES ($1, $2, $3, $4, $5, $6, $7)")
                    .bind(stat_id)
                    .bind(value)
                    .bind(default_value.unwrap_or_else(|| 0.0))
                    .bind(min_value)
                    .bind(max_value)
                    .bind(max_change)
                    .bind(stat.window())
                    .execute(&mut *transaction)
                    .await?;
            }
        }
    }

    let _ = sqlx::query("INSERT INTO database_info VALUES ('stats_retrieved', '1')")
        .execute(&mut *transaction)
        .await;

    let _ = sqlx::query("UPDATE database_info SET value='1' WHERE key='stats_retrieved'")
        .execute(&mut *transaction)
        .await;

    transaction.commit().await?;
    Ok(())
}

pub async fn set_stat_float(
    context: &HandlerContext,
    stat_id: i64,
    value: f32,
) -> Result<(), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;

    sqlx::query("UPDATE float_statistic SET value=$1 WHERE id=$2; UPDATE statistic SET changed=1 WHERE id=$2;")
        .bind(value)
        .bind(stat_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}
pub async fn set_stat_int(context: &HandlerContext, stat_id: i64, value: i32) -> Result<(), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;

    sqlx::query("UPDATE int_statistic SET value=$1 WHERE id=$2; UPDATE statistic SET changed=1 WHERE id=$2;")
        .bind(value)
        .bind(stat_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn has_achievements(context: &HandlerContext) -> bool {
    let database = context.db_connection();
    let mut connection = database.acquire().await;
    if let Err(_) = connection {
        return false;
    }
    let mut connection = connection.unwrap();
    let res = sqlx::query("SELECT * FROM database_info WHERE key='achievements_retrieved'")
        .fetch_one(&mut *connection)
        .await;

    match res {
        Ok(result) => {
            let value = result
                .try_get("value")
                .unwrap_or("0")
                .parse::<u8>()
                .unwrap();
            !result.is_empty() && value != 0
        }
        Err(_) => false,
    }
}

fn achievement_from_database_row(row: SqliteRow) -> Achievement {
    let visible: u8 = row.try_get("visible_while_locked").unwrap();
    let achievement_id: i64 = row.try_get("id").unwrap();
    Achievement::new(
        achievement_id.to_string(),
        row.try_get("key").unwrap(),
        row.try_get("name").unwrap(),
        row.try_get("description").unwrap(),
        row.try_get("image_url_locked").unwrap(),
        row.try_get("image_url_unlocked").unwrap(),
        visible == 1,
        row.try_get("unlock_time").unwrap(),
        row.try_get("rarity").unwrap(),
        row.try_get("rarity_level_description").unwrap(),
        row.try_get("rarity_level_slug").unwrap(),
    )
}

pub async fn get_achievements(
    context: &HandlerContext,
    only_changed: bool,
) -> Result<(Vec<Achievement>, String), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;
    let mut achievements: Vec<Achievement> = Vec::new();

    let mode_res = sqlx::query("SELECT * FROM database_info WHERE key='achievements_mode'")
        .fetch_one(&mut *connection)
        .await?;
    let achievements_mode = mode_res.try_get("value")?;

    let db_achievements = sqlx::query(
        r#"SELECT id, key, name, description, visible_while_locked,
        unlock_time, image_url_locked, image_url_unlocked, rarity,
        rarity_level_description, rarity_level_slug
        FROM achievement WHERE ($1=1 AND changed=1) OR ($1=0 AND 1)"#,
    )
    .bind(only_changed as u8)
    .fetch_all(&mut *connection)
    .await?;

    for row in db_achievements {
        let new_achievement = achievement_from_database_row(row);
        achievements.push(new_achievement);
    }

    Ok((achievements, achievements_mode))
}

pub async fn set_achievements(
    context: &HandlerContext,
    achievements: &Vec<Achievement>,
    mode: &str,
) -> Result<(), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;
    let mut transaction = connection.begin().await?;

    sqlx::query("DELETE FROM achievement;")
        .execute(&mut *transaction)
        .await?;

    for achievement in achievements {
        let achievement_id = achievement.achievement_id().parse::<i64>().unwrap();

        sqlx::query(
            "INSERT INTO achievement VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, $9, $10, $11)",
        )
        .bind(achievement_id)
        .bind(achievement.achievement_key())
        .bind(achievement.name())
        .bind(achievement.description())
        .bind(*achievement.visible() as u32)
        .bind(achievement.date_unlocked())
        .bind(achievement.image_url_locked())
        .bind(achievement.image_url_unlocked())
        .bind(achievement.rarity())
        .bind(achievement.rarity_level_description())
        .bind(achievement.rarity_level_slug())
        .execute(&mut *transaction)
        .await?;
    }

    let previously_retrieved =
        sqlx::query("SELECT * FROM database_info WHERE key='achievements_retrieved'")
            .fetch_optional(&mut *transaction)
            .await?;

    if let None = previously_retrieved {
        sqlx::query("INSERT INTO database_info VALUES ('achievements_retrieved', '1'), ('achievements_mode', $1)")
            .bind(mode)
            .execute(&mut *transaction)
            .await?;
    } else {
        sqlx::query(
            "UPD.to_string()ATE database_info SET value='1' WHERE key='achievements_retrieved'",
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query("UPDATE database_info SET value='$1' WHERE key='achievements_mode'")
            .bind(mode)
            .execute(&mut *transaction)
            .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn get_achievement(
    context: &HandlerContext,
    achievement_id: i64,
) -> Result<Achievement, Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;

    let result = sqlx::query(
        r#"SELECT id, key, name, description, visible_while_locked,
        unlock_time, image_url_locked, image_url_unlocked, rarity,
        rarity_level_description, rarity_level_slug
        FROM achievement WHERE id=$1"#,
    )
    .bind(achievement_id)
    .fetch_one(&mut *connection)
    .await?;

    Ok(achievement_from_database_row(result))
}

pub async fn set_achievement(
    context: &HandlerContext,
    achievement_id: i64,
    date_unlocked: Option<String>,
) -> Result<(), Error> {
    let database = context.db_connection();
    let mut connection = database.acquire().await?;

    sqlx::query("UPDATE achievement SET changed=1, unlock_time=$1 WHERE id=$2")
        .bind(date_unlocked)
        .bind(achievement_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}
