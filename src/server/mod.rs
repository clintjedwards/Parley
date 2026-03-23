pub mod api;
pub mod permissioning;
pub mod webhook;
pub mod ws;

use crate::conf::{ApiConfig, Configuration};
use anyhow::Result;

pub async fn start() -> Result<()> {
    let conf = Configuration::<ApiConfig>::load(None)?;
    // TODO: init logger
    // TODO: init storage
    // TODO: init system roles
    // TODO: start webhook + api server
    todo!()
}

pub async fn bootstrap() -> Result<()> {
    // TODO: connect to running server and create bootstrap token
    // Print plaintext token to stdout
    todo!()
}

pub async fn sync() -> Result<()> {
    // TODO: connect to running server and trigger a full repo re-sync
    todo!()
}

pub async fn token_create(user: String, role: String) -> Result<()> {
    // TODO: connect to running server and create token
    // Print plaintext token to stdout
    todo!()
}

pub async fn token_list() -> Result<()> {
    // TODO: connect to running server and list tokens
    todo!()
}

pub async fn token_disable(id: String) -> Result<()> {
    // TODO: connect to running server and disable token
    todo!()
}
