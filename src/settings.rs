use anyhow::{Context, Result};
use config::{Config, Environment, File};
use rust_telemetry::config::OtelConfig;
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
    pub telemetry: Option<OtelConfig>,
}

impl Settings {
    pub fn load(config: &str) -> Result<Self> {
        Ok(Config::builder()
            .set_default("server.host", "[::]:8080")?
            .set_default("log.level", "info")?
            .add_source(
                Environment::with_prefix("FAMEDLY_BDR")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .add_source(File::with_name(config).required(false))
            .build()
            .context("can't load config")?
            .try_deserialize()?)
    }
}
