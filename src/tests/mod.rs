use crate::model;
use crate::server;
use crate::sql;

use std::{collections::HashMap, env};

use axum::handler::put;
use axum::AddExtensionLayer;
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
    let (tx, mut rx) = mpsc::channel::<model::StatsReport>(64);

    let app = || {
        Router::new()
            .route("/report-usage-stats/push", put(server::tests::save_report))
            .layer(AddExtensionLayer::new(tx.clone()))
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
        let id = sql::tests::save_report(&pool, &report).await.unwrap();
        assert_eq!(
            report,
            sql::tests::get_report_by_id(&pool, id).await.unwrap()
        );
    }
    sql::tests::aggregate_stats(&pool).await.unwrap();
}
