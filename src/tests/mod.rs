use crate::database;
use crate::model;
use crate::model::AggregatedStatsByContext;
use crate::server;
use crate::settings::DBSettings;

use std::sync::Arc;
use std::{collections::HashMap, env};

use axum::routing::{get, put};
use axum::Extension;
use axum::Router;
use http::Request;
use http::StatusCode;
use http_body::Body as HttpBody;
use hyper::Body;
use tokio::sync::mpsc;

use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn integration_testing() {
    // crate::setup_logging("debug");
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
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
                    .unwrap(),
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

        let report = rx.recv().await.unwrap();
        let id = database::tests::save_report(&pool, &report).await.unwrap();
        assert_eq!(
            report,
            database::tests::get_report_by_id(&pool, id).await.unwrap()
        );
    }
    database::tests::aggregate_stats(&pool).await.unwrap();
    database::tests::aggregate_stats_by_context(&pool)
        .await
        .unwrap();

    let date = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .date();

    let date = date
        .format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap();
    let uri = format!("/aggregated-stats/{}", date);

    let aggregated_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .unwrap(),
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

    let uri = format!("/aggregated-stats/{}/test_context", date);
    let aggregated_context_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(&uri)
                .body(Body::empty())
                .unwrap(),
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
            .unwrap()
            .to_bytes()
            .as_ref(),
    )
    .expect("Converting response body to json");
    assert_eq!(body.server_context, "test_context");
    assert_eq!(body.daily_active_users, Some(9));
    assert_eq!(body.monthly_active_users, Some(14));
}

#[tokio::test]
async fn test_healthcheck() {
    let app = || Router::new().route("/health", get(server::tests::health_check));
    let resp = app()
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
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
