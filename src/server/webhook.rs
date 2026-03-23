// GitHub webhook handler.
// Receives push events, verifies HMAC signature, triggers git sync.

use anyhow::Result;

pub struct PushPayload {
    pub head_commit_sha: String,
    pub head_commit_message: String,
    pub changed_paths: Vec<String>,
}

/// Verify the X-Hub-Signature-256 header against the configured webhook secret.
pub fn verify_signature(_payload: &[u8], _signature: &str, _secret: &str) -> bool {
    // TODO: HMAC-SHA256 the payload with the secret
    // TODO: hex encode and compare to signature header (constant-time compare)
    todo!()
}

/// Parse a GitHub push event payload and return affected RFD numbers.
pub fn affected_rfd_numbers(payload: &PushPayload) -> Vec<u32> {
    // TODO: scan changed_paths for pattern rfd/(\d{4})/
    // TODO: deduplicate and return
    todo!()
}

/// Pull the repo and compile any changed RFDs. Called after signature is verified.
pub async fn sync_repo(_rfd_numbers: Vec<u32>) -> Result<()> {
    // TODO: git pull --ff-only (fallback: git fetch && git reset --hard origin/<branch>)
    // TODO: for each rfd_number:
    //   - read + parse metadata.toml
    //   - upsert rfds row
    //   - crate::typst::compile + post_process
    //   - insert rfd_revisions row
    //   - insert event rfd.revision_added or rfd.created
    //   - broadcast WsEvent::RfdUpdated
    todo!()
}
