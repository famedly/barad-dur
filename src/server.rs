use std::net::{IpAddr, SocketAddr};
use std::process;

use anyhow::{Context, Result};
use axum::headers::{Header, HeaderName, UserAgent};
use axum::routing::put;
use axum::{extract, Extension, Router, Server, TypedHeader};
use http::{HeaderValue, StatusCode};
use tokio::sync::mpsc;

use crate::model;
use crate::settings::ServerSettings;
pub async fn run_server(settings: ServerSettings, tx: mpsc::Sender<model::Report>) -> Result<()> {
    Server::bind(&settings.host.parse::<SocketAddr>()?)
        .serve(
            Router::new()
                .route("/report-usage-stats/push", put(save_report))
                .layer(Extension(tx))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

    Ok(())
}

pub struct XForwardedFor(IpAddr);

impl Header for XForwardedFor {
    fn name() -> &'static axum::headers::HeaderName {
        static NAME: HeaderName = HeaderName::from_static("x-forwarded-for");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;
        Ok(Self(
            value
                .to_str()
                .map_err(|_| axum::headers::Error::invalid())?
                .parse()
                .map_err(|_| axum::headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0.to_string())
            .expect("IP addresses are always safe header values");
        values.extend(std::iter::once(value))
    }
}

async fn save_report(
    tx: extract::Extension<mpsc::Sender<model::Report>>,
    addr: Option<extract::ConnectInfo<SocketAddr>>,
    forwarded_addr: Option<TypedHeader<XForwardedFor>>,
    user_agent: Option<TypedHeader<UserAgent>>,
    report: extract::Json<model::Report>,
) -> StatusCode {
    let mut report = report;

    report.local_timestamp = Some({
        let ts = time::OffsetDateTime::now_utc();
        // Dropping some precision here, because postgres doesn't store it anyway, which causes
        // tests to fail because the value coming out was less precise than the value going in
        ts.replace_millisecond((ts.microsecond() / 1000).try_into().unwrap())
            .unwrap()
    });
    report.remote_addr = addr.map(|addr| addr.0.to_string());
    report.forwarded_for =
        forwarded_addr.map(|TypedHeader(forwarded_addr)| forwarded_addr.0.to_string());
    report.user_agent = user_agent.map(|TypedHeader(user_agent)| user_agent.to_string());

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

    use axum::{extract, headers::UserAgent, TypedHeader};
    use http::StatusCode;
    use tokio::sync::mpsc;

    use crate::model;

    use super::XForwardedFor;

    pub async fn save_report(
        tx: extract::Extension<mpsc::Sender<model::Report>>,
        addr: Option<extract::ConnectInfo<SocketAddr>>,
        forwarded_addr: Option<TypedHeader<XForwardedFor>>,
        user_agent: Option<TypedHeader<UserAgent>>,
        report: extract::Json<model::Report>,
    ) -> StatusCode {
        super::save_report(tx, addr, forwarded_addr, user_agent, report).await
    }
}
