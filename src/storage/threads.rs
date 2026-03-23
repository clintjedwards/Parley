use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Thread {
    pub id: String,
    pub rfd_id: String,
    pub resolved: i64,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub created_by: String,
    pub created: String,
    pub updated: String,
}

pub async fn insert_thread(
    conn: &mut SqliteConnection,
    thread: &Thread,
) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO threads (id, rfd_id, resolved, resolved_by, resolved_at, created_by, created, updated) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?);",
    )
    .bind(&thread.id)
    .bind(&thread.rfd_id)
    .bind(thread.resolved)
    .bind(&thread.resolved_by)
    .bind(&thread.resolved_at)
    .bind(&thread.created_by)
    .bind(&thread.created)
    .bind(&thread.updated);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_thread(conn: &mut SqliteConnection, id: &str) -> Result<Thread, StorageError> {
    let query = sqlx::query_as::<_, Thread>(
        "SELECT id, rfd_id, resolved, resolved_by, resolved_at, created_by, created, updated \
         FROM threads WHERE id = ?;",
    )
    .bind(id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_threads(
    conn: &mut SqliteConnection,
    rfd_id: &str,
) -> Result<Vec<Thread>, StorageError> {
    let query = sqlx::query_as::<_, Thread>(
        "SELECT id, rfd_id, resolved, resolved_by, resolved_at, created_by, created, updated \
         FROM threads WHERE rfd_id = ? ORDER BY created ASC;",
    )
    .bind(rfd_id);

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn resolve_thread(
    conn: &mut SqliteConnection,
    id: &str,
    resolved_by: &str,
    resolved_at: &str,
) -> Result<(), StorageError> {
    let query = sqlx::query(
        "UPDATE threads SET resolved = 1, resolved_by = ?, resolved_at = ?, updated = ? WHERE id = ?;",
    )
    .bind(resolved_by)
    .bind(resolved_at)
    .bind(resolved_at)
    .bind(id);

    let sql = query.sql();
    let result = query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }

    Ok(())
}

pub async fn unresolve_thread(
    conn: &mut SqliteConnection,
    id: &str,
    updated: &str,
) -> Result<(), StorageError> {
    let query = sqlx::query(
        "UPDATE threads SET resolved = 0, resolved_by = NULL, resolved_at = NULL, updated = ? WHERE id = ?;",
    )
    .bind(updated)
    .bind(id);

    let sql = query.sql();
    let result = query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }

    Ok(())
}
