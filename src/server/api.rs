use crate::{conf, storage};
use dropshot::{
    endpoint, ApiDescription, ClientErrorStatusCode, ConfigDropshot, HttpError, HttpResponseOk,
    HttpServer, RequestContext, RequestInfo, ServerBuilder,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, pin::Pin, str::FromStr, sync::atomic, sync::Arc};

const BUILD_SEMVER: &str = env!("BUILD_SEMVER");
const BUILD_COMMIT: &str = env!("BUILD_COMMIT");

/// A constant for the header that tracks which version of the API a client has requested.
const API_VERSION_HEADER: &str = "parley-api-version";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ApiVersion {
    V0,
}

impl ApiVersion {
    pub fn to_list() -> [String; 1] {
        ["v0".into()]
    }
}

impl FromStr for ApiVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "v0" => Ok(ApiVersion::V0),
            _ => Err(anyhow::anyhow!("Invalid API version")),
        }
    }
}

/// Holds objects that are created and used over the lifetime of a single request.
///
/// This is different from [`dropshot::RequestContext`] since that is automatically created for us but we need some
/// more Gofer specific information.
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    #[allow(dead_code)]
    api_version: ApiVersion,
}

pub struct PreflightOptions {
    pub bypass_auth: bool,
}

#[derive(Debug)]
pub struct ApiState {
    pub config: conf::ApiConfig,
    pub storage: storage::Db,
}

impl ApiState {
    pub async fn preflight_check(
        &self,
        _request: &RequestInfo,
        options: PreflightOptions,
    ) -> Result<(), HttpError> {
        if options.bypass_auth || self.config.development.bypass_auth {
            return Ok(());
        }

        // TODO: extract Bearer token from Authorization header
        // TODO: hash token, look up in DB
        // TODO: check disabled + expiry
        // TODO: resolve roles → permissions
        // TODO: call permissioning::is_authorized

        Err(HttpError::for_client_error(
            None,
            ClientErrorStatusCode::UNAUTHORIZED,
            "Route requires authorization".into(),
        ))
    }
}

// ============================================================
// Token endpoints
// ============================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BootstrapTokenResponse {
    pub token: String, // plaintext — shown once
}

/// Create the bootstrap token. No auth required. Callable exactly once.
#[endpoint(method = POST, path = "/api/tokens/bootstrap", tags = ["tokens"])]
pub async fn bootstrap_token(
    _rqctx: RequestContext<Arc<ApiState>>,
) -> Result<HttpResponseOk<BootstrapTokenResponse>, HttpError> {
    // TODO: check if a bootstrap token already exists (look for bootstrap event in events table)
    // TODO: generate random 32-byte token, SHA256 hash it, store hash
    // TODO: insert event token.bootstrapped
    // TODO: return plaintext token
    todo!()
}

/// List all tokens. Admin only.
#[endpoint(method = GET, path = "/api/tokens", tags = ["tokens"])]
pub async fn list_tokens(
    _rqctx: RequestContext<Arc<ApiState>>,
) -> Result<HttpResponseOk<Vec<crate::models::Token>>, HttpError> {
    // TODO: preflight_check admin
    // TODO: storage::tokens::list_tokens
    todo!()
}

// ============================================================
// RFD endpoints
// ============================================================

/// List all RFDs.
#[endpoint(method = GET, path = "/api/rfds", tags = ["rfds"])]
pub async fn list_rfds(
    _rqctx: RequestContext<Arc<ApiState>>,
) -> Result<HttpResponseOk<Vec<crate::models::Rfd>>, HttpError> {
    // TODO: preflight_check member read rfds
    // TODO: storage::rfds::list_rfds
    todo!()
}

/// Get an RFD with its latest rendered revision.
#[endpoint(method = GET, path = "/api/rfds/{id}", tags = ["rfds"])]
pub async fn get_rfd(
    _rqctx: RequestContext<Arc<ApiState>>,
    _path: dropshot::Path<RfdPathParams>,
) -> Result<HttpResponseOk<RfdWithRevision>, HttpError> {
    // TODO: preflight_check member read rfds
    // TODO: storage::rfds::get_rfd + storage::revisions::get_latest_revision
    todo!()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RfdPathParams {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RfdWithRevision {
    pub rfd: crate::models::Rfd,
    pub revision: crate::models::RfdRevision,
}

// ============================================================
// Thread endpoints
// ============================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ThreadPathParams {
    pub id: String,  // rfd id
    pub tid: String, // thread id
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateThreadRequest {
    pub body: String, // first message markdown body
}

/// List threads for an RFD.
#[endpoint(method = GET, path = "/api/rfds/{id}/threads", tags = ["threads"])]
pub async fn list_threads(
    _rqctx: RequestContext<Arc<ApiState>>,
    _path: dropshot::Path<RfdPathParams>,
) -> Result<HttpResponseOk<Vec<crate::models::Thread>>, HttpError> {
    // TODO: preflight_check member read threads
    // TODO: storage::threads::list_threads
    todo!()
}

/// Create a new thread on an RFD.
#[endpoint(method = POST, path = "/api/rfds/{id}/threads", tags = ["threads"])]
pub async fn create_thread(
    _rqctx: RequestContext<Arc<ApiState>>,
    _path: dropshot::Path<RfdPathParams>,
    _body: dropshot::TypedBody<CreateThreadRequest>,
) -> Result<HttpResponseOk<crate::models::Thread>, HttpError> {
    // TODO: preflight_check member write threads
    // TODO: insert thread + first message in a transaction
    // TODO: insert event thread.created
    // TODO: broadcast WsEvent::ThreadCreated
    todo!()
}

/// Resolve a thread.
#[endpoint(method = POST, path = "/api/rfds/{id}/threads/{tid}/resolve", tags = ["threads"])]
pub async fn resolve_thread(
    _rqctx: RequestContext<Arc<ApiState>>,
    _path: dropshot::Path<ThreadPathParams>,
) -> Result<HttpResponseOk<crate::models::Thread>, HttpError> {
    // TODO: preflight_check member write threads
    // TODO: storage::threads::resolve_thread
    // TODO: insert event thread.resolved
    // TODO: broadcast WsEvent::ThreadResolved
    todo!()
}
