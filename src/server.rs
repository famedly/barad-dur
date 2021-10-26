use std::net::SocketAddr;
use std::process;

use anyhow::{Context, Result};
use axum::extract::{FromRequest, RequestParts};
use axum::handler::put;
use axum::{async_trait, extract, AddExtensionLayer, Router, Server};
use chrono::SubsecRound;
use http::{HeaderMap, StatusCode};
use tokio::sync::mpsc;

use crate::model;
use crate::settings::ServerSettings;
pub async fn run_server(
    settings: ServerSettings,
    tx: mpsc::Sender<model::StatsReport>,
) -> Result<()> {
    Server::bind(&settings.host.parse::<SocketAddr>()?)
        .serve(
            Router::new()
                .route("/report-usage-stats/push", put(save_report))
                .layer(AddExtensionLayer::new(tx))
                .into_make_service_with_connect_info::<SocketAddr, _>(),
        )
        .await?;

    Ok(())
}

pub struct ExtractHeaderMap(Option<HeaderMap>);

#[async_trait]
impl<B> FromRequest<B> for ExtractHeaderMap
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(Self(req.take_headers()))
    }
}

async fn save_report(
    tx: extract::Extension<mpsc::Sender<model::StatsReport>>,
    report: extract::Json<model::StatsReport>,
    addr: Option<extract::ConnectInfo<SocketAddr>>,
    ExtractHeaderMap(headers): ExtractHeaderMap,
) -> StatusCode {
    let mut report = report;

    report.local_timestamp = Some(chrono::Utc::now().round_subsecs(6));

    report.remote_addr = addr.map(|addr| addr.0.to_string());

    report.x_forwarded_for = headers.as_ref().and_then(|headers| {
        headers
            .get("X-Forwarded-For")
            .map(|addr| addr.to_str().ok())
            .flatten()
            .map(String::from)
    });

    report.user_agent = headers.as_ref().and_then(|headers| {
        headers
            .get("User-Agent")
            .map(|value| value.to_str().ok())
            .flatten()
            .map(String::from)
    });

    if let Err(err) = tx
        .send(report.0)
        .await
        .context("can't send report to sql thread.")
    {
        log::error!("{:?}", err);
        process::exit(-1);
    }
    StatusCode::OK
}

#[cfg(test)]
pub mod tests {
    use std::net::SocketAddr;

    use axum::extract;
    use http::StatusCode;
    use tokio::sync::mpsc;

    use crate::model;

    use super::ExtractHeaderMap;

    pub async fn save_report(
        tx: extract::Extension<mpsc::Sender<model::StatsReport>>,
        report: extract::Json<model::StatsReport>,
        addr: Option<extract::ConnectInfo<SocketAddr>>,
        ExtractHeaderMap(headers): ExtractHeaderMap,
    ) -> StatusCode {
        super::save_report(tx, report, addr, ExtractHeaderMap(headers)).await
    }
}
