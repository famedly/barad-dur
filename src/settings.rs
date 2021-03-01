use anyhow::{Context, Result};
use config::{Config, File};
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
    pub fn load() -> Result<Self> {
        let mut conf = Config::new();

        conf.set_default("server.host", "[::]:8080")?;
        conf.set_default("log.level", "warn")?;
        conf.merge(File::with_name("config.yaml"))?;
        conf.try_into().context("can't load config")
    }
}
