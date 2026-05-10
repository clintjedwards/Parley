mod error;

use crate::conf::{cli::CliConfig, Configuration};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use polyfmt::{println, question};
use rand::seq::IndexedRandom;
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
///
///   1. Built-in defaults
///
///   2. Config file (see locations below)
///
///   3. Environment variables
///
/// ### Config file locations
///
///   $HOME/.config/parley/parley.toml
///
///   $HOME/.parley.toml
///
/// ### Environment variables
///
/// All config keys are available as env vars with the prefix PARLEY_ and double underscores
/// for nesting. Examples:
///
///   PARLEY_SERVER__BIND_ADDRESS=0.0.0.0:7000
///
///   PARLEY_DEVELOPMENT__BYPASS_AUTH=true
///
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

    /// Initialize config file and register with server.
    ///
    /// Allows the caller to configure the basics about themselves and their use of Parley
    /// in order to set up their local configuration. Additionally, we register the existence
    /// of the new user into the Parley API.
    Init,
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

        let conf =
            Configuration::<CliConfig>::load(None).context("Could not initialize configuration")?;

        let output_format = polyfmt::Format::from(conf.output_format.clone());

        error::alter_error_formatter(conf.debug);

        let fmtter_options = polyfmt::Options {
            debug: conf.debug,
            padding: 1,
            ..Default::default()
        };

        let fmtter = polyfmt::new(output_format, fmtter_options);

        polyfmt::set_global_formatter(fmtter);

        Ok(Cli { args })
    }

    pub async fn run(&mut self) -> Result<(), Report> {
        match self.args.clone().command {
            Commands::Server { command } => match command {
                ServerCommands::Start => crate::api::start_web_services().await,
            },
            Commands::Init => init(),
        }
    }
}

fn init() -> Result<(), Report> {
    let taglines = [
        "Bikeshedding at terminal velocity.",
        "Finally, a terminal native way to argue about variable names.",
        "Your opinions deserve more sane infrastructure.",
        "For teams with strong opinions and weak consensus.",
        "Because your team needs 47 opinions before merging.",
        "Turn your bikeshed into a cathedral.",
        "For devs who think GitHub comments are too mainstream.",
        "Overengineered opinions, elegantly delivered.",
        "For teams that argue in O(n log n).",
        "You're gonna want a paper trail for who designed that monstrosity.",
        "We turned your meeting into markdown. You're welcome.",
    ];

    let tagline = taglines.choose(&mut rand::rng()).unwrap();

    println!("Parley :: {tagline}\n\n");
    println!("Let's set up your config file: \n\n");

    question!(
        "What is the URL of the Parley API server ({}): ",
        "ex. https://parley.example.com".dimmed()
    );

    // TODO(cje): We should ping the server here before the user continues.

    question!(
        "Choose a unique identifier for yourself aka a screen name. This will be how people reference you in documents. ({}): @",
        "ex. clintjedwards".dimmed()
    );

    // TODO(): Here we check to make sure the user's name isn't chosen already.

    question!(
        "Lastly, Let's have your name or what you like to be called. This is how your name will show up in documents. ({}): ",
        "Clint J. Edwards".dimmed()
    );

    // Before we overwrite the configuration file we should check if there is one that exists already and prompt the
    // user for confirmation if true.
    // Then tell the user what we've set for them and where the configuration file is.
    Ok(())
}
