//! Data storage layer.
//!
//! Uses SQLite with split read/write connection pools:
//!   - Read pool: up to 10 concurrent connections
//!   - Write pool: exactly 1 connection (serialises all writes)
//!
//! All timestamps are stored as epoch milliseconds in TEXT columns to avoid
//! SQLite's i64 limitation when working with u64 values.
//!
//! ## Transactions
//! ```ignore
//! let mut tx = storage.open_tx().await;
//! let some_db_call(&mut tx).await;
//! let some_other_db_call(&mut tx).await;
//! tx.commit() // Make sure you call commit or changes made inside the transaction wont be changed.

pub mod events;
pub mod messages;
pub mod revisions;
pub mod rfds;
pub mod roles;
pub mod threads;
pub mod tokens;

use anyhow::Result;
use futures::TryFutureExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{
    migrate,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite, SqliteConnection, Transaction,
};
use std::{fs::File, io, ops::Deref, path::Path, str::FromStr};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum StorageError {
    #[error("could not establish connection to database; {0}")]
    Connection(String),

    #[error("requested entity not found")]
    NotFound,

    #[error("entity already exists")]
    Exists,

    #[error("request did not update any fields")]
    NoFieldsUpdated,

    #[error("did not find required foreign key for query; {0}")]
    ForeignKeyViolation(String),

    #[error(
        "unexpected storage error occurred; code: {code:?}; message: {message}; query: {query}"
    )]
    GenericDBError {
        code: Option<String>,
        message: String,
        query: String,
    },
}

/// Maps sqlx errors to domain-specific StorageError variants.
/// SQLite error codes: https://www.sqlite.org/rescode.html
pub fn map_sqlx_error(e: sqlx::Error, query: &str) -> StorageError {
    match e {
        sqlx::Error::RowNotFound => StorageError::NotFound,
        sqlx::Error::Database(database_err) => {
            if let Some(err_code) = database_err.code() {
                match err_code.deref() {
                    "1555" => StorageError::Exists,
                    "787" => StorageError::ForeignKeyViolation(database_err.to_string()),
                    _ => StorageError::GenericDBError {
                        code: Some(err_code.to_string()),
                        message: format!("Unmapped error occurred; {}", database_err),
                        query: query.into(),
                    },
                }
            } else {
                StorageError::GenericDBError {
                    code: None,
                    message: database_err.to_string(),
                    query: query.into(),
                }
            }
        }
        _ => StorageError::GenericDBError {
            code: None,
            message: e.to_string(),
            query: query.into(),
        },
    }
}

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

        // We create two different pools of connections. The read pool has many connections and is high concurrency.
        // The write pool is essentially a single connection in which only one write can be made at a time.
        // Not using this paradigm may result in sqlite "database is locked(error: 5)" errors because of the
        // manner in which sqlite handles transactions.
        let connect_options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path))
            .unwrap()
            // * journal_mode: Turns on WAL mode which increases concurrency and reliability.
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            // * synchronous: Tells sqlite to not sync to disk as often and specifically only focus on syncing at critcal
            //   junctures. This makes sqlite speedier and also has no downside because we have WAL mode.
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            // * foreign_keys: Turns on relational style foreign keys. A must have.
            .foreign_keys(true)
            // * busy_timeout: How long should a sqlite query try before it returns an error. Very helpful to avoid
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
