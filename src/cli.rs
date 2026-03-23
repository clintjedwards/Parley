use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fmt::Debug;

/// Parley is a terminal-native tool for writing, publishing, and discussing RFDs.
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
///   PARLEY_GIT__REPO_PATH=/var/lib/parley/repo
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

    /// Manage tokens.
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },

    /// Launch the terminal UI (connects to configured server).
    Tui,
}

#[derive(Debug, Subcommand, Clone)]
enum ServerCommands {
    /// Start the HTTP API server.
    Start,

    /// Print a bootstrap token. Only works once — if a bootstrap token already exists this
    /// will fail. This is how the first admin gets access.
    Bootstrap,

    /// Manually trigger a full re-sync of the RFD repository.
    Sync,
}

#[derive(Debug, Subcommand, Clone)]
enum TokenCommands {
    /// Create a new token and print the plaintext secret once.
    Create {
        /// Display name for this token's owner.
        #[arg(long)]
        user: String,

        /// Role to assign (bootstrap, admin, member, reader).
        #[arg(long, default_value = "member")]
        role: String,
    },

    /// List all tokens.
    List,

    /// Disable a token by ID.
    Disable {
        /// Token ID.
        id: String,
    },
}

#[derive(Debug, Clone)]
pub struct Cli {
    args: Args,
}

impl Cli {
    pub fn new() -> Result<Self> {
        let args = Args::parse();
        Ok(Cli { args })
    }

    pub async fn run(&mut self) -> Result<()> {
        match self.args.clone().command {
            Commands::Server { command } => match command {
                ServerCommands::Start => crate::server::start().await,
                ServerCommands::Bootstrap => crate::server::bootstrap().await,
                ServerCommands::Sync => crate::server::sync().await,
            },
            Commands::Token { command } => match command {
                TokenCommands::Create { user, role } => {
                    crate::server::token_create(user, role).await
                }
                TokenCommands::List => crate::server::token_list().await,
                TokenCommands::Disable { id } => crate::server::token_disable(id).await,
            },
            Commands::Tui => crate::tui::run().await,
        }
    }
}
