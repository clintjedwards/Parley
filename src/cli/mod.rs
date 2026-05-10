use crate::conf::{cli::CliConfig, Configuration};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use rootcause::prelude::*;
use serde::{de, Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::{EnumString, VariantNames};

#[derive(Default, Debug, Clone, ValueEnum, Serialize, PartialEq, Eq, EnumString, VariantNames)]
#[strum(ascii_case_insensitive)]
#[serde(try_from = "String")]
pub(crate) enum OutputFormat {
    #[default]
    Pretty,
    Plain,
    Silent,
    Json,
}

pub(crate) fn deserialize_output_format<'de, D>(deserializer: D) -> Result<OutputFormat, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    OutputFormat::from_str(&s, true).map_err(de::Error::custom)
}

impl From<OutputFormat> for polyfmt::Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Pretty => polyfmt::Format::Spinner,
            OutputFormat::Plain => polyfmt::Format::Plain,
            OutputFormat::Silent => polyfmt::Format::Silent,
            OutputFormat::Json => polyfmt::Format::Json,
        }
    }
}

/// Parley is a terminal-native tool for writing, publishing, and discussing technical documents like RFDs.
///
/// ## Configuration
///
/// Settings are loaded from multiple sources in order, with later sources overriding earlier ones:
///   1. Built-in defaults
///   2. Config file (see locations below)
///   3. Environment variables
///
/// ### Config file locations
///
///   /etc/parley/parley.toml
///   ~/.config/parley/parley.toml
///   ./parley.toml
///
/// ### Environment variables
///
/// All config keys are available as env vars with the prefix PARLEY_ and double underscores
/// for nesting. Examples:
///
///   PARLEY_SERVER__BIND_ADDRESS=0.0.0.0:7000
///   PARLEY_DEVELOPMENT__BYPASS_AUTH=true
#[derive(Debug, Parser, Clone)]
#[command(name = "parley")]
#[command(bin_name = "parley")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    /// Start the Parley server.
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Debug, Subcommand, Clone)]
enum ServerCommands {
    /// Start the HTTP API server.
    Start,
}

#[derive(Debug, Clone)]
pub struct Cli {
    args: Args,
}

impl Cli {
    pub fn new() -> Result<Self, Report> {
        let args = Args::parse();
        Ok(Cli { args })
    }

    pub async fn run(&mut self) -> Result<()> {
        match self.args.clone().command {
            Commands::Server { command } => match command {
                ServerCommands::Start => crate::server::start().await,
            },
        }
    }
}
