use std::fmt::Debug;

use anyhow::{Context, Result};
use config::{Config, Environment, File};
use rust_telemetry::config::OtelConfig;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DBSettings {
    pub url: String,
}

impl Debug for DBSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DBSettings")
            .field("url", &&"<redacted>")
            .finish()
    }
}

impl std::fmt::Display for DBSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DBSettings {{ url: <redacted> }}")
    }
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
            .add_source(File::with_name(config).required(false))
            .add_source(
                Environment::with_prefix("FAMEDLY_BDR")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .build()
            .context("can't load config")?
            .try_deserialize()?)
    }
}
