use std::process;

use anyhow::{Context, Result};
use sqlx::PgPool;
use tokio::{sync::mpsc::Receiver, time::interval};

use crate::model::Report;
use crate::settings::DBSettings;

pub async fn aggregate_loop(settings: &DBSettings) {
    let pool = get_db_pool(settings).await;

    let interval = &mut interval(std::time::Duration::new(86400, 0));
    loop {
        if let Err(err) = aggregate_stats(&pool).await {
            log::error!("{:?}", err);
            process::exit(-1);
        }
        interval.tick().await;
    }
}

pub async fn insert_reports_loop(settings: &DBSettings, mut rx: Receiver<Report>) {
    let pool = get_db_pool(settings).await;

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

async fn connect_pg(url: &str) -> PgPool {
    let pool = match sqlx::PgPool::connect(url)
        .await
        .context("failed connecting to PostgreSQL server.")
    {
        Ok(pool) => pool,
        Err(err) => {
            log::error!("{:?}", err);
            process::exit(-1);
        }
    };

    if let Err(err) = sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run migrations")
    {
        log::error!("{:?}", err);
        process::exit(-1);
    }

    pool
}

async fn get_db_pool(DBSettings { url }: &DBSettings) -> PgPool {
    use once_cell::sync::OnceCell;
    static PG_POOL_CELL: OnceCell<PgPool> = OnceCell::new();

    PG_POOL_CELL.get().map(PgPool::clone).unwrap_or({
        let pool = connect_pg(url).await;
        let _ = PG_POOL_CELL.set(pool.clone());
        pool
    })
}

async fn aggregate_stats(pool: &sqlx::PgPool) -> Result<()> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO
          aggregated_stats (
            day,
            total_users,
            total_nonbridged_users,
            total_room_count,
            daily_active_users,
            daily_active_rooms,
            daily_messages,
            daily_sent_messages,
            daily_active_e2ee_rooms,
            daily_e2ee_messages,
            daily_sent_e2ee_messages,
            monthly_active_users,
            r30_users_all,
            r30_users_android,
            r30_users_ios,
            r30_users_electron,
            r30_users_web,
            r30v2_users_all,
            r30v2_users_android,
            r30v2_users_ios,
            r30v2_users_electron,
            r30v2_users_web,
            daily_user_type_native,
            daily_user_type_bridged,
            daily_user_type_guest,
            daily_active_homeservers
          )
        SELECT
          local_timestamp:: DATE,
          SUM(total_users),
          SUM(total_nonbridged_users),
          SUM(total_room_count),
          SUM(daily_active_users),
          SUM(daily_active_rooms),
          SUM(daily_messages),
          SUM(daily_sent_messages),
          SUM(daily_active_e2ee_rooms),
          SUM(daily_e2ee_messages),
          SUM(daily_sent_e2ee_messages),
          SUM(monthly_active_users),
          SUM(r30_users_all),
          SUM(r30_users_android),
          SUM(r30_users_ios),
          SUM(r30_users_electron),
          SUM(r30_users_web),
          SUM(r30v2_users_all),
          SUM(r30v2_users_android),
          SUM(r30v2_users_ios),
          SUM(r30v2_users_electron),
          SUM(r30v2_users_web),
          SUM(daily_user_type_native),
          SUM(daily_user_type_bridged),
          SUM(daily_user_type_guest),
          COUNT(homeserver)
        FROM
          (
            SELECT
              DISTINCT ON (homeserver, local_timestamp:: DATE) *
            FROM
              reports
            WHERE
              local_timestamp:: DATE >= (
                SELECT
                  COALESCE(MAX(day), 'EPOCH':: DATE)
                FROM
                  aggregated_stats
              )
            ORDER BY
              homeserver,
              local_timestamp:: DATE,
              local_timestamp DESC
          ) as pg_sucks
        GROUP BY
          local_timestamp:: DATE ON CONFLICT (day) DO
        UPDATE
        SET
          total_users = excluded.total_users,
          total_nonbridged_users = excluded.total_nonbridged_users,
          total_room_count = excluded.total_room_count,
          daily_active_users = excluded.daily_active_users,
          daily_active_rooms = excluded.daily_active_rooms,
          daily_messages = excluded.daily_messages,
          daily_sent_messages = excluded.daily_sent_messages,
          daily_active_e2ee_rooms = excluded.daily_active_e2ee_rooms,
          daily_e2ee_messages = excluded.daily_e2ee_messages,
          daily_sent_e2ee_messages = excluded.daily_sent_e2ee_messages,
          monthly_active_users = excluded.monthly_active_users,
          r30_users_all = excluded.r30_users_all,
          r30_users_android = excluded.r30_users_android,
          r30_users_ios = excluded.r30_users_ios,
          r30_users_electron = excluded.r30_users_electron,
          r30_users_web = excluded.r30_users_web,
          r30v2_users_all = excluded.r30_users_all,
          r30v2_users_android = excluded.r30_users_android,
          r30v2_users_ios = excluded.r30_users_ios,
          r30v2_users_electron = excluded.r30_users_electron,
          r30v2_users_web = excluded.r30_users_web,
          daily_user_type_native = excluded.daily_user_type_native,
          daily_user_type_bridged = excluded.daily_user_type_bridged,
          daily_user_type_guest = excluded.daily_user_type_guest,
          daily_active_homeservers = excluded.daily_active_homeservers;"#
    )
    .execute(pool)
    .await
    .context("could not aggregate stats")?;

    Ok(())
}

