use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{App, Arg};
use settings::Settings;

mod model;
mod server;
mod settings;
mod sql;
#[cfg(test)]
mod tests;

fn setup_logging(level: &str) -> Result<()> {
    let level = log::LevelFilter::from_str(&level).unwrap();

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
}

#[actix_web::main]
async fn main() -> Result<()> {
    let opts = App::new("barad-dur")
        .version("0.1")
        .args(&[
            Arg::with_name("config")
                .help("path of config file")
                .takes_value(true)
                .short("c")
                .long("config")
                .default_value("./config.yaml"),
            Arg::with_name("log_level")
                .help("log level")
                .possible_values(&["Error", "Warn", "Info", "Debug", "Trace"])
                .takes_value(true)
                .long("log")
                .default_value("Warn"),
        ])
        .get_matches();

    setup_logging(opts.value_of("log_level").unwrap())?;

    let settings =
        Settings::load(opts.value_of("config").unwrap()).context("can't load config.")?;

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
