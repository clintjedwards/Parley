use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Token {
    pub id: String,
    pub hash: String,
    pub created: String,
    pub expires: String,
    pub disabled: i64,
    pub user: String,
    pub roles: String,    // JSON
    pub metadata: String, // JSON
}

pub async fn insert_token(conn: &mut SqliteConnection, token: &Token) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO tokens (id, hash, created, expires, disabled, user, roles, metadata) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?);",
    )
    .bind(&token.id)
    .bind(&token.hash)
    .bind(&token.created)
    .bind(&token.expires)
    .bind(token.disabled)
    .bind(&token.user)
    .bind(&token.roles)
    .bind(&token.metadata);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_token_by_hash(
    conn: &mut SqliteConnection,
    hash: &str,
) -> Result<Token, StorageError> {
    let query = sqlx::query_as::<_, Token>(
        "SELECT id, hash, created, expires, disabled, user, roles, metadata \
         FROM tokens WHERE hash = ?;",
    )
    .bind(hash);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn get_token(conn: &mut SqliteConnection, id: &str) -> Result<Token, StorageError> {
    let query = sqlx::query_as::<_, Token>(
        "SELECT id, hash, created, expires, disabled, user, roles, metadata \
         FROM tokens WHERE id = ?;",
    )
    .bind(id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_tokens(conn: &mut SqliteConnection) -> Result<Vec<Token>, StorageError> {
    let query = sqlx::query_as::<_, Token>(
        "SELECT id, hash, created, expires, disabled, user, roles, metadata \
         FROM tokens ORDER BY created ASC;",
    );

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn disable_token(conn: &mut SqliteConnection, id: &str) -> Result<(), StorageError> {
    let query = sqlx::query("UPDATE tokens SET disabled = 1 WHERE id = ?;").bind(id);

    let sql = query.sql();
    let result = query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }

    Ok(())
}
