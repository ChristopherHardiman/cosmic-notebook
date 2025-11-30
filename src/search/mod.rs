//! Search module for Cosmic Notebook
//!
//! Handles search functionality including:
//! - In-document search
//! - Find and replace
//! - Regular expression support
//! - Global search across files

mod find;

pub use find::{FindOptions, FindResult, SearchEngine, SearchDirection};

/// Search state for the UI
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search query
    pub query: String,
    /// Replacement text
    pub replace_text: String,
    /// Search options
    pub options: FindOptions,
    /// Current search results
    pub results: Vec<FindResult>,
    /// Current result index (0-indexed)
    pub current_index: Option<usize>,
    /// Whether find dialog is open
    pub find_open: bool,
    /// Whether replace dialog is open  
    pub replace_open: bool,
    /// Error message if search failed
    pub error: Option<String>,
}

impl SearchState {
    /// Create a new search state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the search query and clear results
    pub fn set_query(&mut self, query: String) {
        if self.query != query {
            self.query = query;
            self.results.clear();
            self.current_index = None;
            self.error = None;
        }
    }

    /// Set the replacement text
    pub fn set_replace_text(&mut self, text: String) {
        self.replace_text = text;
    }

    /// Toggle case sensitivity
    pub fn toggle_case_sensitive(&mut self) {
        self.options.case_sensitive = !self.options.case_sensitive;
        self.results.clear();
        self.current_index = None;
    }

    /// Toggle whole word matching
    pub fn toggle_whole_word(&mut self) {
        self.options.whole_word = !self.options.whole_word;
        self.results.clear();
        self.current_index = None;
    }

    /// Toggle regex mode
    pub fn toggle_regex(&mut self) {
        self.options.use_regex = !self.options.use_regex;
        self.results.clear();
        self.current_index = None;
    }

    /// Update search results
    pub fn update_results(&mut self, results: Vec<FindResult>) {
        self.results = results;
        if !self.results.is_empty() {
            self.current_index = Some(0);
        } else {
            self.current_index = None;
        }
    }

    /// Move to next result
    pub fn next_result(&mut self) -> Option<&FindResult> {
        if self.results.is_empty() {
            return None;
        }

        let next = match self.current_index {
            Some(i) => (i + 1) % self.results.len(),
            None => 0,
        };
        self.current_index = Some(next);
        self.results.get(next)
    }

    /// Move to previous result
    pub fn prev_result(&mut self) -> Option<&FindResult> {
        if self.results.is_empty() {
            return None;
        }

        let prev = match self.current_index {
            Some(i) if i > 0 => i - 1,
            Some(_) | None => self.results.len() - 1,
        };
        self.current_index = Some(prev);
        self.results.get(prev)
    }

    /// Get current result
    pub fn current_result(&self) -> Option<&FindResult> {
        self.current_index.and_then(|i| self.results.get(i))
    }

    /// Get result count display string (e.g., "3 of 15")
    pub fn result_count_display(&self) -> String {
        if self.results.is_empty() {
            if self.query.is_empty() {
                String::new()
            } else {
                "No results".to_string()
            }
        } else {
            let current = self.current_index.map(|i| i + 1).unwrap_or(0);
            format!("{} of {}", current, self.results.len())
        }
    }

    /// Clear search state
    pub fn clear(&mut self) {
        self.query.clear();
        self.replace_text.clear();
        self.results.clear();
        self.current_index = None;
        self.error = None;
    }

    /// Open find dialog
    pub fn open_find(&mut self) {
        self.find_open = true;
        self.replace_open = false;
    }

    /// Open find and replace dialog
    pub fn open_replace(&mut self) {
        self.find_open = true;
        self.replace_open = true;
    }

    /// Close dialogs
    pub fn close(&mut self) {
        self.find_open = false;
        self.replace_open = false;
    }

    /// Check if any dialog is open
    pub fn is_open(&self) -> bool {
        self.find_open || self.replace_open
    }
}
