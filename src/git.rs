use anyhow::Result;
use std::path::Path;

/// Pull the latest changes from the remote.
pub async fn pull(repo_path: &Path, branch: &str) -> Result<()> {
    // TODO: spawn git pull --ff-only
    // TODO: on failure: git fetch origin && git reset --hard origin/<branch>
    todo!()
}

/// Extract RFD numbers from a list of changed file paths.
/// Matches paths like rfd/0001/rfd.typ or rfd/0001/metadata.toml.
pub fn affected_rfd_numbers(paths: &[String]) -> Vec<u32> {
    // TODO: regex match rfd/(\d{4})/ and collect unique numbers
    todo!()
}
