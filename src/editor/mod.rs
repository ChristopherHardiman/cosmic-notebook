//! Editor module for Cosmic Notebook
//!
//! Contains the core text editing functionality including:
//! - Text buffer management (using ropey)
//! - Cursor and selection handling
//! - Text input processing
//! - Undo/redo operations
//! - Clipboard operations
//! - Line operations (indent, comment, etc.)

pub mod buffer;
pub mod clipboard;
pub mod cursor;
pub mod undo;
pub mod widget;

pub use buffer::TextBuffer;
pub use clipboard::{clipboard, copy_text, paste_text, ClipboardError, ClipboardManager};
pub use cursor::CursorController;
pub use undo::{EditKind, EditOperation, UndoManager};
pub use widget::EditorWidget;

use crate::state::{CursorPosition, EditorState, Selection};

/// Main editor component managing a text buffer and state
pub struct Editor {
    /// The text buffer
    buffer: TextBuffer,
    /// Editor state (cursor, selection, undo/redo)
    state: EditorState,
    /// Preferred column for vertical movement
    preferred_col: Option<usize>,
    /// Viewport height in lines
    viewport_lines: usize,
    /// Current scroll position (first visible line)
    scroll_line: usize,
    /// Scroll margin (lines to keep visible above/below cursor)
    scroll_margin: usize,
}

