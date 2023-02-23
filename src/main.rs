use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Arg, Command};
use settings::Settings;
use time::macros::format_description;

mod database;
mod model;
mod server;
mod settings;
#[cfg(test)]
mod tests;

fn setup_logging(level: &str) -> Result<()> {
    let level = log::LevelFilter::from_str(level).unwrap();

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                    .format(format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    ))
                    .unwrap(),
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Command::new("barad-dur")
        .version("0.1")
        .args(&[
            Arg::new("config")
                .help("path of config file")
                .short('c')
                .long("config")
                .default_value("./config.yaml"),
            Arg::new("log_level")
                .help("log level")
                .value_parser(["Error", "Warn", "Info", "Debug", "Trace"])
                .long("log")
                .default_value("Warn"),
        ])
        .get_matches();

    setup_logging(opts.get_one::<String>("log_level").unwrap())?;

    let settings =
        Settings::load(opts.get_one::<String>("config").unwrap()).context("can't load config.")?;

    let (tx, rx) = tokio::sync::mpsc::channel::<model::Report>(64);

    let server = {
        let settings = settings.server;
        tokio::spawn(async move {
            let tx = tx.clone();
            server::run_server(settings, tx).await.unwrap();
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
