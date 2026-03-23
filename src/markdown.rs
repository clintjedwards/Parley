/// Render a markdown string to sanitized HTML.
/// Used for discussion message bodies.
pub fn render(_markdown: &str) -> String {
    // TODO: pulldown-cmark to convert markdown → HTML
    // TODO: ammonia to sanitize (allow standard text tags, blockquote for >quotes)
    todo!()
}
