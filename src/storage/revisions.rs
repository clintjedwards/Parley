use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct RfdRevision {
    pub id: String,
    pub rfd_id: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub rendered_html: String,
    pub title: String,
    pub status: String,
    pub authors: String, // JSON
    pub created: String,
}

pub async fn insert_revision(
    conn: &mut SqliteConnection,
    revision: &RfdRevision,
) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO rfd_revisions \
         (id, rfd_id, commit_sha, commit_message, rendered_html, title, status, authors, created) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);",
    )
    .bind(&revision.id)
    .bind(&revision.rfd_id)
    .bind(&revision.commit_sha)
    .bind(&revision.commit_message)
    .bind(&revision.rendered_html)
    .bind(&revision.title)
    .bind(&revision.status)
    .bind(&revision.authors)
    .bind(&revision.created);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_latest_revision(
    conn: &mut SqliteConnection,
    rfd_id: &str,
) -> Result<RfdRevision, StorageError> {
    let query = sqlx::query_as::<_, RfdRevision>(
        "SELECT id, rfd_id, commit_sha, commit_message, rendered_html, title, status, authors, created \
         FROM rfd_revisions WHERE rfd_id = ? ORDER BY created DESC LIMIT 1;",
    )
    .bind(rfd_id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_revisions(
    conn: &mut SqliteConnection,
    rfd_id: &str,
) -> Result<Vec<RfdRevision>, StorageError> {
    let query = sqlx::query_as::<_, RfdRevision>(
        "SELECT id, rfd_id, commit_sha, commit_message, rendered_html, title, status, authors, created \
         FROM rfd_revisions WHERE rfd_id = ? ORDER BY created DESC;",
    )
    .bind(rfd_id);

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}
