use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub author: String,
    pub body: String,
    pub body_html: String,
    pub created: String,
    pub updated: Option<String>,
}

pub async fn insert_message(
    conn: &mut SqliteConnection,
    message: &Message,
) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO messages (id, thread_id, author, body, body_html, created) \
         VALUES (?, ?, ?, ?, ?, ?);",
    )
    .bind(&message.id)
    .bind(&message.thread_id)
    .bind(&message.author)
    .bind(&message.body)
    .bind(&message.body_html)
    .bind(&message.created);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_message(conn: &mut SqliteConnection, id: &str) -> Result<Message, StorageError> {
    let query = sqlx::query_as::<_, Message>(
        "SELECT id, thread_id, author, body, body_html, created, updated FROM messages WHERE id = ?;",
    )
    .bind(id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_messages(
    conn: &mut SqliteConnection,
    thread_id: &str,
) -> Result<Vec<Message>, StorageError> {
    let query = sqlx::query_as::<_, Message>(
        "SELECT id, thread_id, author, body, body_html, created, updated \
         FROM messages WHERE thread_id = ? ORDER BY created ASC;",
    )
    .bind(thread_id);

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn update_message(
    conn: &mut SqliteConnection,
    id: &str,
    body: &str,
    body_html: &str,
    updated: &str,
) -> Result<(), StorageError> {
    let query =
        sqlx::query("UPDATE messages SET body = ?, body_html = ?, updated = ? WHERE id = ?;")
            .bind(body)
            .bind(body_html)
            .bind(updated)
            .bind(id);

    let sql = query.sql();
    let result = query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }

    Ok(())
}

pub async fn delete_message(conn: &mut SqliteConnection, id: &str) -> Result<(), StorageError> {
    let query = sqlx::query("DELETE FROM messages WHERE id = ?;").bind(id);

    let sql = query.sql();
    let result = query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;

    if result.rows_affected() == 0 {
        return Err(StorageError::NotFound);
    }

    Ok(())
}