impl Editor {
    /// Create a new empty editor
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            state: EditorState::new(),
            preferred_col: None,
            viewport_lines: 30,
            scroll_line: 0,
            scroll_margin: 3,
        }
    }

    /// Create editor with initial content
    pub fn with_content(content: &str) -> Self {
        Self {
            buffer: TextBuffer::from_str(content),
            state: EditorState::new(),
            preferred_col: None,
            viewport_lines: 30,
            scroll_line: 0,
            scroll_margin: 3,
        }
    }

    /// Get the text buffer reference
    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    /// Get mutable text buffer reference
    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    /// Get the editor state
    pub fn state(&self) -> &EditorState {
        &self.state
    }

    /// Get mutable editor state
    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    /// Get cursor position
    pub fn cursor(&self) -> CursorPosition {
        self.state.cursor
    }

    /// Set cursor position and update scroll
    pub fn set_cursor(&mut self, pos: CursorPosition) {
        let clamped = CursorController::clamp(&self.buffer, pos);
        self.state.cursor = clamped;
        self.state.selection = Selection::collapsed(clamped);
        self.update_scroll();
        // Clear preferred column on explicit position set
        self.preferred_col = None;
    }

    /// Get current selection (None if collapsed)
    pub fn selection(&self) -> Option<&Selection> {
        if self.state.selection.is_collapsed() {
            None
        } else {
            Some(&self.state.selection)
        }
    }

    /// Check if there's an active selection (non-collapsed)
    pub fn has_selection(&self) -> bool {
        !self.state.selection.is_collapsed()
    }

    /// Set selection
    pub fn set_selection(&mut self, selection: Selection) {
        self.state.selection = selection;
    }

    /// Get current scroll line
    pub fn scroll_line(&self) -> usize {
        self.scroll_line
    }

    /// Set viewport size
    pub fn set_viewport_lines(&mut self, lines: usize) {
        self.viewport_lines = lines;
        self.update_scroll();
    }

    /// Update scroll to keep cursor visible
    fn update_scroll(&mut self) {
        self.scroll_line = cursor::calculate_scroll(
            self.state.cursor.line,
            self.scroll_line,
            self.viewport_lines,
            self.scroll_margin,
        );
    }

    /// Check if buffer is modified
    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    /// Mark buffer as saved
    pub fn mark_saved(&mut self) {
        self.buffer.mark_saved();
    }

    /// Get full content as string
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// Set content (replaces everything)
    pub fn set_content(&mut self, content: &str) {
        self.buffer = TextBuffer::from_str(content);
        self.state = EditorState::new();
        self.preferred_col = None;
        self.scroll_line = 0;
    }

    // === Movement Operations ===

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_left(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_right(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor up
    pub fn move_up(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let (new_pos, pref) =
            CursorController::move_up(&self.buffer, self.state.cursor, self.preferred_col);
        self.state.cursor = new_pos;
        self.preferred_col = pref;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor down
    pub fn move_down(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let (new_pos, pref) =
            CursorController::move_down(&self.buffer, self.state.cursor, self.preferred_col);
        self.state.cursor = new_pos;
        self.preferred_col = pref;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to line start (smart home)
    pub fn move_home(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_home(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to line end
    pub fn move_end(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_end(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to previous word
    pub fn move_word_left(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_word_left(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to next word
    pub fn move_word_right(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let new_pos = CursorController::move_word_right(&self.buffer, self.state.cursor);
        self.state.cursor = new_pos;
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor up by page
    pub fn page_up(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let (new_pos, pref) = CursorController::move_page_up(
            &self.buffer,
            self.state.cursor,
            self.viewport_lines,
            self.preferred_col,
        );
        self.state.cursor = new_pos;
        self.preferred_col = pref;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor down by page
    pub fn page_down(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        let (new_pos, pref) = CursorController::move_page_down(
            &self.buffer,
            self.state.cursor,
            self.viewport_lines,
            self.preferred_col,
        );
        self.state.cursor = new_pos;
        self.preferred_col = pref;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to document start
    pub fn move_document_start(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        self.state.cursor = CursorController::move_document_start();
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Move cursor to document end
    pub fn move_document_end(&mut self, extend_selection: bool) {
        self.update_selection_start(extend_selection);
        self.state.cursor = CursorController::move_document_end(&self.buffer);
        self.preferred_col = None;
        self.update_selection_end(extend_selection);
        self.update_scroll();
    }

    /// Go to specific line number (1-indexed)
    pub fn go_to_line(&mut self, line_number: usize) {
        self.clear_selection();
        self.state.cursor = CursorController::go_to_line(&self.buffer, line_number);
        self.preferred_col = None;
        self.update_scroll();
    }

    // === Selection Helpers ===

    fn update_selection_start(&mut self, extend: bool) {
        if extend {
            // If selection is collapsed, start a new selection from current cursor
            if self.state.selection.is_collapsed() {
                self.state.selection = Selection::new(self.state.cursor, self.state.cursor);
            }
            // Otherwise keep the existing anchor (start)
        } else {
            // Not extending - will collapse selection after movement
        }
    }

    fn update_selection_end(&mut self, extend: bool) {
        if extend {
            // Update the end of selection to new cursor position
            self.state.selection.end = self.state.cursor;
        } else {
            // Collapse selection to cursor position
            self.state.selection = Selection::collapsed(self.state.cursor);
        }
    }

    /// Clear current selection
    pub fn clear_selection(&mut self) {
        self.state.selection = Selection::collapsed(self.state.cursor);
    }

    /// Select all text
    pub fn select_all(&mut self) {
        let start = CursorPosition::new(0, 0);
        let end = CursorController::move_document_end(&self.buffer);
        self.state.selection = Selection::new(start, end);
        self.state.cursor = end;
    }

    // === Edit Operations ===

    /// Insert a character at cursor position
    pub fn insert_char(&mut self, ch: char) {
        // Delete selection first if present
        self.delete_selection();

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(self.buffer.len_chars());

        self.buffer.insert_char(char_idx, ch);

        // Move cursor after inserted char
        if ch == '\n' {
            self.state.cursor = CursorPosition::new(self.state.cursor.line + 1, 0);
        } else {
            self.state.cursor.column += 1;
        }
        self.state.selection = Selection::collapsed(self.state.cursor);
        self.preferred_col = None;
        self.update_scroll();
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        self.delete_selection();

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(self.buffer.len_chars());

        self.buffer.insert_str(char_idx, text);

        // Calculate new cursor position
        let newlines = text.matches('\n').count();
        if newlines > 0 {
            let after_last_newline = text.rfind('\n').map(|i| &text[i + 1..]).unwrap_or("");
            self.state.cursor = CursorPosition::new(
                self.state.cursor.line + newlines,
                after_last_newline.chars().count(),
            );
        } else {
            self.state.cursor.column += text.chars().count();
        }
        self.state.selection = Selection::collapsed(self.state.cursor);
        self.preferred_col = None;
        self.update_scroll();
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.state.cursor.line == 0 && self.state.cursor.column == 0 {
            return;
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(0);

        if char_idx > 0 {
            // Get character being deleted to know how to move cursor
            let del_char = self.buffer.char_at(char_idx - 1);
            self.buffer.delete_range(char_idx - 1, char_idx);

            // Move cursor back
            if del_char == Some('\n') {
                let prev_line = self.state.cursor.line - 1;
                let prev_col = self.buffer.line_len(prev_line).unwrap_or(0);
                self.state.cursor = CursorPosition::new(prev_line, prev_col);
            } else {
                self.state.cursor.column = self.state.cursor.column.saturating_sub(1);
            }
        }
        self.state.selection = Selection::collapsed(self.state.cursor);
        self.preferred_col = None;
        self.update_scroll();
    }

    /// Delete character after cursor (delete key)
    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(self.buffer.len_chars());

        if char_idx < self.buffer.len_chars() {
            self.buffer.delete_range(char_idx, char_idx + 1);
        }
        // Cursor stays in place
        self.preferred_col = None;
    }

    /// Delete current selection if any, returns true if something was deleted
    fn delete_selection(&mut self) -> bool {
        if self.state.selection.is_collapsed() {
            return false;
        }

        let (start, end) = self.state.selection.normalized();
        let start_idx = self
            .buffer
            .line_col_to_char(start.line, start.column)
            .unwrap_or(0);
        let end_idx = self
            .buffer
            .line_col_to_char(end.line, end.column)
            .unwrap_or(self.buffer.len_chars());

        self.buffer.delete_range(start_idx, end_idx);
        self.state.cursor = start;
        self.state.selection = Selection::collapsed(start);
        self.preferred_col = None;
        true
    }

    /// Get selected text, if any
    pub fn selected_text(&self) -> Option<String> {
        if self.state.selection.is_collapsed() {
            return None;
        }

        let (start, end) = self.state.selection.normalized();
        let start_idx = self
            .buffer
            .line_col_to_char(start.line, start.column)
            .unwrap_or(0);
        let end_idx = self
            .buffer
            .line_col_to_char(end.line, end.column)
            .unwrap_or(self.buffer.len_chars());
        Some(self.buffer.slice(start_idx, end_idx))
    }

    /// Delete word before cursor
    pub fn delete_word_left(&mut self) {
        if self.delete_selection() {
            return;
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(0);

        if char_idx > 0 {
            let word_start = self.buffer.prev_word_boundary(char_idx);
            self.buffer.delete_range(word_start, char_idx);

            let (line, col) = self.buffer.char_to_line_col(word_start);
            self.state.cursor = CursorPosition::new(line, col);
        }
        self.state.selection = Selection::collapsed(self.state.cursor);
        self.preferred_col = None;
        self.update_scroll();
    }

    /// Delete word after cursor
    pub fn delete_word_right(&mut self) {
        if self.delete_selection() {
            return;
        }

        let char_idx = self
            .buffer
            .line_col_to_char(self.state.cursor.line, self.state.cursor.column)
            .unwrap_or(self.buffer.len_chars());

        if char_idx < self.buffer.len_chars() {
            let word_end = self.buffer.next_word_boundary(char_idx);
            self.buffer.delete_range(char_idx, word_end);
        }
        // Cursor stays in place
        self.preferred_col = None;
    }

    // === Line Information ===

    /// Get total line count
    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.buffer.len_chars()
    }

    /// Get specific line content
    pub fn get_line(&self, line_idx: usize) -> Option<String> {
        self.buffer.line_without_newline(line_idx)
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
