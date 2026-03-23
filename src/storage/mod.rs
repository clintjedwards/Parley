//! Data storage layer for Parley.
//!
//! Uses SQLite with split read/write connection pools:
//!   - Read pool: up to 10 concurrent connections
//!   - Write pool: exactly 1 connection (serialises all writes)
//!
//! All timestamps are stored as epoch milliseconds in TEXT columns to avoid
//! SQLite's i64 limitation when working with u64 values.
//!
//! ## Transactions
//!
//! ```ignore
//! let mut tx = db.open_tx().await?;
//! storage::rfds::insert_rfd(&mut tx, &rfd).await?;
//! tx.commit().await?;
//! ```

pub mod events;
pub mod messages;
pub mod rfds;
pub mod revisions;
pub mod roles;
pub mod threads;
pub mod tokens;

use anyhow::Result;
use futures::TryFutureExt;
use sqlx::{
    Pool, Sqlite, SqliteConnection, Transaction, migrate,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{fs::File, io, ops::Deref, path::Path, str::FromStr};

use crate::errors::{StorageError, map_sqlx_error};

#[derive(Debug, Clone)]
pub struct Db {
    write_pool: Pool<Sqlite>,
    read_pool: Pool<Sqlite>,
}

fn touch_file(path: &Path) -> io::Result<()> {
    if !path.exists() {
        File::create(path)?;
    }
    Ok(())
}

impl Db {
    pub async fn new(path: &str) -> Result<Self> {
        touch_file(Path::new(path)).unwrap();

        let connect_options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path))
            .unwrap()
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5));

        let read_pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options.clone())
            .await?;

        let write_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await?;

        migrate!("src/storage/migrations")
            .run(&write_pool)
            .await
            .unwrap();

        Ok(Db {
            write_pool,
            read_pool,
        })
    }

    pub async fn write_conn(&self) -> Result<PoolConnection<Sqlite>, StorageError> {
        self.write_pool
            .acquire()
            .await
            .map_err(|e| StorageError::Connection(format!("{:?}", e)))
    }

    pub async fn read_conn(&self) -> Result<PoolConnection<Sqlite>, StorageError> {
        self.read_pool
            .acquire()
            .await
            .map_err(|e| StorageError::Connection(format!("{:?}", e)))
    }

    pub async fn open_tx(&self) -> Result<Transaction<'_, Sqlite>, StorageError> {
        self.write_pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(|e| StorageError::Connection(format!("{:?}", e)))
    }
}
