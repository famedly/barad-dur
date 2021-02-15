use std::process;

use anyhow::{Context, Result};
use chrono::Duration;
use tokio::{sync::mpsc::Receiver, time::interval};

use crate::model;

pub(crate) async fn aggregate_loop(pool: sqlx::PgPool) {
    let interval = &mut interval(Duration::days(1i64).to_std().unwrap());
    loop {
        interval.tick().await;
        if let Err(err) = aggregate_stats(&pool).await {
            log::error!("{:?}", err);
            process::exit(-1);
        }
    }
}

pub(crate) async fn insert_reports_loop(pool: sqlx::PgPool, rx: &mut Receiver<model::StatsReport>) {
    loop {
        let report = match rx
            .recv()
            .await
            .context("sender threads have been closed and no further reports available.")
        {
            Ok(report) => report,
            Err(err) => {
                log::error!("{:?}", err);
                process::exit(-1);
            }
        };

        if let Err(err) = save_report(&pool, &report)
            .await
            .context("failed writing report to database.")
        {
            log::error!("{:?}", err);
            process::exit(-1);
        }
    }
}

pub(crate) async fn connect_db(db_url: &str) -> sqlx::PgPool {
    match sqlx::PgPool::connect(&db_url)
        .await
        .context("failed connecting to PostgreSQL server.")
    {
        Ok(pool) => pool,
        Err(err) => {
            log::error!("{:?}", err);
            process::exit(-1);
        }
    }
}

async fn aggregate_stats(pool: &sqlx::PgPool) -> Result<()> {
    let _ = sqlx::query!(
        r#"
INSERT INTO aggregate_stats (
    day,
    daily_active_e2ee_rooms,
    daily_active_rooms,
    daily_active_users,
    daily_e2ee_messages,
    daily_messages,
    daily_sent_e2ee_messages,
    daily_sent_messages,
    daily_user_type_bridged,
    daily_user_type_guest,
    daily_user_type_native,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    total_nonbridged_users,
    total_room_count,
    total_users,
    daily_active_homeservers
)
SELECT
    local_timestamp::DATE,
    SUM(daily_active_e2ee_rooms),
    SUM(daily_active_rooms),
    SUM(daily_active_users),
    SUM(daily_e2ee_messages),
    SUM(daily_messages),
    SUM(daily_sent_e2ee_messages),
    SUM(daily_sent_messages),
    SUM(daily_user_type_bridged),
    SUM(daily_user_type_guest),
    SUM(daily_user_type_native),
    SUM(monthly_active_users),
    SUM(r30_users_all),
    SUM(r30_users_android),
    SUM(r30_users_ios),
    SUM(r30_users_electron),
    SUM(r30_users_web),
    SUM(total_nonbridged_users),
    SUM(total_room_count),
    SUM(total_users),
    COUNT(homeserver)
FROM statsreport WHERE local_timestamp::DATE >= (SELECT COALESCE(MAX(day), 'EPOCH'::DATE) FROM aggregate_stats)
GROUP BY local_timestamp::DATE
ON CONFLICT (day) 
DO UPDATE SET
    daily_active_e2ee_rooms = excluded.daily_active_e2ee_rooms,
    daily_active_rooms = excluded.daily_active_rooms,
    daily_active_users = excluded.daily_active_users,
    daily_e2ee_messages = excluded.daily_e2ee_messages,
    daily_messages = excluded.daily_messages,
    daily_sent_e2ee_messages = excluded.daily_sent_e2ee_messages,
    daily_sent_messages = excluded.daily_sent_messages,
    daily_user_type_bridged = excluded.daily_user_type_bridged,
    daily_user_type_guest = excluded.daily_user_type_guest,
    daily_user_type_native = excluded.daily_user_type_native,
    monthly_active_users = excluded.monthly_active_users,
    r30_users_all = excluded.r30_users_all,
    r30_users_android = excluded.r30_users_android,
    r30_users_ios = excluded.r30_users_ios,
    r30_users_electron = excluded.r30_users_electron,
    r30_users_web = excluded.r30_users_web,
    total_nonbridged_users = excluded.total_nonbridged_users,
    total_room_count = excluded.total_room_count,
    total_users = excluded.total_users,
    daily_active_homeservers = excluded.daily_active_homeservers
        ;"#
    )
    .execute(&*pool)
    .await
    .context("could not aggregate stats")?;

    Ok(())
}

async fn save_report(pool: &sqlx::PgPool, report: &model::StatsReport) -> Result<()> {
    sqlx::query!(
        r#"
INSERT INTO statsreport (
    local_timestamp,
    remote_timestamp,
    daily_active_e2ee_rooms,
    daily_active_rooms,
    daily_active_users,
    daily_e2ee_messages,
    daily_messages,
    daily_sent_e2ee_messages,
    daily_sent_messages,
    daily_user_type_bridged,
    daily_user_type_guest,
    daily_user_type_native,
    cpu_average,
    event_cache_size,
    memory_rss,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    total_nonbridged_users,
    total_room_count,
    total_users,
    uptime_seconds,
    cache_factor,
    database_engine,
    database_server_version,
    homeserver,
    log_level,
    python_version,
    server_context,
    remote_addr,
    x_forwarded_for,
    user_agent) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35 )
        "#, 
        report.local_timestamp,
        report.remote_timestamp,
        report.daily_active_e2ee_rooms,
        report.daily_active_rooms,
        report.daily_active_users,
        report.daily_e2ee_messages,
        report.daily_messages,
        report.daily_sent_e2ee_messages,
        report.daily_sent_messages,
        report.daily_user_type_bridged,
        report.daily_user_type_guest,
        report.daily_user_type_native,
        report.cpu_average,
        report.event_cache_size,
        report.memory_rss,
        report.monthly_active_users,
        report.r30_users_all,
        report.r30_users_android,
        report.r30_users_ios,
        report.r30_users_electron,
        report.r30_users_web,
        report.total_nonbridged_users,
        report.total_room_count,
        report.total_users,
        report.uptime_seconds,
        report.cache_factor,
        report.database_engine,
        report.database_server_version,
        report.homeserver,
        report.log_level,
        report.python_version,
        report.server_context,
        report.remote_addr,
        report.x_forwarded_for,
        report.user_agent)
        .execute(&*pool)
        .await
        .context("failed executing aggregation query.")?;

    Ok(())
}
