mod users;

use crate::{conf, storage};
use dropshot::{
    ApiDescription, Body, ClientErrorStatusCode, ConfigDropshot, DropshotState, EndpointTagPolicy,
    ErrorStatusCode, HandlerError, HandlerTaskMode, HttpError, HttpServer, RequestInfo,
    ServerBuilder, ServerContext, TagConfig, TagDetails, WebsocketConnectionRaw,
};
use futures::Future;
use lazy_regex::regex;
use rootcause::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use std::{pin::Pin, str::FromStr, sync::Arc};
use tokio::signal;
use tokio_tungstenite::{tungstenite, WebSocketStream};
use tracing::{error, info, warn};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};

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
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "v0" => Ok(ApiVersion::V0),
            _ => Err(report!("Invalid API version")),
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
    pub config: conf::api::ApiConfig,
    pub storage: storage::Db,
}

impl ApiState {
    fn new(config: conf::api::ApiConfig, storage: storage::Db) -> Self {
        Self { config, storage }
    }

    pub async fn preflight_check(
        &self,
        _request: &RequestInfo,
        options: PreflightOptions,
    ) -> Result<(), HttpError> {
        if options.bypass_auth || self.config.development.bypass_auth {
            return Ok(());
        }

        Err(HttpError::for_client_error(
            None,
            ClientErrorStatusCode::UNAUTHORIZED,
            "Route requires authorization".into(),
        ))
    }
}

fn init_logger(log_level: &str, pretty: bool) -> Result<(), Report> {
    let level =
        LevelFilter::from_str(log_level).context("could not parse 'log_level' configuration")?;

    let filter = EnvFilter::from_default_env()
        .add_directive("sqlx=off".parse().expect("Invalid directive"))
        .add_directive("h2=off".parse().expect("Invalid directive"))
        .add_directive("hyper=off".parse().expect("Invalid directive"))
        .add_directive("rustls=off".parse().expect("Invalid directive"))
        .add_directive("reqwest=off".parse().expect("Invalid directive"))
        .add_directive("dropshot=off".parse().expect("Invalid directive"))
        .add_directive(level.into());

    if pretty {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .compact()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .json()
            .init();
    }

    if pretty {
        warn!("pretty logging activated due to config value 'development.pretty_logging'");
    }

    Ok(())
}

fn init_api_description() -> Result<ApiDescription<Arc<ApiState>>, Report> {
    let mut api = ApiDescription::new();
    api = set_tagging_policy(api);
    register_routes(&mut api);
    Ok(api)
}

async fn init_api(conf: conf::api::ApiConfig) -> Result<Arc<ApiState>, Report> {
    let storage = storage::Db::new(&conf.server.storage_path)
        .await
        .context("Could not initialize storage")?;

    let api_state = ApiState::new(conf.clone(), storage);

    Ok(Arc::new(api_state))
}

pub async fn start_web_services() -> Result<(), Report> {
    let conf = conf::Configuration::<conf::api::ApiConfig>::load(None)
        .context("Could not initialize configuration")?;

    init_logger(&conf.general.log_level, conf.development.pretty_logging)?;

    let api_state = init_api(conf.clone())
        .await
        .context("Could not initialize API")?;

    start_web_service(conf, api_state.clone()).await?;

    Ok(())
}

pub async fn start_web_service(
    conf: conf::api::ApiConfig,
    api_state: Arc<ApiState>,
) -> Result<(), Report> {
    if conf.development.bypass_auth {
        warn!("Bypass auth activated due to config value 'development.bypass_auth'");
    }

    let bind_address = std::net::SocketAddr::from_str(&conf.server.bind_address.clone())
        .context_with(|| {
        format!(
            "Could not parse url '{}' while trying to bind binary to port; \
    should be in format '<ip>:<port>'; Please be sure to use an ip instead of something like 'localhost', \
    when attempting to bind",
            &conf.server.bind_address.clone()
        )
    })?;

    let dropshot_conf = ConfigDropshot {
        bind_address,
        default_request_body_max_bytes: 524288000,
        default_handler_task_mode: HandlerTaskMode::Detached,
    };

    let api = init_api_description()?;

    let server = ServerBuilder::new(api, api_state.clone(), Some(Arc::new(Middleware)))
        .config(dropshot_conf)
        .start()
        .map_err(|error| report!("failed to create server: {}", error))?;

    let shutdown = server.wait_for_shutdown();

    tokio::spawn(wait_for_shutdown_signal(server));

    info!(
        message = "Started http service",
        host = %bind_address.ip(),
        port = %bind_address.port(),
    );

    shutdown
        .await
        .map_err(|error| report!("Server encountered errors while running; {:#?}", error))
}

#[allow(dead_code)]
pub fn write_openapi_spec(path: PathBuf) -> Result<(), Report> {
    let api = init_api_description()?;
    let mut file = std::fs::File::create(path)?;
    api.openapi("Storage", semver::Version::from_str(BUILD_SEMVER).unwrap())
        .write(&mut file)?;

    Ok(())
}

async fn wait_for_shutdown_signal(server: HttpServer<Arc<ApiState>>) {
    listen_for_terminate_signal().await;
    server.close().await.unwrap()
}

