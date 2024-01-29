use crate::database;
use crate::model;
use crate::server;

use std::{collections::HashMap, env};

use axum::routing::{get, put};
use axum::Extension;
use axum::Router;
use http::Request;
use http::StatusCode;
use hyper::Body;
use tokio::sync::mpsc;

use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn integration_testing() {
    // crate::setup_logging("debug");
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
    let (tx, mut rx) = mpsc::channel::<model::Report>(64);

    let app = || {
        Router::new()
            .route("/report-usage-stats/push", put(server::tests::save_report))
            .layer(Extension(tx.clone()))
    };

    let mut test_payloads = HashMap::new();
    test_payloads.insert("v0.33.6", include_str!("./report-v0.33.6.json"));
    test_payloads.insert("v0.99.2", include_str!("./report-v0.99.2.json"));
    test_payloads.insert("v0.99.4", include_str!("./report-v0.99.4.json"));
    test_payloads.insert("v1.28.0", include_str!("./report-v1.28.0.json"));

    for (version, payload) in test_payloads {
        let app = app();

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
