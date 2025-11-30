//! Undo/Redo system for the editor
//!
//! Implements the Command pattern for reversible operations with:
//! - Operation grouping (consecutive chars within 500ms)
//! - Memory optimization with max stack depth
//! - Support for insert, delete, and replace operations

use crate::state::{CursorPosition, Selection};
use std::time::{Duration, Instant};

/// Maximum time between operations to allow grouping (500ms)
const GROUP_TIMEOUT: Duration = Duration::from_millis(500);

/// Types of edit operations
#[derive(Debug, Clone, PartialEq)]
pub enum EditKind {
    /// Text was inserted
    Insert,
    /// Text was deleted
    Delete,
    /// Text was replaced (used for paste over selection, etc.)
    Replace {
        /// The original text that was replaced
        old_text: String,
    },
}

/// A single edit operation that can be undone/redone
#[derive(Debug, Clone)]
pub struct EditOperation {
    /// Type of edit
    pub kind: EditKind,
    /// Position where edit occurred (character index)
    pub position: usize,
    /// Text involved in the operation
    pub text: String,
    /// Cursor position before the edit
    pub cursor_before: CursorPosition,
    /// Selection before the edit
    pub selection_before: Selection,
    /// Cursor position after the edit
    pub cursor_after: CursorPosition,
    /// When the operation was performed
    pub timestamp: Instant,
}

impl EditOperation {
    /// Create a new insert operation
    pub fn insert(
        position: usize,
        text: String,
        cursor_before: CursorPosition,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Insert,
            position,
            text,
            cursor_before,
            selection_before,
            cursor_after,
            timestamp: Instant::now(),
        }
    }

    /// Create a new delete operation
    pub fn delete(
        position: usize,
        text: String,
        cursor_before: CursorPosition,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Delete,
            position,
            text,
            cursor_before,
            selection_before,
            cursor_after,
            timestamp: Instant::now(),
        }
    }

    /// Create a new replace operation
    pub fn replace(
        position: usize,
        old_text: String,
        new_text: String,
        cursor_before: CursorPosition,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Replace { old_text },
            position,
            text: new_text,
            cursor_before,
            selection_before,
            cursor_after,
            timestamp: Instant::now(),
        }
    }

    /// Check if this operation can be merged with a subsequent operation
    pub fn can_merge_with(&self, other: &EditOperation) -> bool {
        // Only merge inserts with inserts, deletes with deletes
        match (&self.kind, &other.kind) {
            (EditKind::Insert, EditKind::Insert) => {
                // Must be within timeout
                if other.timestamp.duration_since(self.timestamp) > GROUP_TIMEOUT {
                    return false;
                }

                // Must be adjacent (new insert right after previous)
                if other.position != self.position + self.text.len() {
                    return false;
                }

                // Don't merge newlines or if previous ended with space and new is space
                if other.text == "\n" {
                    return false;
                }
                if other.text == " " && self.text.ends_with(' ') {
                    return false;
                }

                true
            }
            (EditKind::Delete, EditKind::Delete) => {
                // Must be within timeout
                if other.timestamp.duration_since(self.timestamp) > GROUP_TIMEOUT {
                    return false;
                }

                // For backspace: new delete is at position before current
                // For delete key: new delete is at same position
                let is_backspace = other.position + other.text.len() == self.position;
                let is_delete = other.position == self.position;

                if !is_backspace && !is_delete {
                    return false;
                }

                // Don't merge if deleting newlines
                if other.text.contains('\n') || self.text.contains('\n') {
                    return false;
                }

                true
            }
            _ => false,
        }
    }

    /// Merge another operation into this one
    pub fn merge(&mut self, other: EditOperation) {
        match (&mut self.kind, &other.kind) {
            (EditKind::Insert, EditKind::Insert) => {
                // Append text
                self.text.push_str(&other.text);
                self.cursor_after = other.cursor_after;
                self.timestamp = other.timestamp;
            }
            (EditKind::Delete, EditKind::Delete) => {
                // For backspace (other is before self)
                if other.position < self.position {
                    self.text = format!("{}{}", other.text, self.text);
                    self.position = other.position;
                } else {
                    // For delete key (same position)
                    self.text.push_str(&other.text);
                }
                self.cursor_after = other.cursor_after;
                self.timestamp = other.timestamp;
            }
            _ => {
                // Should not happen if can_merge_with is checked first
                log::warn!("Attempted to merge incompatible operations");
            }
        }
    }
}

/// Manages undo and redo stacks for a document
#[derive(Debug, Clone)]
pub struct UndoManager {
    /// Stack of operations that can be undone
    undo_stack: Vec<EditOperation>,
    /// Stack of operations that can be redone
    redo_stack: Vec<EditOperation>,
    /// Maximum number of operations to keep
    max_history: usize,
    /// Version at last save (for detecting if we're at saved state)
    saved_version: usize,
    /// Current version (increments with each operation)
    current_version: usize,
}

