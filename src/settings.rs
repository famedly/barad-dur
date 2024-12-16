use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DBSettings {
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerSettings {
    pub host: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DBSettings,
}

impl Settings {
    pub fn load(config: &str) -> Result<Self> {
        Ok(Config::builder()
            .set_default("server.host", "[::]:8080")?
            .set_default("log.level", "warn")?
            .add_source(
                Environment::with_prefix("BDR")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .add_source(File::with_name(config).required(false))
            .build()
            .context("can't load config")?
            .try_deserialize()?)
    }
}
