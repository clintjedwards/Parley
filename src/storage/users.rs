use crate::api::epoch_milli;
use crate::storage::{map_sqlx_error, StorageError};
use futures::TryFutureExt;
use sqlx::{Execute, FromRow, QueryBuilder, Sqlite, SqliteConnection};

#[derive(Clone, Debug, Default, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub created: i64,
    pub modified: String,
}

#[derive(Clone, Debug)]
pub struct UpdatableFields {
    pub id: Option<String>,
    pub name: Option<String>,
    pub modified: String,
}

impl Default for UpdatableFields {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            modified: epoch_milli().to_string(),
        }
    }
}

pub async fn insert(conn: &mut SqliteConnection, user: &User) -> Result<(), StorageError> {
    let query = sqlx::query("INSERT INTO users (id, name, created, modified) VALUES (?, ?, ?, ?);")
        .bind(&user.id)
        .bind(&user.name)
        .bind(user.created)
        .bind(&user.modified);

    let sql = query.sql();

    query
        .execute(conn)
        .map_err(|e| map_sqlx_error(e, sql))
        .await?;

    Ok(())
}

pub async fn list(conn: &mut SqliteConnection) -> Result<Vec<User>, StorageError> {
    let query = sqlx::query_as::<_, User>("SELECT id, name, created, modified FROM users;");

    let sql = query.sql();

    query
        .fetch_all(conn)
        .map_err(|e| map_sqlx_error(e, sql))
        .await
}

pub async fn get(conn: &mut SqliteConnection, id: &str) -> Result<User, StorageError> {
    let query =
        sqlx::query_as::<_, User>("SELECT id, name, created, modified FROM users WHERE id = ?;")
            .bind(id);

    let sql = query.sql();

    query
        .fetch_one(conn)
        .map_err(|e| map_sqlx_error(e, sql))
        .await
}

pub async fn update(
    conn: &mut SqliteConnection,
    id: &str,
    fields: UpdatableFields,
) -> Result<(), StorageError> {
    let mut update_query: QueryBuilder<Sqlite> = QueryBuilder::new(r#"UPDATE users SET "#);
    let mut updated_fields_total = 0;

    if let Some(value) = &fields.id {
        if updated_fields_total > 0 {
            update_query.push(", ");
        }
        update_query.push("id = ");
        update_query.push_bind(value);
        updated_fields_total += 1;
    }

    if let Some(value) = &fields.name {
        if updated_fields_total > 0 {
            update_query.push(", ");
        }
        update_query.push("name = ");
        update_query.push_bind(value);
        updated_fields_total += 1;
    }

    // If no fields were updated, return an error
    if updated_fields_total == 0 {
        return Err(StorageError::NoFieldsUpdated);
    }

    update_query.push(", ");
    update_query.push("modified = ");
    update_query.push_bind(fields.modified);

    update_query.push(" WHERE id = ");
    update_query.push_bind(id);
    update_query.push(";");

    let update_query = update_query.build();

    let sql = update_query.sql();

    update_query
        .execute(conn)
        .await
        .map(|_| ())
        .map_err(|e| map_sqlx_error(e, sql))
}

pub async fn delete(conn: &mut SqliteConnection, id: &str) -> Result<(), StorageError> {
    let query = sqlx::query("DELETE FROM users WHERE id = ?;").bind(id);

    let sql = query.sql();

    query
        .execute(conn)
        .map_ok(|_| ())
        .map_err(|e| map_sqlx_error(e, sql))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::tests::TestHarness;
    use sqlx::{pool::PoolConnection, Sqlite};

    async fn setup() -> Result<(TestHarness, PoolConnection<Sqlite>), Box<dyn std::error::Error>> {
        let harness = TestHarness::new().await;
        let mut conn = harness.write_conn().await.unwrap();

        let user = User {
            id: "some_id".into(),
            name: "some_name".into(),
            created: 0,
            modified: "some_time_mod".into(),
        };

        insert(&mut conn, &user).await?;

        Ok((harness, conn))
    }

    #[tokio::test]
    async fn test_list_users() {
        let (_harness, mut conn) = setup().await.expect("Failed to set up DB");

        let users = list(&mut conn).await.expect("Failed to list users");

        assert!(!users.is_empty(), "No users returned");

        let some_user = users
            .iter()
            .find(|u| u.id == "some_id")
            .expect("User not found");
        assert_eq!(some_user.name, "some_name");
    }

    #[tokio::test]
    async fn test_insert_user() {
        let (_harness, mut conn) = setup().await.expect("Failed to set up DB");

        let new_user = User {
            id: "new_id".into(),
            name: "new_name".into(),
            created: 0,
            modified: "some_other_time".into(),
        };

        insert(&mut conn, &new_user)
            .await
            .expect("Failed to insert user");

        let retrieved_user = get(&mut conn, "new_id")
            .await
            .expect("Failed to retrieve user");

        assert_eq!(retrieved_user.id, "new_id");
        assert_eq!(retrieved_user.name, "new_name");
    }

    #[tokio::test]
    async fn test_get_user() {
        let (_harness, mut conn) = setup().await.expect("Failed to set up DB");

        let user = get(&mut conn, "some_id").await.expect("Failed to get user");

        assert_eq!(user.id, "some_id");
        assert_eq!(user.name, "some_name");

        assert!(
            get(&mut conn, "non_existent").await.is_err(),
            "Unexpectedly found a user"
        );
    }

    #[tokio::test]
    async fn test_update_user() {
        let (_harness, mut conn) = setup().await.expect("Failed to set up DB");

        let fields_to_update = UpdatableFields {
            id: None,
            name: Some("updated_name".into()),
            modified: "updated_time".into(),
        };

        update(&mut conn, "some_id", fields_to_update)
            .await
            .expect("Failed to update user");

        let updated_user = get(&mut conn, "some_id")
            .await
            .expect("Failed to retrieve updated user");

        assert_eq!(updated_user.name, "updated_name");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (_harness, mut conn) = setup().await.expect("Failed to set up DB");

        delete(&mut conn, "some_id")
            .await
            .expect("Failed to delete user");

        assert!(
            get(&mut conn, "some_id").await.is_err(),
            "User was not deleted"
        );
    }
}
