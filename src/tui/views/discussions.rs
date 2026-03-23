// Discussions view — flat thread list for the current RFD.
//
// Keybindings:
//   j / k        navigate threads
//   Enter        expand/collapse thread messages
//   r            reply (opens $EDITOR or inline compose)
//   R            resolve / unresolve thread
//   n            new thread
//   q            back

use crate::models::{Message, Thread};

pub struct DiscussionsView {
    pub rfd_id: String,
    pub threads: Vec<Thread>,
    pub messages: std::collections::HashMap<String, Vec<Message>>,
    pub cursor: usize,
    pub expanded: std::collections::HashSet<String>,
}

impl DiscussionsView {
    pub fn new(rfd_id: String) -> Self {
        Self {
            rfd_id,
            threads: vec![],
            messages: std::collections::HashMap::new(),
            cursor: 0,
            expanded: std::collections::HashSet::new(),
        }
    }

    pub fn render(&self, _frame: &mut ratatui::Frame) {
        // TODO: render thread list
        // TODO: for expanded threads, render flat message list below
        // TODO: show resolved threads dimmed with [resolved] label
    }
}
