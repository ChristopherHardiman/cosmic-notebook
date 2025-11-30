//! Tab bar state management
//!
//! Manages the state of the tab bar including tab ordering,
//! active tab, and drag operations.

use super::DocumentId;
use serde::{Deserialize, Serialize};

/// A single tab in the tab bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// Document ID this tab represents
    pub document_id: DocumentId,

    /// Display title for the tab
    pub title: String,

    /// Whether this tab is pinned
    pub pinned: bool,
}

impl Tab {
    /// Create a new tab
    pub fn new(document_id: DocumentId, title: String) -> Self {
        Self {
            document_id,
            title,
            pinned: false,
        }
    }
}

/// State of the tab bar
#[derive(Debug, Clone, Default)]
pub struct TabState {
    /// Ordered list of tabs
    pub tabs: Vec<Tab>,

    /// Index of the currently active tab
    pub active_index: Option<usize>,

    /// Tab being dragged (if any)
    pub dragging_index: Option<usize>,

    /// Drop target index during drag
    pub drop_target_index: Option<usize>,

    /// Whether tab context menu is open
    pub context_menu_open: bool,

    /// Index of tab with context menu open
    pub context_menu_index: Option<usize>,
}

impl TabState {
    /// Create a new empty tab state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new tab at the end (or after active tab based on preference)
    pub fn add_tab(&mut self, document_id: DocumentId, title: String) {
        let tab = Tab::new(document_id, title);

        // Insert after active tab if there is one, otherwise at end
        let insert_index = self
            .active_index
            .map(|i| i + 1)
            .unwrap_or(self.tabs.len());

        self.tabs.insert(insert_index, tab);
        self.active_index = Some(insert_index);
    }

    /// Remove a tab by document ID
    pub fn remove_tab(&mut self, document_id: DocumentId) {
        if let Some(index) = self.find_tab_index(document_id) {
            self.tabs.remove(index);

            // Update active index
            if self.tabs.is_empty() {
                self.active_index = None;
            } else if let Some(active) = self.active_index {
                if active == index {
                    // Removed active tab, select previous or next
                    self.active_index = Some(active.min(self.tabs.len() - 1));
                } else if active > index {
                    // Shift active index down
                    self.active_index = Some(active - 1);
                }
            }
        }
    }

    /// Set the active tab by document ID
    pub fn set_active(&mut self, document_id: DocumentId) {
        if let Some(index) = self.find_tab_index(document_id) {
            self.active_index = Some(index);
        }
    }

