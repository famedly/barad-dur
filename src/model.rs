use chrono::{serde::ts_seconds_option, DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, PartialEq, FromRow, Clone)]
pub struct Report {
    pub homeserver: Option<String>,
    #[serde(with = "ts_seconds_option", default)]
    pub local_timestamp: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option", rename(deserialize = "timestamp"), default)]
    pub remote_timestamp: Option<DateTime<Utc>>,
    pub remote_addr: Option<String>,
    pub forwarded_for: Option<String>,
    pub uptime_seconds: Option<i64>,
    pub total_users: Option<i64>,
    pub total_nonbridged_users: Option<i64>,
    pub total_room_count: Option<i64>,
    pub daily_active_users: Option<i64>,
    pub daily_active_rooms: Option<i64>,
    pub daily_messages: Option<i64>,
    pub daily_sent_messages: Option<i64>,
    pub daily_active_e2ee_rooms: Option<i64>,
    pub daily_e2ee_messages: Option<i64>,
    pub daily_sent_e2ee_messages: Option<i64>,
    pub monthly_active_users: Option<i64>,
    pub r30_users_all: Option<i64>,
    pub r30_users_android: Option<i64>,
    pub r30_users_ios: Option<i64>,
    pub r30_users_electron: Option<i64>,
    pub r30_users_web: Option<i64>,
    pub r30v2_users_all: Option<i64>,
    pub r30v2_users_android: Option<i64>,
    pub r30v2_users_ios: Option<i64>,
    pub r30v2_users_electron: Option<i64>,
    pub r30v2_users_web: Option<i64>,
    pub cpu_average: Option<i64>,
    pub memory_rss: Option<i64>,
    pub cache_factor: Option<f64>,
    pub event_cache_size: Option<i64>,
    pub user_agent: Option<String>,
    pub daily_user_type_native: Option<i64>,
    pub daily_user_type_bridged: Option<i64>,
    pub daily_user_type_guest: Option<i64>,
    pub python_version: Option<String>,
    pub database_engine: Option<String>,
    pub database_server_version: Option<String>,
    pub server_context: Option<String>,
    pub log_level: Option<String>,
}
