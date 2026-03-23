// RFD detail view — renders the compiled Typst content in the terminal.
//
// Keybindings:
//   j / k / arrows   scroll
//   c                new discussion thread on this RFD
//   Tab              switch to Discussions view
//   q                back to RFD list

use crate::models::{Rfd, RfdRevision};

pub struct RfdDetailView {
    pub rfd: Rfd,
    pub revision: RfdRevision,
    pub scroll_offset: u16,
    // Terminal-renderable lines converted from rendered_html
    pub lines: Vec<String>,
}

impl RfdDetailView {
    pub fn new(rfd: Rfd, revision: RfdRevision) -> Self {
        // TODO: convert revision.rendered_html → terminal-renderable lines
        // (strip HTML tags, preserve basic structure: headings, paragraphs, lists)
        Self {
            rfd,
            revision,
            scroll_offset: 0,
            lines: vec![],
        }
    }

    pub fn render(&self, _frame: &mut ratatui::Frame) {
        // TODO: render lines in a ratatui Paragraph widget with scroll
        // TODO: render header bar: RFD number + title + status
    }
}