async fn save_report(pool: &sqlx::PgPool, report: &Report) -> Result<i64> {
    #[derive(sqlx::FromRow)]
    struct Id {
        id: i64,
    }

    let id: Id = sqlx::query_as!(
        Id,
        r#"
        INSERT INTO
          reports (
            homeserver,
            local_timestamp,
            remote_timestamp,
            remote_addr,
            forwarded_for,
            uptime_seconds,
            total_users,
            total_nonbridged_users,
            total_room_count,
            daily_active_users,
            daily_active_rooms,
            daily_messages,
            daily_sent_messages,
            daily_active_e2ee_rooms,
            daily_e2ee_messages,
            daily_sent_e2ee_messages,
            monthly_active_users,
            r30_users_all,
            r30_users_android,
            r30_users_ios,
            r30_users_electron,
            r30_users_web,
            r30v2_users_all,
            r30v2_users_android,
            r30v2_users_ios,
            r30v2_users_electron,
            r30v2_users_web,
            cpu_average,
            memory_rss,
            cache_factor,
            event_cache_size,
            user_agent,
            daily_user_type_native,
            daily_user_type_bridged,
            daily_user_type_guest,
            python_version,
            database_engine,
            database_server_version,
            server_context,
            log_level
          )
        VALUES
          (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11,
            $12,
            $13,
            $14,
            $15,
            $16,
            $17,
            $18,
            $19,
            $20,
            $21,
            $22,
            $23,
            $24,
            $25,
            $26,
            $27,
            $28,
            $29,
            $30,
            $31,
            $32,
            $33,
            $34,
            $35,
            $36,
            $37,
            $38,
            $39,
            $40
          ) RETURNING id;"#,
        report.homeserver,
        report.local_timestamp,
        report.remote_timestamp,
        report.remote_addr,
        report.forwarded_for,
        report.uptime_seconds,
        report.total_users,
        report.total_nonbridged_users,
        report.total_room_count,
        report.daily_active_users,
        report.daily_active_rooms,
        report.daily_messages,
        report.daily_sent_messages,
        report.daily_active_e2ee_rooms,
        report.daily_e2ee_messages,
        report.daily_sent_e2ee_messages,
        report.monthly_active_users,
        report.r30_users_all,
        report.r30_users_android,
        report.r30_users_ios,
        report.r30_users_electron,
        report.r30_users_web,
        report.r30v2_users_all,
        report.r30v2_users_android,
        report.r30v2_users_ios,
        report.r30v2_users_electron,
        report.r30v2_users_web,
        report.cpu_average,
        report.memory_rss,
        report.cache_factor,
        report.event_cache_size,
        report.user_agent,
        report.daily_user_type_native,
        report.daily_user_type_bridged,
        report.daily_user_type_guest,
        report.python_version,
        report.database_engine,
        report.database_server_version,
        report.server_context,
        report.log_level,
    )
    .fetch_one(pool)
    .await
    .context("failed executing aggregation query.")?;

    Ok(id.id)
}

#[cfg(test)]
pub mod tests {
    use crate::model::Report;
    use anyhow::Result;

    pub async fn aggregate_stats(pool: &sqlx::PgPool) -> Result<()> {
        super::aggregate_stats(pool).await
    }

    pub async fn save_report(pool: &sqlx::PgPool, report: &Report) -> Result<i64> {
        super::save_report(pool, report).await
    }

    pub async fn get_report_by_id(pool: &sqlx::PgPool, id: i64) -> Result<Report> {
        let report = sqlx::query_as!(
            Report,
            r#"
            SELECT
              homeserver,
              local_timestamp,
              remote_timestamp,
              remote_addr,
              forwarded_for,
              uptime_seconds,
              total_users,
              total_nonbridged_users,
              total_room_count,
              daily_active_users,
              daily_active_rooms,
              daily_messages,
              daily_sent_messages,
              daily_active_e2ee_rooms,
              daily_e2ee_messages,
              daily_sent_e2ee_messages,
              monthly_active_users,
              r30_users_all,
              r30_users_android,
              r30_users_ios,
              r30_users_electron,
              r30_users_web,
              r30v2_users_all,
              r30v2_users_android,
              r30v2_users_ios,
              r30v2_users_electron,
              r30v2_users_web,
              cpu_average,
              memory_rss,
              cache_factor,
              event_cache_size,
              user_agent,
              daily_user_type_native,
              daily_user_type_bridged,
              daily_user_type_guest,
              python_version,
              database_engine,
              database_server_version,
              server_context,
              log_level
            FROM
              reports
            WHERE
              id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(report)
    }
}
