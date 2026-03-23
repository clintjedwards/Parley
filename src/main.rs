mod cli;
mod conf;
mod errors;
mod models;
mod server;
mod storage;
mod tui;

use tracing::error;

#[tokio::main]
async fn main() {
    let mut cli = match cli::Cli::new() {
        Ok(cli) => cli,
        Err(e) => {
            error!("{:?}", e);
            std::process::exit(1)
        }
    };

    match cli.run().await {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e);
            std::process::exit(1)
        }
    }
}
