use crate::{
    api::{epoch_milli, is_valid_identifier, ApiState, PreflightOptions},
    http_error, storage,
};
use dropshot::{
    endpoint, ClientErrorStatusCode, HttpError, HttpResponseCreated, HttpResponseDeleted,
    HttpResponseOk, Path, RequestContext, TypedBody,
};
use rootcause::{compat::boxed_error::IntoBoxedError, prelude::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UserPathArgs {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct User {
    pub id: String,
    pub name: String,
    pub created: u64,
    pub modified: u64,
}

impl User {
    pub fn new(id: &str, name: &str) -> Self {
        User {
            id: id.into(),
            name: name.into(),
            created: epoch_milli(),
            modified: 0,
        }
    }
}

impl TryFrom<storage::users::User> for User {
    type Error = Report;

    fn try_from(value: storage::users::User) -> Result<Self, Report> {
        let modified = value.modified.parse::<u64>().context_with(|| {
            format!(
                "Could not parse field 'modified' from storage value '{}'",
                value.modified
            )
        })?;

        Ok(User {
            id: value.id,
            name: value.name,
            created: value.created as u64,
            modified,
        })
    }
}

impl From<User> for storage::users::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created: value.created as i64,
            modified: value.modified.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ListUsersResponse {
    pub users: Vec<User>,
}

/// List all users.
#[endpoint(
    method = GET,
    path = "/api/users",
    tags = ["Users"],
)]
pub async fn list_users(
    rqctx: RequestContext<Arc<ApiState>>,
) -> Result<HttpResponseOk<ListUsersResponse>, HttpError> {
    let api_state = rqctx.context();
    let _req_metadata = api_state
        .preflight_check(&rqctx.request, PreflightOptions { bypass_auth: false })
        .await?;

    let mut conn = match api_state.storage.read_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(http_error!(
                "Could not open connection to database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    let storage_users = match storage::users::list(&mut conn).await {
        Ok(users) => users,
        Err(e) => {
            return Err(http_error!(
                "Could not get objects from database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    let mut users: Vec<User> = vec![];

    for storage_user in storage_users {
        let user = User::try_from(storage_user).map_err(|e| {
            http_error!(
                "Could not parse object from database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            )
        })?;

        users.push(user);
    }

    Ok(HttpResponseOk(ListUsersResponse { users }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GetUserResponse {
    pub user: User,
}

/// Get user by id.
#[endpoint(
    method = GET,
    path = "/api/users/{user_id}",
    tags = ["Users"],
)]
pub async fn get_user(
    rqctx: RequestContext<Arc<ApiState>>,
    path_params: Path<UserPathArgs>,
) -> Result<HttpResponseOk<GetUserResponse>, HttpError> {
    let api_state = rqctx.context();
    let path = path_params.into_inner();
    let _req_metadata = api_state
        .preflight_check(&rqctx.request, PreflightOptions { bypass_auth: false })
        .await?;

    let mut conn = match api_state.storage.read_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(http_error!(
                "Could not open connection to database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    let storage_user = match storage::users::get(&mut conn, &path.user_id).await {
        Ok(user) => user,
        Err(e) => match e {
            storage::StorageError::NotFound => {
                return Err(HttpError::for_not_found(None, String::new()));
            }
            _ => {
                return Err(http_error!(
                    "Could not get object from database",
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    rqctx.request_id.clone(),
                    Some(e.into())
                ));
            }
        },
    };

    let user = User::try_from(storage_user).map_err(|e| {
        http_error!(
            "Could not parse object from database",
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            rqctx.request_id.clone(),
            Some(e.into())
        )
    })?;

    Ok(HttpResponseOk(GetUserResponse { user }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserRequest {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserResponse {
    pub user: User,
}

/// Create a new user.
#[endpoint(
    method = POST,
    path = "/api/users",
    tags = ["Users"],
)]
pub async fn create_user(
    rqctx: RequestContext<Arc<ApiState>>,
    body: TypedBody<CreateUserRequest>,
) -> Result<HttpResponseCreated<CreateUserResponse>, HttpError> {
    let api_state = rqctx.context();
    let body = body.into_inner();
    let _req_metadata = api_state
        .preflight_check(&rqctx.request, PreflightOptions { bypass_auth: false })
        .await?;

    if let Err(e) = is_valid_identifier(&body.id) {
        return Err(HttpError::for_bad_request(
            None,
            format!("'{}' is not a valid identifier; {}", &body.id, &e),
        ));
    };

    let mut conn = match api_state.storage.write_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(http_error!(
                "Could not open connection to database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    let new_user = User::new(&body.id, &body.name);

    if let Err(e) = storage::users::insert(&mut conn, &new_user.clone().into()).await {
        match e {
            storage::StorageError::Exists => {
                return Err(HttpError::for_client_error(
                    None,
                    ClientErrorStatusCode::CONFLICT,
                    "user already exists".into(),
                ));
            }
            _ => {
                return Err(http_error!(
                    "Could not insert object into database",
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    rqctx.request_id.clone(),
                    Some(e.into())
                ));
            }
        }
    };

    Ok(HttpResponseCreated(CreateUserResponse { user: new_user }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
}

impl From<UpdateUserRequest> for storage::users::UpdatableFields {
    fn from(value: UpdateUserRequest) -> Self {
        Self {
            id: None,
            name: value.name,
            modified: epoch_milli().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserResponse {
    pub user: User,
}

/// Update a user's details.
#[endpoint(
    method = PATCH,
    path = "/api/users/{user_id}",
    tags = ["Users"],
)]
pub async fn update_user(
    rqctx: RequestContext<Arc<ApiState>>,
    path_params: Path<UserPathArgs>,
    body: TypedBody<UpdateUserRequest>,
) -> Result<HttpResponseOk<UpdateUserResponse>, HttpError> {
    let api_state = rqctx.context();
    let body = body.into_inner();
    let path = path_params.into_inner();
    let _req_metadata = api_state
        .preflight_check(&rqctx.request, PreflightOptions { bypass_auth: false })
        .await?;

    let mut tx = match api_state.storage.open_tx().await {
        Ok(tx) => tx,
        Err(e) => {
            return Err(http_error!(
                "Could not open connection to database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    let updatable_fields = storage::users::UpdatableFields::from(body);

    if let Err(e) = storage::users::update(&mut tx, &path.user_id, updatable_fields).await {
        match e {
            storage::StorageError::NotFound => {
                return Err(HttpError::for_not_found(
                    None,
                    "user for id given does not exist".into(),
                ));
            }
            storage::StorageError::NoFieldsUpdated => {
                return Err(HttpError::for_bad_request(
                    None,
                    "no fields provided to update".into(),
                ));
            }
            _ => {
                return Err(http_error!(
                    "Could not update object in database",
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    rqctx.request_id.clone(),
                    Some(e.into())
                ));
            }
        }
    };

    let storage_user = match storage::users::get(&mut tx, &path.user_id).await {
        Ok(user) => user,
        Err(e) => match e {
            storage::StorageError::NotFound => {
                return Err(HttpError::for_not_found(
                    None,
                    "user for id given does not exist".into(),
                ));
            }
            _ => {
                return Err(http_error!(
                    "Could not get object from database",
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    rqctx.request_id.clone(),
                    Some(e.into())
                ));
            }
        },
    };

    if let Err(e) = tx.commit().await {
        error!(message = "Could not commit transaction to database", error = %e);
        return Err(HttpError::for_internal_error(format!(
            "Encountered error when attempting to write user to database; {:#?}",
            e
        )));
    };

    let user = User::try_from(storage_user).map_err(|e| {
        http_error!(
            "Could not parse object from database",
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            rqctx.request_id.clone(),
            Some(e.into())
        )
    })?;

    Ok(HttpResponseOk(UpdateUserResponse { user }))
}

/// Delete user by id.
#[endpoint(
    method = DELETE,
    path = "/api/users/{user_id}",
    tags = ["Users"],
)]
pub async fn delete_user(
    rqctx: RequestContext<Arc<ApiState>>,
    path_params: Path<UserPathArgs>,
) -> Result<HttpResponseDeleted, HttpError> {
    let api_state = rqctx.context();
    let path = path_params.into_inner();
    let _req_metadata = api_state
        .preflight_check(&rqctx.request, PreflightOptions { bypass_auth: false })
        .await?;

    let mut conn = match api_state.storage.write_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(http_error!(
                "Could not open connection to database",
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                rqctx.request_id.clone(),
                Some(e.into())
            ));
        }
    };

    if let Err(e) = storage::users::delete(&mut conn, &path.user_id).await {
        match e {
            storage::StorageError::NotFound => {
                return Err(HttpError::for_not_found(
                    None,
                    "user for id given does not exist".into(),
                ));
            }
            _ => {
                return Err(http_error!(
                    "Could not delete object from database",
                    hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    rqctx.request_id.clone(),
                    Some(e.into())
                ));
            }
        }
    };

    Ok(HttpResponseDeleted())
}
