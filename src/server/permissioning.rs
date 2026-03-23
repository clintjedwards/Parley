// Mirrors the auth model from Gofer (gofer/src/api/permissioning.rs).
// Tokens carry roles; roles carry permissions over resources + actions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource {
    All,
    Rfds,
    Threads,
    Messages,
    Tokens,
    Roles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Delete,
}

pub struct AuthContext {
    pub token_id: String,
    pub user: String,
    pub roles: Vec<String>,
}

/// Check whether the resolved roles for a token grant the required resource + action.
/// Called by preflight_check in api.rs.
pub fn is_authorized(
    _ctx: &AuthContext,
    _resource: &Resource,
    _action: &Action,
) -> bool {
    // TODO: resolve roles → permissions from DB and check
    todo!()
}

/// System role IDs. These are created on startup and cannot be modified.
pub mod system_roles {
    pub const BOOTSTRAP: &str = "bootstrap";
    pub const ADMIN: &str = "admin";
    pub const MEMBER: &str = "member";
    pub const READER: &str = "reader";
}