    /// Set the active tab by index
    pub fn set_active_index(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = Some(index);
        }
    }

    /// Get the active tab's document ID
    pub fn active_tab(&self) -> Option<DocumentId> {
        self.active_index
            .and_then(|i| self.tabs.get(i))
            .map(|tab| tab.document_id)
    }

    /// Get the active tab reference
    pub fn active_tab_ref(&self) -> Option<&Tab> {
        self.active_index.and_then(|i| self.tabs.get(i))
    }

    /// Find tab index by document ID
    pub fn find_tab_index(&self, document_id: DocumentId) -> Option<usize> {
        self.tabs.iter().position(|t| t.document_id == document_id)
    }

    /// Move to the next tab
    pub fn next_tab(&mut self) {
        if let Some(active) = self.active_index {
            if !self.tabs.is_empty() {
                self.active_index = Some((active + 1) % self.tabs.len());
            }
        } else if !self.tabs.is_empty() {
            self.active_index = Some(0);
        }
    }

    /// Move to the previous tab
    pub fn prev_tab(&mut self) {
        if let Some(active) = self.active_index {
            if !self.tabs.is_empty() {
                self.active_index = Some(if active == 0 {
                    self.tabs.len() - 1
                } else {
                    active - 1
                });
            }
        } else if !self.tabs.is_empty() {
            self.active_index = Some(self.tabs.len() - 1);
        }
    }

    /// Move tab from one index to another (for drag reordering)
    pub fn move_tab(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to {
            return;
        }

        let tab = self.tabs.remove(from);

        // Adjust target index if needed
        let to_index = if to > from { to - 1 } else { to };
        self.tabs.insert(to_index.min(self.tabs.len()), tab);

        // Update active index
        if let Some(active) = self.active_index {
            self.active_index = Some(if active == from {
                to_index
            } else if from < active && to_index >= active {
                active - 1
            } else if from > active && to_index <= active {
                active + 1
            } else {
                active
            });
        }
    }

    /// Update a tab's title
    pub fn update_title(&mut self, document_id: DocumentId, title: String) {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|t| t.document_id == document_id)
        {
            tab.title = title;
        }
    }

    /// Toggle pin status for a tab
    pub fn toggle_pin(&mut self, document_id: DocumentId) {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|t| t.document_id == document_id)
        {
            tab.pinned = !tab.pinned;
        }
    }

    /// Get tab count
    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if there are any tabs
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Start dragging a tab
    pub fn start_drag(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.dragging_index = Some(index);
        }
    }

    /// Update drop target during drag
    pub fn update_drop_target(&mut self, index: Option<usize>) {
        self.drop_target_index = index;
    }

    /// End drag operation
    pub fn end_drag(&mut self) {
        if let (Some(from), Some(to)) = (self.dragging_index, self.drop_target_index) {
            self.move_tab(from, to);
        }
        self.dragging_index = None;
        self.drop_target_index = None;
    }

    /// Cancel drag operation
    pub fn cancel_drag(&mut self) {
        self.dragging_index = None;
        self.drop_target_index = None;
    }

    /// Close all tabs except the specified one
    pub fn close_others(&mut self, keep_document_id: DocumentId) {
        self.tabs.retain(|t| t.document_id == keep_document_id);
        self.active_index = if self.tabs.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Close all tabs to the right of the specified one
    pub fn close_to_right(&mut self, document_id: DocumentId) {
        if let Some(index) = self.find_tab_index(document_id) {
            self.tabs.truncate(index + 1);
            if let Some(active) = self.active_index {
                if active > index {
                    self.active_index = Some(index);
                }
            }
        }
    }

    /// Get iterator over tabs
    pub fn iter(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    /// Get all document IDs in order
    pub fn document_ids(&self) -> Vec<DocumentId> {
        self.tabs.iter().map(|t| t.document_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_id() -> DocumentId {
        DocumentId::new()
    }

    #[test]
    fn test_add_tab() {
        let mut state = TabState::new();
        let id = create_test_id();

        state.add_tab(id, "test.md".to_string());

        assert_eq!(state.count(), 1);
        assert_eq!(state.active_index, Some(0));
        assert_eq!(state.active_tab(), Some(id));
    }

    #[test]
    fn test_remove_tab() {
        let mut state = TabState::new();
        let id1 = create_test_id();
        let id2 = create_test_id();

        state.add_tab(id1, "test1.md".to_string());
        state.add_tab(id2, "test2.md".to_string());

        assert_eq!(state.count(), 2);

        state.remove_tab(id2);
        assert_eq!(state.count(), 1);
        assert_eq!(state.active_tab(), Some(id1));
    }

    #[test]
    fn test_next_prev_tab() {
        let mut state = TabState::new();
        let id1 = create_test_id();
        let id2 = create_test_id();
        let id3 = create_test_id();

        state.add_tab(id1, "1.md".to_string());
        state.set_active_index(0);
        state.add_tab(id2, "2.md".to_string());
        state.set_active_index(1);
        state.add_tab(id3, "3.md".to_string());
        state.set_active_index(0);

        state.next_tab();
        assert_eq!(state.active_index, Some(1));

        state.next_tab();
        assert_eq!(state.active_index, Some(2));

        state.next_tab();
        assert_eq!(state.active_index, Some(0)); // Wraps around

        state.prev_tab();
        assert_eq!(state.active_index, Some(2)); // Wraps around
    }

    #[test]
    fn test_move_tab() {
        let mut state = TabState::new();
        let id1 = create_test_id();
        let id2 = create_test_id();
        let id3 = create_test_id();

        state.tabs.push(Tab::new(id1, "1.md".to_string()));
        state.tabs.push(Tab::new(id2, "2.md".to_string()));
        state.tabs.push(Tab::new(id3, "3.md".to_string()));
        state.active_index = Some(0);

        state.move_tab(0, 2);

        assert_eq!(state.tabs[0].document_id, id2);
        assert_eq!(state.tabs[1].document_id, id1);
        assert_eq!(state.tabs[2].document_id, id3);
    }
}
