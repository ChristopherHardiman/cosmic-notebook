//! Per-document editor state
//!
//! Contains state for a single document's editing session including
//! cursor position, selection, scroll offset, and undo/redo history.

use crate::config::MAX_UNDO_HISTORY;
use serde::{Deserialize, Serialize};

/// Cursor position in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Line number (0-indexed)
    pub line: usize,

    /// Column/character offset within the line (0-indexed)
    pub column: usize,
}

impl CursorPosition {
    /// Create a new cursor position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create cursor at the start of the document
    pub fn start() -> Self {
        Self::default()
    }

    /// Get 1-indexed line number for display
    pub fn display_line(&self) -> usize {
        self.line + 1
    }

    /// Get 1-indexed column for display
    pub fn display_column(&self) -> usize {
        self.column + 1
    }
}

impl std::fmt::Display for CursorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ln {}, Col {}", self.display_line(), self.display_column())
    }
}

/// Text selection range
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start of selection (anchor point)
    pub start: CursorPosition,

    /// End of selection (active point, where cursor moves)
    pub end: CursorPosition,
}

impl Selection {
    /// Create a new selection
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }

    /// Create a collapsed selection (cursor with no selection)
    pub fn collapsed(position: CursorPosition) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Check if selection is collapsed (no text selected)
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    /// Get the selection in normalized order (start before end)
    pub fn normalized(&self) -> (CursorPosition, CursorPosition) {
        if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.column <= self.end.column)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Get the start position (earlier in document)
    pub fn start_position(&self) -> CursorPosition {
        self.normalized().0
    }

    /// Get the end position (later in document)
    pub fn end_position(&self) -> CursorPosition {
        self.normalized().1
    }

    /// Check if a position is within the selection
    pub fn contains(&self, pos: CursorPosition) -> bool {
        let (start, end) = self.normalized();

        if pos.line < start.line || pos.line > end.line {
            return false;
        }

        if pos.line == start.line && pos.column < start.column {
            return false;
        }

        if pos.line == end.line && pos.column > end.column {
            return false;
        }

        true
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::collapsed(CursorPosition::default())
    }
}

/// A single edit operation for undo/redo
#[derive(Debug, Clone)]
pub struct EditOperation {
    /// Type of edit
    pub kind: EditKind,

    /// Position where edit occurred
    pub position: CursorPosition,

    /// Text involved (inserted or deleted)
    pub text: String,

    /// Selection before the edit (for restoration)
    pub selection_before: Selection,

    /// Cursor position after the edit
    pub cursor_after: CursorPosition,

    /// Timestamp of the edit
    pub timestamp: std::time::Instant,
}

