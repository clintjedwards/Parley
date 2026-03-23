// RFD list view.
//
// Keybindings:
//   j / k        navigate rows
//   /            filter by title
//   s            cycle status filter
//   Enter        open selected RFD (switch to RfdDetail)
//   Tab          switch to Discussions for selected RFD
//   q            quit

use crate::models::Rfd;

pub struct RfdListView {
    pub rfds: Vec<Rfd>,
    pub filtered: Vec<usize>, // indices into rfds
    pub cursor: usize,
    pub query: String,
    pub status_filter: Option<String>,
}

impl RfdListView {
    pub fn new(rfds: Vec<Rfd>) -> Self {
        let filtered = (0..rfds.len()).collect();
        Self {
            rfds,
            filtered,
            cursor: 0,
            query: String::new(),
            status_filter: None,
        }
    }

    pub fn apply_filters(&mut self) {
        // TODO: filter rfds by query (title substring) and status_filter
    }

    pub fn render(&self, _frame: &mut ratatui::Frame) {
        // TODO: render table of RFDs with number, title, status badge, authors, updated
    }
}
