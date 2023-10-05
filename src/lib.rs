use anyhow::{Context, Result};
use clap::ArgMatches;
use settings::Settings;

mod database;
mod model;
mod server;
mod settings;
#[cfg(test)]
mod tests;

pub async fn run(opts: ArgMatches) -> Result<()> {
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