impl EditOperation {
    /// Create a new insert operation
    pub fn insert(
        position: CursorPosition,
        text: String,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Insert,
            position,
            text,
            selection_before,
            cursor_after,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new delete operation
    pub fn delete(
        position: CursorPosition,
        text: String,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Delete,
            position,
            text,
            selection_before,
            cursor_after,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new replace operation
    pub fn replace(
        position: CursorPosition,
        old_text: String,
        new_text: String,
        selection_before: Selection,
        cursor_after: CursorPosition,
    ) -> Self {
        Self {
            kind: EditKind::Replace {
                old_text,
                new_text: new_text.clone(),
            },
            position,
            text: new_text,
            selection_before,
            cursor_after,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if this edit can be merged with another
    /// (for grouping consecutive character insertions)
    pub fn can_merge_with(&self, other: &EditOperation) -> bool {
        // Only merge inserts
        if !matches!(self.kind, EditKind::Insert) || !matches!(other.kind, EditKind::Insert) {
            return false;
        }

        // Must be recent (within 500ms)
        if other.timestamp.duration_since(self.timestamp).as_millis() > 500 {
            return false;
        }

        // Must be adjacent
        if other.position.line != self.cursor_after.line
            || other.position.column != self.cursor_after.column
        {
            return false;
        }

        // Don't merge if it's a newline or space after non-space
        if other.text == "\n" || (other.text == " " && !self.text.ends_with(' ')) {
            return false;
        }

        true
    }
}

/// Type of edit operation
#[derive(Debug, Clone)]
pub enum EditKind {
    /// Text was inserted
    Insert,
    /// Text was deleted
    Delete,
    /// Text was replaced
    Replace { old_text: String, new_text: String },
}

/// Editor state for a single document
#[derive(Debug, Clone)]
pub struct EditorState {
    /// Current cursor position
    pub cursor: CursorPosition,

    /// Current selection
    pub selection: Selection,

    /// Scroll offset (line at top of viewport)
    pub scroll_line: usize,

    /// Horizontal scroll offset (character offset)
    pub scroll_column: usize,

    /// Undo history stack
    pub undo_stack: Vec<EditOperation>,

    /// Redo history stack
    pub redo_stack: Vec<EditOperation>,

    /// Maximum undo history size
    pub max_undo_history: usize,

    /// Preferred column for vertical cursor movement
    /// (remembers column when moving through shorter lines)
    pub preferred_column: Option<usize>,

    /// Find results for this document (character offsets)
    pub find_results: Vec<(usize, usize)>,

    /// Current find result index
    pub current_find_index: Option<usize>,

    /// Whether the editor has focus
    pub has_focus: bool,
}

impl EditorState {
    /// Create a new editor state
    pub fn new() -> Self {
        Self {
            cursor: CursorPosition::default(),
            selection: Selection::default(),
            scroll_line: 0,
            scroll_column: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_history: MAX_UNDO_HISTORY,
            preferred_column: None,
            find_results: Vec::new(),
            current_find_index: None,
            has_focus: false,
        }
    }

    /// Set cursor position and collapse selection
    pub fn set_cursor(&mut self, position: CursorPosition) {
        self.cursor = position;
        self.selection = Selection::collapsed(position);
        self.preferred_column = None;
    }

    /// Set cursor position without changing selection (extends selection)
    pub fn extend_selection_to(&mut self, position: CursorPosition) {
        self.cursor = position;
        self.selection.end = position;
        self.preferred_column = None;
    }

    /// Set the selection directly
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
        self.cursor = selection.end;
        self.preferred_column = None;
    }

    /// Select all (requires document length info)
    pub fn select_all(&mut self, end_position: CursorPosition) {
        self.selection = Selection::new(CursorPosition::start(), end_position);
        self.cursor = end_position;
    }

    /// Push an edit to the undo stack
    pub fn push_undo(&mut self, operation: EditOperation) {
        // Try to merge with previous operation
        if let Some(last) = self.undo_stack.last_mut() {
            if last.can_merge_with(&operation) {
                // Merge the operations
                if let EditKind::Insert = &last.kind {
                    last.text.push_str(&operation.text);
                    last.cursor_after = operation.cursor_after;
                    last.timestamp = operation.timestamp;
                    return;
                }
            }
        }

        // Add as new operation
        self.undo_stack.push(operation);

        // Trim if too large
        while self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.remove(0);
        }

        // Clear redo stack on new edit
        self.redo_stack.clear();
    }

    /// Pop from undo stack and push to redo
    pub fn pop_undo(&mut self) -> Option<EditOperation> {
        let op = self.undo_stack.pop()?;
        self.redo_stack.push(op.clone());
        Some(op)
    }

    /// Pop from redo stack and push to undo
    pub fn pop_redo(&mut self) -> Option<EditOperation> {
        let op = self.redo_stack.pop()?;
        self.undo_stack.push(op.clone());
        Some(op)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear undo/redo history
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Clear find results
    pub fn clear_find_results(&mut self) {
        self.find_results.clear();
        self.current_find_index = None;
    }

    /// Move to next find result
    pub fn next_find_result(&mut self) -> Option<(usize, usize)> {
        if self.find_results.is_empty() {
            return None;
        }

        let next_index = match self.current_find_index {
            Some(i) => (i + 1) % self.find_results.len(),
            None => 0,
        };

        self.current_find_index = Some(next_index);
        self.find_results.get(next_index).copied()
    }

    /// Move to previous find result
    pub fn prev_find_result(&mut self) -> Option<(usize, usize)> {
        if self.find_results.is_empty() {
            return None;
        }

        let prev_index = match self.current_find_index {
            Some(i) => {
                if i == 0 {
                    self.find_results.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.find_results.len() - 1,
        };

        self.current_find_index = Some(prev_index);
        self.find_results.get(prev_index).copied()
    }

    /// Get find result count
    pub fn find_result_count(&self) -> usize {
        self.find_results.len()
    }

    /// Get current find result number (1-indexed)
    pub fn current_find_number(&self) -> Option<usize> {
        self.current_find_index.map(|i| i + 1)
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position_display() {
        let pos = CursorPosition::new(5, 10);
        assert_eq!(pos.display_line(), 6);
        assert_eq!(pos.display_column(), 11);
        assert_eq!(format!("{}", pos), "Ln 6, Col 11");
    }

    #[test]
    fn test_selection_collapsed() {
        let pos = CursorPosition::new(0, 0);
        let sel = Selection::collapsed(pos);
        assert!(sel.is_collapsed());
    }

    #[test]
    fn test_selection_normalized() {
        // Selection backwards
        let sel = Selection::new(CursorPosition::new(5, 10), CursorPosition::new(2, 5));
        let (start, end) = sel.normalized();
        assert_eq!(start.line, 2);
        assert_eq!(end.line, 5);
    }

    #[test]
    fn test_selection_contains() {
        let sel = Selection::new(CursorPosition::new(2, 5), CursorPosition::new(5, 10));

        assert!(sel.contains(CursorPosition::new(3, 0)));
        assert!(sel.contains(CursorPosition::new(2, 5)));
        assert!(sel.contains(CursorPosition::new(5, 10)));
        assert!(!sel.contains(CursorPosition::new(1, 0)));
        assert!(!sel.contains(CursorPosition::new(6, 0)));
    }

    #[test]
    fn test_editor_state_undo_redo() {
        let mut state = EditorState::new();

        let op = EditOperation::insert(
            CursorPosition::new(0, 0),
            "test".to_string(),
            Selection::default(),
            CursorPosition::new(0, 4),
        );

        state.push_undo(op);
        assert!(state.can_undo());
        assert!(!state.can_redo());

        state.pop_undo();
        assert!(!state.can_undo());
        assert!(state.can_redo());

        state.pop_redo();
        assert!(state.can_undo());
        assert!(!state.can_redo());
    }

    #[test]
    fn test_find_navigation() {
        let mut state = EditorState::new();
        state.find_results = vec![(0, 5), (10, 15), (20, 25)];

        let result = state.next_find_result();
        assert_eq!(result, Some((0, 5)));
        assert_eq!(state.current_find_number(), Some(1));

        let result = state.next_find_result();
        assert_eq!(result, Some((10, 15)));
        assert_eq!(state.current_find_number(), Some(2));

        let result = state.prev_find_result();
        assert_eq!(result, Some((0, 5)));
        assert_eq!(state.current_find_number(), Some(1));
    }
}
