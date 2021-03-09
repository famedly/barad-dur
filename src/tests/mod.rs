use crate::model;
use crate::server;
use crate::sql;

use std::{collections::HashMap, env};

use actix_web::http::{header, StatusCode};
use actix_web::{test, App};
use tokio::sync::mpsc;

#[actix_rt::test]
async fn integration_testing() {
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
    let (tx, mut rx) = mpsc::channel::<model::StatsReport>(64);

    let mut app =
        test::init_service(App::new().route("/report-usage-stats/push", server::tests::route(&tx)))
            .await;

    let mut test_payloads = HashMap::new();
    test_payloads.insert("v0.33.6", include_str!("./report-v0.33.6.json"));
    test_payloads.insert("v0.99.2", include_str!("./report-v0.99.2.json"));
    test_payloads.insert("v0.99.4", include_str!("./report-v0.99.4.json"));
    test_payloads.insert("v1.28.0", include_str!("./report-v1.28.0.json"));

    for (version, payload) in test_payloads {
        let req = test::TestRequest::put()
            .uri("/report-usage-stats/push")
            .insert_header(header::ContentType::json())
            .set_payload(payload)
            .to_request();

        assert_eq!(
            test::call_service(&mut app, req).await.status(),
            StatusCode::OK,
            "testing panopticon {} report-usage-stats request",
            version
        );

        let report = rx.recv().await.unwrap();
        let id = sql::tests::save_report(&pool, &report).await.unwrap();
        assert_eq!(
            report,
            sql::tests::get_report_by_id(&pool, id).await.unwrap()
        );
    }
}
