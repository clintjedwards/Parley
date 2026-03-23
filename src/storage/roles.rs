use crate::errors::{StorageError, map_sqlx_error};
use sqlx::{Execute, FromRow, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Role {
    pub id: String,
    pub description: String,
    pub permissions: String, // JSON
    pub system_role: i64,
}

pub async fn insert_role(conn: &mut SqliteConnection, role: &Role) -> Result<(), StorageError> {
    let query = sqlx::query(
        "INSERT INTO roles (id, description, permissions, system_role) VALUES (?, ?, ?, ?);",
    )
    .bind(&role.id)
    .bind(&role.description)
    .bind(&role.permissions)
    .bind(role.system_role);

    let sql = query.sql();
    query.execute(conn).await.map_err(|e| map_sqlx_error(e, sql))?;
    Ok(())
}

pub async fn get_role(conn: &mut SqliteConnection, id: &str) -> Result<Role, StorageError> {
    let query = sqlx::query_as::<_, Role>(
        "SELECT id, description, permissions, system_role FROM roles WHERE id = ?;",
    )
    .bind(id);

    let sql = query.sql();
    query.fetch_one(conn).await.map_err(|e| map_sqlx_error(e, sql))
}

pub async fn list_roles(conn: &mut SqliteConnection) -> Result<Vec<Role>, StorageError> {
    let query = sqlx::query_as::<_, Role>(
        "SELECT id, description, permissions, system_role FROM roles ORDER BY id ASC;",
    );

    let sql = query.sql();
    query.fetch_all(conn).await.map_err(|e| map_sqlx_error(e, sql))
}
