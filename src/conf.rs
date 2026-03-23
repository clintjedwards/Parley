use anyhow::Result;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;
use std::path::PathBuf;

pub trait ConfigType: Deserialize<'static> {
    fn default_config() -> &'static str;
    fn config_paths() -> Vec<PathBuf>;
    fn env_prefix() -> &'static str;
}

pub struct Configuration<T: ConfigType> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ConfigType> Configuration<T> {
    pub fn load(path_override: Option<PathBuf>) -> Result<T> {
        let mut config = Figment::new().merge(Toml::string(T::default_config()));

        if let Some(path) = path_override {
            config = config.merge(Toml::file(path));
        } else {
            for path in T::config_paths() {
                config = config.merge(Toml::file(path));
            }
        }

        // Double underscore separates nesting levels in env vars.
        // e.g. PARLEY_SERVER__BIND_ADDRESS maps to server.bind_address
        config = config.merge(Env::prefixed(T::env_prefix()).split("__"));
        let parsed_config: T = config.extract()?;

        Ok(parsed_config)
    }
}

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ApiConfig {
    pub server: Server,
    pub git: Git,
    pub webhook: Webhook,
    pub typst: Typst,
    pub log: Log,
    pub development: Development,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Server {
    /// Address and port the HTTP server listens on. Ex: 0.0.0.0:7000
    pub bind_address: String,

    /// Path to the SQLite database file.
    pub storage_path: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Git {
    /// Absolute path where the RFD repo is (or will be) cloned.
    pub repo_path: String,

    /// HTTPS or SSH URL of the GitHub repo containing RFDs.
    pub remote_url: String,

    /// Branch to track.
    pub branch: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Webhook {
    /// Must match the secret configured in the GitHub repo's webhook settings.
    pub secret: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Typst {
    /// Path to the typst binary. Defaults to "typst" (assumes it is on $PATH).
    pub binary_path: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Log {
    pub level: String,
    pub pretty: bool,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Development {
    /// Skip all auth checks. Useful for local development.
    pub bypass_auth: bool,
}

impl ConfigType for ApiConfig {
    fn default_config() -> &'static str {
        DEFAULT_CONFIG
    }

    fn config_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/etc/parley/parley.toml"),
            PathBuf::from("parley.toml"),
        ]
    }

    fn env_prefix() -> &'static str {
        "PARLEY_"
    }
}