async fn listen_for_terminate_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn set_tagging_policy(api: ApiDescription<Arc<ApiState>>) -> ApiDescription<Arc<ApiState>> {
    api.tag_config(TagConfig {
        allow_other_tags: false,
        policy: EndpointTagPolicy::ExactlyOne,
        tags: vec![
            (
                "Locations".to_string(),
                TagDetails {
                    description: Some("Locations".into()),
                    ..Default::default()
                },
            ),
            (
                "Items".to_string(),
                TagDetails {
                    description: Some("Items".into()),
                    ..Default::default()
                },
            ),
            (
                "Users".to_string(),
                TagDetails {
                    description: Some("Users".into()),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect(),
    })
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.as_millis();
    let micros = duration.as_micros();

    if secs > 0 {
        format!("{}s", secs)
    } else if millis > 0 {
        format!("{}ms", millis)
    } else if micros > 0 {
        format!("{}μs", micros)
    } else {
        format!("{}ns", duration.as_nanos())
    }
}

pub fn epoch_milli() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Debug)]
struct Middleware;

#[async_trait::async_trait]
impl<C: ServerContext> dropshot::Middleware<C> for Middleware {
    async fn handle(
        &self,
        server: Arc<DropshotState<C>>,
        request: hyper::Request<hyper::body::Incoming>,
        request_id: String,
        remote_addr: std::net::SocketAddr,
        next: fn(
            Arc<DropshotState<C>>,
            hyper::Request<hyper::body::Incoming>,
            String,
            std::net::SocketAddr,
        ) -> Pin<
            Box<dyn Future<Output = Result<hyper::Response<Body>, HandlerError>> + Send>,
        >,
    ) -> Result<hyper::Response<Body>, HandlerError> {
        let start_time = std::time::Instant::now();

        let method = request.method().as_str().to_string();
        let uri = request.uri().to_string();

        let remote_ip = match request.headers().get("X-Forwarded-For") {
            Some(value) => value
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| remote_addr.to_string()),
            None => remote_addr.to_string(),
        };

        let response = next(server.clone(), request, request_id.clone(), remote_addr).await;

        if let Ok(response) = &response {
            info!(
                remote_addr = remote_ip,
                req_id = request_id,
                method = method,
                uri = uri,
                response_code = response.status().as_str(),
                latency = format_duration(start_time.elapsed()),
                "request completed"
            );
        }

        response
    }
}

pub fn _http_error(
    message: String,
    code: hyper::StatusCode,
    request_id: String,
    context: HashMap<String, String>,
    err: Option<Box<dyn std::error::Error>>,
) -> HttpError {
    if let Some(ref e) = err {
        error!(message = message, request_id, error = %e, context = ?context);
    } else {
        error!(message = message, request_id, context = ?context);
    }

    HttpError {
        status_code: ErrorStatusCode::from_status(code).unwrap(),
        error_code: None,
        external_message: format!("{}: {}", code.canonical_reason().unwrap(), message),
        internal_message: message,
        headers: None,
    }
}

#[macro_export]
macro_rules! http_error {
    ($message:expr, $code:expr, $req_id:expr, $error:expr $(, $key:ident = $value:expr)*) => {{
        #[allow(unused_mut)]
        let mut context = std::collections::HashMap::new();
        $(
            context.insert(stringify!($key).to_string(), $value.to_string());
        )*

        $crate::api::_http_error(
            $message.to_string(),
            $code,
            $req_id,
            context,
            $error
        )
    }};
}

// Function to truncate a string to fit within a specified byte limit
fn truncate_to_utf8_bytes(s: &str, max_bytes: usize) -> String {
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Registers the handlers into the API harness. Can panic.
///
/// It's better to use unwrap here for two reasons. The first is that we fail fast and early when a handler is incorrect
/// in some way. The second is that since the underlying error returned by the register function is simply a string
/// it can be hard to know which route caused said error without unwrapping it on the spot.
fn register_routes(api: &mut ApiDescription<Arc<ApiState>>) {}

async fn websocket_error(
    message: &str,
    code: CloseCode,
    request_id: String,
    mut conn: WebSocketStream<WebsocketConnectionRaw>,
    err: Option<String>,
) -> String {
    if let Some(ref e) = err {
        error!(message = message, request_id, error = %e);
    }

    let _ = conn
        .close(Some(CloseFrame {
            code,
            reason: truncate_to_utf8_bytes(message, 123).into(), // Control frames can only be 125 bytes long (-2 for code)
        }))
        .await;

    message.to_string()
}

/// Generic identifier validation function.
///
/// This function is meant to validate user defined identifiers that may be used as primary keys
/// and therefore should have some sane bounds.
///
/// For all ids we'll want the following:
/// * 32 > characters < 3
/// * Only alphanumeric characters or hyphens
///
/// We don't allow underscores to conform with common practices for url safe strings.
pub fn is_valid_identifier(id: &str) -> Result<(), Report> {
    let alphanumeric_w_hyphen = regex!("^[a-zA-Z0-9-]*$");

    if id.len() > 32 {
        bail!("length cannot be greater than 32");
    }

    if id.len() < 3 {
        bail!("length cannot be less than 3");
    }

    if !alphanumeric_w_hyphen.is_match(id) {
        bail!("can only be made up of alphanumeric and hyphen characters");
    }

    Ok(())
}
