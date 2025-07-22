#![allow(clippy::unwrap_used)]

use crate::database;
use crate::model;
use crate::model::AggregatedStatsByContext;
use crate::server;
use crate::settings::DBSettings;
use crate::AggregatedStats;

use std::sync::Arc;
use std::{collections::HashMap, env};

use axum::routing::{get, put};
use axum::Extension;
use axum::Router;
use http::Request;
use http::StatusCode;
use http_body::Body as HttpBody;
use hyper::Body;
use serde_json::json;
use time::Duration;
use tokio::sync::mpsc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn integration_testing() {
    // crate::setup_logging("debug");
    let db_url = env::var("DATABASE_URL").expect("database URL");
    let pool = sqlx::PgPool::connect(&db_url).await.expect("DB connection");
    let (tx, mut rx) = mpsc::channel::<model::Report>(64);

    let app = Router::new()
        .route("/report-usage-stats/push", put(server::tests::save_report))
        .route(
            "/aggregated-stats/:day",
            get(server::tests::get_aggregated_stats),
        )
        .route(
            "/aggregated-stats/:day/:context",
            get(server::tests::get_aggregated_stats_by_context),
        )
        .with_state(Arc::new(DBSettings {
            url: db_url.clone(),
        }))
        .layer(Extension(tx.clone()));

    let mut test_payloads = HashMap::new();
    test_payloads.insert("v0.33.6", include_str!("./report-v0.33.6.json"));
    test_payloads.insert("v0.99.2", include_str!("./report-v0.99.2.json"));
    test_payloads.insert("v0.99.4", include_str!("./report-v0.99.4.json"));
    test_payloads.insert("v1.28.0", include_str!("./report-v1.28.0.json"));
    test_payloads.insert("v1.78.0", include_str!("./report-v1.78.0.json"));
    test_payloads.insert("v1.85.2", include_str!("./report-v1.85.2.json"));
    test_payloads.insert("v1.99.0", include_str!("./report-v1.99.0.json"));

    for (version, payload) in test_payloads {
        let app = app.clone();

        let resp = app
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri("/report-usage-stats/push")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload))
                    .expect("building request"),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "testing synapse {} report-usage-stats request, got response {:?} with body {:?}",
            version,
            resp,
            resp.body(),
        );

        let report = rx.recv().await.expect("receive report");
        let id = database::tests::save_report(&pool, &report)
            .await
            .expect("save report");
        assert_eq!(
            report,
            database::tests::get_report_by_id(&pool, id)
                .await
                .expect("get report by id")
        );
    }
    let today = time::OffsetDateTime::now_utc().date();
    database::tests::aggregate_stats(&pool, today)
        .await
        .expect("aggregate stats");
    database::tests::aggregate_stats_by_context(&pool, today)
        .await
        .expect("aggregate stats by context");

    let date = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .date();

    let date = date
        .format(&time::format_description::well_known::Iso8601::DATE)
        .expect("format date");
    let uri = format!("/aggregated-stats/{date}");

    let aggregated_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .unwrap();

    assert_eq!(
        aggregated_res.status(),
        StatusCode::OK,
        "testing GET '{}', got response {:?} with body {:?}",
        uri,
        aggregated_res,
        aggregated_res.body(),
    );

    let uri = format!("/aggregated-stats/{date}/test_context");
    let aggregated_context_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .unwrap();

    assert_eq!(
        aggregated_context_res.status(),
        StatusCode::OK,
        "testing GET '{}', got response '{:?}' with body '{:?}'",
        uri,
        aggregated_context_res,
        aggregated_context_res.body()
    );

    let body: AggregatedStatsByContext = serde_json::from_slice(
        aggregated_context_res
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes()
            .as_ref(),
    )
    .expect("Converting response body to json");
    assert_eq!(body.server_context, "test_context");
    assert_eq!(body.daily_active_users, Some(9));
    assert_eq!(body.monthly_active_users, Some(14));
    assert_eq!(body.total_messages, Some(4));
    assert_eq!(body.total_e2ee_messages, Some(0));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn load_test() {
    const HOMESERVERS: i64 = 3;
    const DAYS: i64 = 750;
    const DAILY_ACTIVE_USERS: i64 = 200;
    const MONTHLY_ACTIVE_USERS: i64 = 600;
    const DAILY_MESSAGES: i64 = 5;
    const DAILY_E2EE_MESSAGES: i64 = 10;

    let db_url = env::var("DATABASE_URL").expect("database URL");
    let pool = sqlx::PgPool::connect(&db_url).await.expect("DB connection");
    let (tx, mut rx) = mpsc::channel::<model::Report>(1);

    let app = Router::new()
        .route("/report-usage-stats/push", put(server::tests::save_report))
        .route(
            "/aggregated-stats/:day",
            get(server::tests::get_aggregated_stats),
        )
        .route(
            "/aggregated-stats/:day/:context",
            get(server::tests::get_aggregated_stats_by_context),
        )
        .with_state(Arc::new(DBSettings {
            url: db_url.clone(),
        }))
        .layer(Extension(tx.clone()));

    let mut days = Vec::new();
    for day in (0..DAYS).rev() {
        let mut now = time::OffsetDateTime::now_utc();
        now = now.replace_time(time::Time::MIDNIGHT);
        now = now.checked_sub(Duration::days(day)).expect("sub days");
        days.push(now);
    }

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        loop {
            let report = rx.recv().await.expect("receive report");
            database::tests::save_report(&pool_clone, &report)
                .await
                .expect("save report");
        }
    });

    let mut set = JoinSet::new();
    let sem = Arc::new(Semaphore::new(20));

    for homeserver in 0..HOMESERVERS {
        for day in &days {
            let app_clone = app.clone();
            let day = *day;

            let permit = Arc::clone(&sem).acquire_owned().await;
            set.spawn(async move {
                let _permit = permit;
                // println!(
                //     "Sending report for day: {}, homeserver: {}",
                //     day.date(),
                //     homeserver
                // );
                let payload = json!(
                    {
                        "version": "custom",
                        "server_context": "load_test_".to_owned() + &homeserver.to_string(),
                        "homeserver": "load_test_".to_owned() + &homeserver.to_string(),
                        "daily_active_users": DAILY_ACTIVE_USERS,
                        "monthly_active_users": MONTHLY_ACTIVE_USERS,
                        "daily_messages": DAILY_MESSAGES,
                        "daily_e2ee_messages": DAILY_E2EE_MESSAGES,
                        "local_timestamp": day.unix_timestamp(),
                    }
                );

                let resp = app_clone
                    .oneshot(
                        Request::builder()
                            .method(http::Method::PUT)
                            .uri("/report-usage-stats/push")
                            .header(http::header::CONTENT_TYPE, "application/json")
                            .body(Body::from(payload.to_string()))
                            .expect("building request"),
                    )
                    .await
                    .unwrap();

                assert_eq!(
                    resp.status(),
                    StatusCode::OK,
                    "report-usage-stats request, got body {:?}",
                    resp.into_body()
                        .collect()
                        .await
                        .expect("collect body")
                        .to_bytes(),
                );
            });
        }
    }

    set.join_all().await;

    let mut set = JoinSet::new();
    let sem = Arc::new(Semaphore::new(1));
    for day in &days {
        let permit = Arc::clone(&sem).acquire_owned().await;
        let pool = pool.clone();
        let day = *day;
        set.spawn(async move {
            let _permit = permit;
            //println!("Aggregating stats for day: {}", day.date());
            database::tests::aggregate_stats(&pool, day.date())
                .await
                .expect("aggregate stats");
            database::tests::aggregate_stats_by_context(&pool, day.date())
                .await
                .expect("aggregate stats by context");
        });
    }
    set.join_all().await;

    let mut set = JoinSet::new();
    let sem = Arc::new(Semaphore::new(20));

    // Check only every 7th day to avoid too many requests
    for (day_pos, day) in days
        .iter()
        .enumerate()
        .filter(|(_, day)| day.day() % 7 == 0)
    {
        let day_pos: i64 = day_pos.try_into().expect("i64");
        let app_clone = app.clone();
        let day_clone = *day;
        let permit = Arc::clone(&sem).acquire_owned().await;
        set.spawn(async move {
            let _permit = permit;
            let uri = format!("/aggregated-stats/{}", day_clone.date());
            //println!("aggregated-stats request: {uri}");
            let aggregated_res = app_clone
                .oneshot(
                    Request::builder()
                        .method(http::Method::GET)
                        .uri(&uri)
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .unwrap();

            let body: AggregatedStats = serde_json::from_slice(
                aggregated_res
                    .into_body()
                    .collect()
                    .await
                    .expect("collect body")
                    .to_bytes()
                    .as_ref(),
            )
            .expect("Converting response body to json");

            assert_eq!(
                body.daily_active_users.unwrap(),
                DAILY_ACTIVE_USERS * HOMESERVERS
            );
            assert_eq!(
                body.monthly_active_users.unwrap(),
                (MONTHLY_ACTIVE_USERS) * HOMESERVERS
            );
            assert_eq!(body.daily_messages.unwrap(), (DAILY_MESSAGES) * HOMESERVERS);
            assert_eq!(
                body.daily_e2ee_messages.unwrap(),
                (DAILY_E2EE_MESSAGES) * HOMESERVERS
            );
            assert_eq!(
                body.total_messages.unwrap(),
                (DAILY_MESSAGES * HOMESERVERS) * (day_pos + 1)
            );
            assert_eq!(
                body.total_e2ee_messages.unwrap(),
                (DAILY_E2EE_MESSAGES * HOMESERVERS) * (day_pos + 1)
            );
        });
        for homeserver in 0..HOMESERVERS.min(2) {
            // We only test the first two homeserver contexts to avoid too many requests
            let app_clone = app.clone();
            let day_clone = *day;
            let permit = Arc::clone(&sem).acquire_owned().await;
            set.spawn(async move {
                let _permit = permit;
                let uri = format!(
                    "/aggregated-stats/{}/load_test_{homeserver}",
                    day_clone.date()
                );
                //println!("aggregated-stats context request: {uri}");
                let aggregated_context_res = app_clone
                    .oneshot(
                        Request::builder()
                            .method(http::Method::GET)
                            .uri(&uri)
                            .body(Body::empty())
                            .expect("build request"),
                    )
                    .await
                    .unwrap();

                let body: AggregatedStatsByContext = serde_json::from_slice(
                    aggregated_context_res
                        .into_body()
                        .collect()
                        .await
                        .expect("collect body")
                        .to_bytes()
                        .as_ref(),
                )
                .expect("Converting response body to json");
                assert_eq!(
                    body.server_context,
                    "load_test_".to_owned() + &homeserver.to_string()
                );

                assert_eq!(body.daily_active_users.unwrap(), DAILY_ACTIVE_USERS);
                assert_eq!(body.monthly_active_users.unwrap(), MONTHLY_ACTIVE_USERS);
                assert_eq!(body.daily_messages.unwrap(), DAILY_MESSAGES);
                assert_eq!(body.daily_e2ee_messages.unwrap(), DAILY_E2EE_MESSAGES);
                assert_eq!(body.total_messages.unwrap(), DAILY_MESSAGES * (day_pos + 1));
                assert_eq!(
                    body.total_e2ee_messages.unwrap(),
                    (DAILY_E2EE_MESSAGES * (day_pos + 1))
                );
            });
        }
    }

    set.join_all().await;
}

#[tokio::test]
async fn test_healthcheck() {
    let db_url = env::var("DATABASE_URL").expect("database URL");
    let app = || {
        Router::new()
            .route("/health", get(server::tests::health_check))
            .with_state(Arc::new(DBSettings {
                url: db_url.clone(),
            }))
    };
    let resp = app()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "testing GET '/health', got response {:?} with body {:?}",
        resp,
        resp.body(),
    );
}

#[tokio::test]
async fn test_healthcheck_no_db() {
    let db_url = "http://example.invalid".to_owned();
    let app = || {
        Router::new()
            .route("/health", get(server::tests::health_check))
            .with_state(Arc::new(DBSettings {
                url: db_url.clone(),
            }))
    };
    let resp = app()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "testing GET '/health', got response {:?} with body {:?}",
        resp,
        resp.body(),
    );
}
