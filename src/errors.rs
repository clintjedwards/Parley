use std::ops::Deref;

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
