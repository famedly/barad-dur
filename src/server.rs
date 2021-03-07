use std::process;

use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::{Context, Result};

use crate::model;
use crate::settings::ServerSettings;

pub async fn run_server(
    settings: ServerSettings,
    tx: tokio::sync::mpsc::Sender<model::StatsReport>,
) -> Result<()> {
    let server = match HttpServer::new(move || {
        let tx = tx.clone();

        App::new().route(
            "/report-usage-stats/push",
            web::put().to(
                move |req: web::HttpRequest, stats: web::Json<model::StatsReport>| {
                    let tx = tx.clone();
                    async move {
                        let req = req.clone();
                        let mut stats = stats;

                        stats.local_timestamp = Some(chrono::Utc::now());

                        stats.remote_addr = req.peer_addr().map(|addr| addr.to_string());

                        stats.x_forwarded_for = req
                            .headers()
                            .get("X-Forwarded-For")
                            .map(|addr| addr.to_str().ok())
                            .flatten()
                            .map(String::from);

                        stats.user_agent = req
                            .headers()
                            .get("User-Agent")
                            .map(|value| value.to_str().ok())
                            .flatten()
                            .map(String::from);

                        if let Err(err) = tx
                            .send(stats.into_inner())
                            .await
                            .context("can't send report to sql thread.")
                        {
                            log::error!("{:?}", err);
                            process::exit(-1);
                        }
                        HttpResponse::Ok()
                    }
                },
            ),
        )
    })
    .bind(settings.host)
    .context("failed to start server.")
    {
        Ok(server) => server,
        Err(err) => {
            log::error!("{:?}", err);
            process::exit(-1);
        }
    };

    if let Err(err) = server.run().await.context("server crashed") {
        log::error!("{:?}", err);
        process::exit(-1);
    };

    Ok(())
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model;
    use actix_web::http::{header, Method, StatusCode};
    use actix_web::{test, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer};
    #[actix_rt::test]
    async fn test_requests() {
        let mut app = test::init_service(App::new().route(
            "/report-usage-stats/push",
            web::put().to(
                move |req: web::HttpRequest, stats: web::Json<model::StatsReport>| async {
                    HttpResponse::Ok().await
                },
            ),
        ))
        .await;

        let mut test_payloads = HashMap::new();
        test_payloads.insert("v0.33.6", include_str!("../tests/report-v0.33.6.json"));
        test_payloads.insert("v0.99.2", include_str!("../tests/report-v0.99.2.json"));
        test_payloads.insert("v0.99.4", include_str!("../tests/report-v0.99.4.json"));
        test_payloads.insert("v1.28.0", include_str!("../tests/report-v1.28.0.json"));

        for (version, payload) in test_payloads {
            let req = test::TestRequest::put()
            .uri("/report-usage-stats/push")
            .insert_header(header::ContentType::json())
            .set_payload(payload)
            .to_request();

            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), StatusCode::OK, "testing panopticon {} report-usage-stats request", version);
        }
    }
}
