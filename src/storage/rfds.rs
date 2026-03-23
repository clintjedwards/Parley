use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Rfd {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub status: String,
    pub authors: String, // JSON
    pub created: String,
    pub updated: String,
}

pub async fn upsert_rfd(conn: &mut SqliteConnection, rfd: &Rfd) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO rfds (id, number, title, status, authors, created, updated) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(number) DO UPDATE SET \
           title = excluded.title, \
           status = excluded.status, \
           authors = excluded.authors, \
           updated = excluded.updated;",
    )
    .bind(&rfd.id)
    .bind(rfd.number)
    .bind(&rfd.title)
    .bind(&rfd.status)
    .bind(&rfd.authors)
    .bind(&rfd.created)
    .bind(&rfd.updated);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_rfd(conn: &mut SqliteConnection, id: &str) -> Result<Rfd, StorageError> {
    let query = sqlx::query_as::<_, Rfd>(
        "SELECT id, number, title, status, authors, created, updated FROM rfds WHERE id = ?;",
    )
    .bind(id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_rfds(conn: &mut SqliteConnection) -> Result<Vec<Rfd>, StorageError> {
    let query = sqlx::query_as::<_, Rfd>(
        "SELECT id, number, title, status, authors, created, updated FROM rfds ORDER BY number ASC;",
    );

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}
