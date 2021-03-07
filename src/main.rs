use std::str::FromStr;

use anyhow::{Context, Result};
use settings::Settings;

mod model;
mod server;
mod settings;
mod sql;

fn setup_logging() -> Result<()> {
    let inner = |level| -> Result<()> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}][{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(level)
            //This line is to avoid being flooded with event loop messages
            //(one per thread and second, so 12Hz for a hyperthreaded hexacore)
            //while running with LOG_LEVEL=debug
            .level_for("tokio_reactor", log::LevelFilter::Error)
            .level_for("tokio_core", log::LevelFilter::Error)
            .chain(std::io::stdout())
            .apply()
            .context("error setting up logging")?;
        log::info!("logging set up properly");
        Ok(())
    };

    let log_level = match std::env::var("LOG_LEVEL") {
        Ok(val) => log::LevelFilter::from_str(&val).ok(),
        Err(_) => Some(log::LevelFilter::Warn),
    };

    match log_level {
        Some(level) => inner(level)?,
        None => {
            inner(log::LevelFilter::Warn)?;
            log::warn!("invalid environment variable LOG_LEVEL, falling back to LOG_LEVEL=warn.");
        }
    };

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    setup_logging()?;
    let settings = Settings::load().context("can't load config.")?;

    let (tx, rx) = tokio::sync::mpsc::channel::<model::StatsReport>(64);

    let server = {
        let settings = settings.server;
        actix_rt::spawn(async move {
            let tx = tx.clone();
            server::run_server(settings, tx).await.unwrap();
        })
    };

    {
        let settings = settings.database.clone();
        actix_rt::spawn(async move {
            sql::aggregate_loop(&settings).await;
        });
    }

    {
        let settings = settings.database;
        actix_rt::spawn(async move {
            sql::insert_reports_loop(&settings, rx).await;
        });
    }

    server.await?;

    Ok(())
}
