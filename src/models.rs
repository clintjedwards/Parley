use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================
// Tokens & Roles
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Token {
    pub id: String,
    #[serde(skip_serializing)]
    pub hash: String,
    pub created: String,
    pub expires: String,
    pub disabled: bool,
    pub user: String,
    pub roles: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Role {
    pub id: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub system_role: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Permission {
    pub resources: Vec<String>,
    pub actions: Vec<String>,
}

// ============================================================
// RFDs
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Rfd {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub status: RfdStatus,
    pub authors: Vec<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RfdStatus {
    Draft,
    Discussion,
    Accepted,
    Rejected,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RfdRevision {
    pub id: String,
    pub rfd_id: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub rendered_html: String,
    pub title: String,
    pub status: RfdStatus,
    pub authors: Vec<String>,
    pub created: String,
}

// ============================================================
// Discussions
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Thread {
    pub id: String,
    pub rfd_id: String,
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub created_by: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub author: String,
    pub body: String,
    pub body_html: String,
    pub created: String,
    pub updated: Option<String>,
}

// ============================================================
// Events
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event {
    pub id: String,
    pub kind: String,
    pub actor: Option<String>,
    pub rfd_id: Option<String>,
    pub thread_id: Option<String>,
    pub payload: serde_json::Value,
    pub created: String,
}

// ============================================================
// WebSocket messages (server → TUI client)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsEvent {
    MessageCreated { thread_id: String, message: Message },
    ThreadCreated { rfd_id: String, thread: Thread },
    ThreadResolved { thread_id: String },
    RfdUpdated { rfd_id: String, revision_id: String },
}
