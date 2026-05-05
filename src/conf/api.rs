use crate::conf::ConfigType;
use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_API_CONFIG: &str = include_str!("./default_api_config.toml");

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ApiConfig {
    pub general: General,
    pub development: Development,
    pub server: Server,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct General {}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Development {
    /// Tells the logging package to use human readable output.
    pub pretty_logging: bool,

    /// Turns off authentication.
    pub bypass_auth: bool,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Server {
    /// The bind address the server will listen on. Ex: 0.0.0.0:8080
    pub bind_address: String,

    /// Path to database.
    pub storage_path: String,
}

impl ConfigType for ApiConfig {
    fn default_config() -> &'static str {
        DEFAULT_API_CONFIG
    }

    fn config_paths() -> Vec<std::path::PathBuf> {
        vec![PathBuf::from("/etc/parley/parley_web.toml")]
    }

    fn env_prefix() -> &'static str {
        "PARLEY_WEB_"
    }
}
