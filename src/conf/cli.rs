use crate::conf::ConfigType;
use serde::{Deserialize, Serialize};

const DEFAULT_CLI_CONFIG: &str = include_str!("./default_cli_config.toml");

#[derive(Deserialize, Debug, Default, Clone)]
pub struct CliConfig {
    /// The URL of the API server.
    pub api_base_url: String,

    /// Provides extra debug output.
    pub debug: bool,

    /// Turn on extra detail for certain commands. Controls things like what format time is in.
    pub detail: bool,

    /// Don't verify server certificate; useful for development.
    pub insecure_skip_tls_verify: bool,

    /// What format the CLI will write to the terminal in.
    #[serde(deserialize_with = "crate::cli::deserialize_output_format")]
    pub output_format: crate::cli::OutputFormat,

    /// An API token to authenticate to the API server with.
    pub token: String,

    /// Unique identifier for user.
    pub username: String,

    /// Full, human readable name for user.
    pub full_name: String,
}

impl ConfigType for CliConfig {
    fn default_config() -> &'static str {
        DEFAULT_CLI_CONFIG
    }

    // We look for configuration in different spots depending on the env vars
    // to help developers not mix up their real config from their development config.

    #[cfg(debug_assertions)]
    fn config_paths() -> Vec<std::path::PathBuf> {
        let user_home = dirs::home_dir().expect("Unable to get home directory");

        vec![
            user_home.join(".parley_dev.toml"),
            user_home.join(".config/parley_dev.toml"),
        ]
    }

    #[cfg(not(debug_assertions))]
    fn config_paths() -> Vec<std::path::PathBuf> {
        let user_home = dirs::home_dir().expect("Unable to get home directory");

        vec![
            user_home.join(".parley.toml"),
            user_home.join(".config/parley.toml"),
        ]
    }

    fn env_prefix() -> &'static str {
        "PARLEY_"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conf::Configuration;
    use pretty_assertions::assert_eq;
    use std::env;

    #[test]
    fn load_from_environment_variables() {
        env::set_var("PARLEY_API_BASE_URL", "http://localhost:3001");
        env::set_var("PARLEY_TOKEN", "envoveride");

        let config = Configuration::<CliConfig>::load(None).unwrap();

        // Cleanup environment variables after test
        env::remove_var("PARLEY_API_BASE_URL");
        env::remove_var("PARLEY_TOKEN");

        assert_eq!(config.api_base_url, "http://localhost:3001");
        assert_eq!(config.token, "envoveride");
    }
}
