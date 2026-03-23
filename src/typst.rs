use anyhow::Result;
use std::path::Path;

/// Compile a .typ file to HTML using the typst CLI.
/// Returns post-processed HTML ready to store in rfd_revisions.rendered_html.
pub async fn compile(typ_path: &Path, repo_root: &Path, binary: &str) -> Result<String> {
    // TODO: tokio::process::Command::new(binary)
    //   .args(["compile", "--format", "html", "--root", repo_root, typ_path, "-"])
    //   .output()
    //
    // TODO: call post_process on stdout HTML
    todo!()
}

/// Post-process raw typst HTML output:
///   1. Extract <body> inner content
///   2. Walk block elements and inject data-pindex="N"
///   3. Extract <style>, scope rules to #rfd-body, re-inject inline
///   4. Sanitize with ammonia
pub fn post_process(_raw_html: &str) -> Result<String> {
    // TODO: use scraper to parse and mutate
    // TODO: use ammonia to sanitize
    todo!()
}

/// Convert post-processed HTML to terminal-renderable plain text lines.
/// Used by the TUI to display RFD content.
pub fn to_terminal_lines(_html: &str) -> Vec<String> {
    // TODO: strip tags, preserve headings (##), lists (-), paragraphs
    todo!()
}
