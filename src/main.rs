use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Arg, Command};
use time::macros::format_description;

fn setup_logging(level: &str) -> Result<()> {
    let level = log::LevelFilter::from_str(level).expect("Log level parsing");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                    .format(format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    ))
                    .expect("Formatting"),
                record.level(),
                record.target(),
                message
            ));
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

    setup_logging(opts.get_one::<String>("log_level").expect("Log level"))?;

    barad_dur::run(opts).await?;
    Ok(())
}
