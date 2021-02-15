use anyhow::{Context, Result};
use log::info;
use std::{process, str::FromStr};

mod model;
mod server;
mod sql;

fn setup_logging(level: log::LevelFilter) -> Result<()> {
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
    info!("logging set up properly");
    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    let log_level = match std::env::var("LOG_LEVEL") {
        Ok(val) => log::LevelFilter::from_str(&val).ok(),
        Err(_) => Some(log::LevelFilter::Warn),
    };

    match log_level {
        Some(level) => setup_logging(level)?,
        None => {
            setup_logging(log::LevelFilter::Warn)?;
            log::warn!("invalid environment variable LOG_LEVEL, falling back to LOG_LEVEL=warn.");
        }
    }

    let db_url =
        match dotenv::var("DATABASE_URL").context("failed connecting to PostgreSQL server.") {
            Ok(db_url) => db_url,
            Err(err) => {
                log::error!("{:?}", err);
                process::exit(-1);
            }
        };

    let pool = sql::connect_db(&db_url).await;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<model::StatsReport>(64);

    let server = actix_rt::spawn(async move {
        let tx = tx.clone();
        server::run_server(tx).await.unwrap();
    });

    {
        let pool = pool.clone();
        actix_rt::spawn(async move {
            sql::insert_reports_loop(pool, &mut rx).await;
        });
    }

    actix_rt::spawn(async move {
        sql::aggregate_loop(pool).await;
    });

    server.await?;

    Ok(())
}
