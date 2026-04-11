use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub tls: TlsSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct TlsSettings {
    pub certificate_path: String,
    pub certificate_key_path: String,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from_str(
                include_str!("config/base.toml"),
                FileFormat::Toml,
            ))
            .add_source(Environment::default().separator("__").ignore_empty(true))
            .build()?
            .try_deserialize()
    }
}
