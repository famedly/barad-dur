use std::net::{IpAddr, SocketAddr};
use std::process;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::headers::{Header, HeaderName, UserAgent};
use axum::{extract, response::IntoResponse, Extension, Json, Router, Server, TypedHeader};
use axum::{routing::get, routing::put};
use http::{HeaderValue, StatusCode};
use tokio::sync::mpsc;

use crate::model;
use crate::settings::{DBSettings, ServerSettings};

pub async fn run_server(
    settings: ServerSettings,
    db_settings: Arc<DBSettings>,
    tx: mpsc::Sender<model::Report>,
) -> Result<()> {
    Server::bind(&settings.host.parse::<SocketAddr>()?)
        .serve(
            Router::new()
                .route("/health", get(health_check))
                .route("/report-usage-stats/push", put(save_report))
                .route("/aggregated-stats/:day", get(get_aggregated_stats))
                .route(
                    "/aggregated-stats/:day/:context",
                    get(get_aggregated_stats_by_context),
                )
                .with_state(db_settings)
                .layer(Extension(tx))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

    Ok(())
}

/// Returns 200 OK for health checking
async fn health_check(State(db_settings): State<Arc<DBSettings>>) -> impl IntoResponse {
    if let Err(e) = crate::database::connect_pg_gracefully(&db_settings.url).await {
        log::error!("Database connection failed during health check: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}

/// X-Forwarded-For header
pub struct XForwardedFor(IpAddr);

impl Header for XForwardedFor {
    fn name() -> &'static HeaderName {
        /// X-Forwarded-For
        static NAME: HeaderName = HeaderName::from_static("x-forwarded-for");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
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

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        #[allow(clippy::expect_used)]
        let value = HeaderValue::from_str(&self.0.to_string())
            .expect("IP addresses are always safe header values");
        values.extend(std::iter::once(value));
    }
}

async fn get_aggregated_stats(
    State(db_settings): State<Arc<DBSettings>>,
    Path(day): Path<sqlx::types::time::Date>,
) -> Result<Json<model::AggregatedStats>, StatusCode> {
    Ok(Json(
        crate::database::get_aggregated_stats(&db_settings, day)
            .await
            .map_err(|err| {
                log::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?,
    ))
}

async fn get_aggregated_stats_by_context(
    State(db_settings): State<Arc<DBSettings>>,
    Path((day, context)): Path<(sqlx::types::time::Date, String)>,
) -> Result<Json<model::AggregatedStatsByContext>, StatusCode> {
    Ok(Json(
        crate::database::get_aggregated_stats_by_context(&db_settings, day, context)
            .await
            .map_err(|err| {
                log::error!("{:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?,
    ))
}

async fn save_report(
    tx: Extension<mpsc::Sender<model::Report>>,
    addr: Option<extract::ConnectInfo<SocketAddr>>,
    forwarded_addr: Option<TypedHeader<XForwardedFor>>,
    user_agent: Option<TypedHeader<UserAgent>>,
    report: Json<model::Report>,
) -> StatusCode {
    let mut report = report;

    report.local_timestamp = Some({
        let ts = time::OffsetDateTime::now_utc();
        // Dropping some precision here, because postgres doesn't store it anyway, which causes
        // tests to fail because the value coming out was less precise than the value going in
        ts.replace_millisecond((ts.microsecond() / 1000).try_into().expect("ms conversion"))
            .expect("replace millisecond")
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
    use std::sync::Arc;

    use axum::extract::{Path, State};
    use axum::{extract, headers::UserAgent, response::IntoResponse, Json, TypedHeader};
    use http::StatusCode;
    use tokio::sync::mpsc;

    use crate::model;
    use crate::settings::DBSettings;

    use super::XForwardedFor;

    pub async fn save_report(
        tx: extract::Extension<mpsc::Sender<model::Report>>,
        addr: Option<extract::ConnectInfo<SocketAddr>>,
        forwarded_addr: Option<TypedHeader<XForwardedFor>>,
        user_agent: Option<TypedHeader<UserAgent>>,
        report: Json<model::Report>,
    ) -> StatusCode {
        super::save_report(tx, addr, forwarded_addr, user_agent, report).await
    }

    pub async fn health_check(db_settings: State<Arc<DBSettings>>) -> impl IntoResponse {
        super::health_check(db_settings).await
    }

    pub async fn get_aggregated_stats(
        db_settings: State<Arc<DBSettings>>,
        day: Path<sqlx::types::time::Date>,
    ) -> Result<Json<model::AggregatedStats>, StatusCode> {
        super::get_aggregated_stats(db_settings, day).await
    }

    pub async fn get_aggregated_stats_by_context(
        db_settings: State<Arc<DBSettings>>,
        extractors: Path<(sqlx::types::time::Date, String)>,
    ) -> Result<Json<model::AggregatedStatsByContext>, StatusCode> {
        super::get_aggregated_stats_by_context(db_settings, extractors).await
    }
}
