use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Event {
    pub id: String,
    pub kind: String,
    pub actor: Option<String>,
    pub rfd_id: Option<String>,
    pub thread_id: Option<String>,
    pub payload: String, // JSON
    pub created: String,
}

pub async fn insert_event(conn: &mut SqliteConnection, event: &Event) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO events (id, kind, actor, rfd_id, thread_id, payload, created) \
         VALUES (?, ?, ?, ?, ?, ?, ?);",
    )
    .bind(&event.id)
    .bind(&event.kind)
    .bind(&event.actor)
    .bind(&event.rfd_id)
    .bind(&event.thread_id)
    .bind(&event.payload)
    .bind(&event.created);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn list_events_for_rfd(
    conn: &mut SqliteConnection,
    rfd_id: &str,
) -> Result<Vec<Event>, StorageError> {
    let query = sqlx::query_as::<_, Event>(
        "SELECT id, kind, actor, rfd_id, thread_id, payload, created \
         FROM events WHERE rfd_id = ? ORDER BY created DESC;",
    )
    .bind(rfd_id);

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_all_events(conn: &mut SqliteConnection) -> Result<Vec<Event>, StorageError> {
    let query = sqlx::query_as::<_, Event>(
        "SELECT id, kind, actor, rfd_id, thread_id, payload, created \
         FROM events ORDER BY created DESC LIMIT 500;",
    );

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}