impl UndoManager {
    /// Create a new undo manager with specified max history
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
            saved_version: 0,
            current_version: 0,
        }
    }

    /// Create with default max history (1000 operations)
    pub fn with_default_history() -> Self {
        Self::new(1000)
    }

    /// Push an operation onto the undo stack
    pub fn push(&mut self, operation: EditOperation) {
        // Try to merge with previous operation
        if let Some(last) = self.undo_stack.last_mut() {
            if last.can_merge_with(&operation) {
                last.merge(operation);
                self.current_version += 1;
                return;
            }
        }

        // Add as new operation
        self.undo_stack.push(operation);
        self.current_version += 1;

        // Trim if too large
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        // Clear redo stack on new edit
        self.redo_stack.clear();
    }

    /// Undo the last operation, returns the operation to reverse
    pub fn undo(&mut self) -> Option<EditOperation> {
        let operation = self.undo_stack.pop()?;
        self.redo_stack.push(operation.clone());
        self.current_version += 1;
        Some(operation)
    }

    /// Redo the last undone operation, returns the operation to apply
    pub fn redo(&mut self) -> Option<EditOperation> {
        let operation = self.redo_stack.pop()?;
        self.undo_stack.push(operation.clone());
        self.current_version += 1;
        Some(operation)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_version = 0;
        self.saved_version = 0;
    }

    /// Mark current state as saved
    pub fn mark_saved(&mut self) {
        self.saved_version = self.current_version;
    }

    /// Check if we're at the saved state
    pub fn is_at_saved_state(&self) -> bool {
        self.saved_version == self.current_version
    }

    /// Get undo stack size (for display/debugging)
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size (for display/debugging)
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let undo_size: usize = self.undo_stack.iter().map(|op| op.text.len() + 128).sum();
        let redo_size: usize = self.redo_stack.iter().map(|op| op.text.len() + 128).sum();
        undo_size + redo_size
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::with_default_history()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_insert(pos: usize, text: &str) -> EditOperation {
        EditOperation::insert(
            pos,
            text.to_string(),
            CursorPosition::new(0, pos),
            Selection::collapsed(CursorPosition::new(0, pos)),
            CursorPosition::new(0, pos + text.len()),
        )
    }

    fn make_delete(pos: usize, text: &str) -> EditOperation {
        EditOperation::delete(
            pos,
            text.to_string(),
            CursorPosition::new(0, pos + text.len()),
            Selection::collapsed(CursorPosition::new(0, pos + text.len())),
            CursorPosition::new(0, pos),
        )
    }

    #[test]
    fn test_undo_redo_basic() {
        let mut manager = UndoManager::with_default_history();

        let op = make_insert(0, "Hello");
        manager.push(op);

        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        let undone = manager.undo();
        assert!(undone.is_some());
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        let redone = manager.redo();
        assert!(redone.is_some());
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_operation_merging() {
        let mut manager = UndoManager::with_default_history();

        // These should merge (consecutive inserts)
        let op1 = make_insert(0, "H");
        let op2 = make_insert(1, "e");
        let op3 = make_insert(2, "l");

        manager.push(op1);
        manager.push(op2);
        manager.push(op3);

        // Should be merged into one operation
        assert_eq!(manager.undo_count(), 1);

        let undone = manager.undo().unwrap();
        assert_eq!(undone.text, "Hel");
    }

    #[test]
    fn test_newline_breaks_merge() {
        let mut manager = UndoManager::with_default_history();

        let op1 = make_insert(0, "a");
        let op2 = make_insert(1, "\n");
        let op3 = make_insert(2, "b");

        manager.push(op1);
        manager.push(op2);
        manager.push(op3);

        // Newline should break the merge
        assert_eq!(manager.undo_count(), 3);
    }

    #[test]
    fn test_saved_state_tracking() {
        let mut manager = UndoManager::with_default_history();

        assert!(manager.is_at_saved_state());

        manager.push(make_insert(0, "test"));
        assert!(!manager.is_at_saved_state());

        manager.mark_saved();
        assert!(manager.is_at_saved_state());

        manager.undo();
        assert!(!manager.is_at_saved_state());

        manager.redo();
        assert!(manager.is_at_saved_state());
    }

    #[test]
    fn test_redo_cleared_on_new_edit() {
        let mut manager = UndoManager::with_default_history();

        manager.push(make_insert(0, "first"));
        manager.undo();
        assert!(manager.can_redo());

        manager.push(make_insert(0, "second"));
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_max_history() {
        let mut manager = UndoManager::new(3);

        for i in 0..5 {
            // Use newlines to prevent merging
            manager.push(make_insert(i * 2, "\n"));
            manager.push(make_insert(i * 2 + 1, "x"));
        }

        // Should be limited to max_history
        assert!(manager.undo_count() <= 3);
    }
}
