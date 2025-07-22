use anyhow::{Context, Result};
use clap::ArgMatches;
pub use model::{AggregatedStats, AggregatedStatsByContext};
use rust_telemetry::init_otel;
use settings::Settings;
use std::sync::Arc;

mod database;
mod model;
mod server;
mod settings;
#[cfg(test)]
mod tests;

pub async fn run(opts: ArgMatches) -> Result<()> {
    let settings = Settings::load(opts.get_one::<String>("config").expect("Config string"))
        .context("can't load config.")?;
    let _guard = init_otel!(&settings.telemetry.unwrap_or_default()).unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel::<model::Report>(64);

    let server = {
        let db_settings = Arc::new(settings.database.clone());
        let settings = settings.server;
        tokio::spawn(async move {
            let tx = tx.clone();
            server::run_server(settings, db_settings, tx)
                .await
                .expect("Running server");
        })
    };

    {
        let settings = settings.database.clone();
        tokio::spawn(async move {
            database::aggregate_loop(&settings).await;
        });
    }

    {
        let settings = settings.database;
        tokio::spawn(async move {
            database::insert_reports_loop(&settings, rx).await;
        });
    }

    server.await?;

    Ok(())
}
