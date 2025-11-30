//! Cursor management for text editing
//!
//! Handles cursor positioning, movement, and preferred column tracking.

use crate::editor::buffer::TextBuffer;
use crate::state::CursorPosition;

/// Cursor controller for navigating within a text buffer
pub struct CursorController;

impl CursorController {
    /// Move cursor left by one character
    pub fn move_left(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        if pos.column > 0 {
            CursorPosition::new(pos.line, pos.column - 1)
        } else if pos.line > 0 {
            // Wrap to end of previous line
            let prev_line_len = buffer.line_len(pos.line - 1).unwrap_or(0);
            CursorPosition::new(pos.line - 1, prev_line_len)
        } else {
            pos
        }
    }

    /// Move cursor right by one character
    pub fn move_right(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let line_len = buffer.line_len(pos.line).unwrap_or(0);

        if pos.column < line_len {
            CursorPosition::new(pos.line, pos.column + 1)
        } else if pos.line < buffer.len_lines().saturating_sub(1) {
            // Wrap to start of next line
            CursorPosition::new(pos.line + 1, 0)
        } else {
            pos
        }
    }

    /// Move cursor up by one line
    pub fn move_up(
        buffer: &TextBuffer,
        pos: CursorPosition,
        preferred_col: Option<usize>,
    ) -> (CursorPosition, Option<usize>) {
        if pos.line == 0 {
            return (pos, preferred_col);
        }

        let target_col = preferred_col.unwrap_or(pos.column);
        let prev_line_len = buffer.line_len(pos.line - 1).unwrap_or(0);
        let new_col = target_col.min(prev_line_len);

        (
            CursorPosition::new(pos.line - 1, new_col),
            Some(target_col),
        )
    }

    /// Move cursor down by one line
    pub fn move_down(
        buffer: &TextBuffer,
        pos: CursorPosition,
        preferred_col: Option<usize>,
    ) -> (CursorPosition, Option<usize>) {
        if pos.line >= buffer.len_lines().saturating_sub(1) {
            return (pos, preferred_col);
        }

        let target_col = preferred_col.unwrap_or(pos.column);
        let next_line_len = buffer.line_len(pos.line + 1).unwrap_or(0);
        let new_col = target_col.min(next_line_len);

        (
            CursorPosition::new(pos.line + 1, new_col),
            Some(target_col),
        )
    }

    /// Move cursor to start of line (smart home: first non-whitespace, then column 0)
    pub fn move_home(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let line = buffer.line_without_newline(pos.line).unwrap_or_default();
        let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);

        if pos.column == first_non_ws || pos.column == 0 {
            // Toggle between first non-whitespace and column 0
            if pos.column == 0 && first_non_ws > 0 {
                CursorPosition::new(pos.line, first_non_ws)
            } else {
                CursorPosition::new(pos.line, 0)
            }
        } else {
            CursorPosition::new(pos.line, first_non_ws)
        }
    }

    /// Move cursor to end of line
    pub fn move_end(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let line_len = buffer.line_len(pos.line).unwrap_or(0);
        CursorPosition::new(pos.line, line_len)
    }

    /// Move cursor to start of previous word
    pub fn move_word_left(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let char_idx = buffer.line_col_to_char(pos.line, pos.column).unwrap_or(0);
        let new_idx = buffer.prev_word_boundary(char_idx);
        let (line, col) = buffer.char_to_line_col(new_idx);
        CursorPosition::new(line, col)
    }

    /// Move cursor to start of next word
    pub fn move_word_right(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let char_idx = buffer
            .line_col_to_char(pos.line, pos.column)
            .unwrap_or(buffer.len_chars());
        let new_idx = buffer.next_word_boundary(char_idx);
        let (line, col) = buffer.char_to_line_col(new_idx);
        CursorPosition::new(line, col)
    }

    /// Move cursor up by a page (viewport_lines lines)
    pub fn move_page_up(
        buffer: &TextBuffer,
        pos: CursorPosition,
        viewport_lines: usize,
        preferred_col: Option<usize>,
    ) -> (CursorPosition, Option<usize>) {
        let target_col = preferred_col.unwrap_or(pos.column);
        let new_line = pos.line.saturating_sub(viewport_lines);
        let line_len = buffer.line_len(new_line).unwrap_or(0);
        let new_col = target_col.min(line_len);

        (CursorPosition::new(new_line, new_col), Some(target_col))
    }

    /// Move cursor down by a page (viewport_lines lines)
    pub fn move_page_down(
        buffer: &TextBuffer,
        pos: CursorPosition,
        viewport_lines: usize,
        preferred_col: Option<usize>,
    ) -> (CursorPosition, Option<usize>) {
        let target_col = preferred_col.unwrap_or(pos.column);
        let max_line = buffer.len_lines().saturating_sub(1);
        let new_line = (pos.line + viewport_lines).min(max_line);
        let line_len = buffer.line_len(new_line).unwrap_or(0);
        let new_col = target_col.min(line_len);

        (CursorPosition::new(new_line, new_col), Some(target_col))
    }

    /// Move cursor to start of document
    pub fn move_document_start() -> CursorPosition {
        CursorPosition::new(0, 0)
    }

    /// Move cursor to end of document
    pub fn move_document_end(buffer: &TextBuffer) -> CursorPosition {
        let last_line = buffer.len_lines().saturating_sub(1);
        let last_col = buffer.line_len(last_line).unwrap_or(0);
        CursorPosition::new(last_line, last_col)
    }

    /// Move cursor to specific line (1-indexed for user input)
    pub fn go_to_line(buffer: &TextBuffer, line_number: usize) -> CursorPosition {
        let line = line_number.saturating_sub(1); // Convert to 0-indexed
        let max_line = buffer.len_lines().saturating_sub(1);
        let target_line = line.min(max_line);
        CursorPosition::new(target_line, 0)
    }

    /// Ensure cursor position is valid within buffer bounds
    pub fn clamp(buffer: &TextBuffer, pos: CursorPosition) -> CursorPosition {
        let max_line = buffer.len_lines().saturating_sub(1);
        let line = pos.line.min(max_line);
        let line_len = buffer.line_len(line).unwrap_or(0);
        let col = pos.column.min(line_len);
        CursorPosition::new(line, col)
    }
}

/// Calculate scroll position to keep cursor visible
pub fn calculate_scroll(
    cursor_line: usize,
    current_scroll: usize,
    viewport_lines: usize,
    scroll_margin: usize,
) -> usize {
    let margin = scroll_margin.min(viewport_lines / 2);

    // Cursor too far up
    if cursor_line < current_scroll + margin {
        return cursor_line.saturating_sub(margin);
    }

    // Cursor too far down
    let viewport_bottom = current_scroll + viewport_lines;
    if cursor_line >= viewport_bottom.saturating_sub(margin) {
        return cursor_line
            .saturating_sub(viewport_lines)
            .saturating_add(margin + 1);
    }

    current_scroll
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buffer() -> TextBuffer {
        TextBuffer::from_str("Line one\nLine two\nLine three\nLine four")
    }

    #[test]
    fn test_move_left() {
        let buf = make_buffer();

        // Normal left
        let pos = CursorController::move_left(&buf, CursorPosition::new(0, 5));
        assert_eq!(pos, CursorPosition::new(0, 4));

        // Wrap to previous line
        let pos = CursorController::move_left(&buf, CursorPosition::new(1, 0));
        assert_eq!(pos, CursorPosition::new(0, 8)); // End of "Line one"

        // At document start
        let pos = CursorController::move_left(&buf, CursorPosition::new(0, 0));
        assert_eq!(pos, CursorPosition::new(0, 0));
    }

    #[test]
    fn test_move_right() {
        let buf = make_buffer();

        // Normal right
        let pos = CursorController::move_right(&buf, CursorPosition::new(0, 5));
        assert_eq!(pos, CursorPosition::new(0, 6));

        // Wrap to next line
        let pos = CursorController::move_right(&buf, CursorPosition::new(0, 8));
        assert_eq!(pos, CursorPosition::new(1, 0));
    }

    #[test]
    fn test_move_up_down() {
        let buf = make_buffer();

        // Move down
        let (pos, pref) = CursorController::move_down(&buf, CursorPosition::new(0, 5), None);
        assert_eq!(pos, CursorPosition::new(1, 5));
        assert_eq!(pref, Some(5));

        // Move up
        let (pos, _) = CursorController::move_up(&buf, CursorPosition::new(1, 5), Some(5));
        assert_eq!(pos, CursorPosition::new(0, 5));

        // Move down with preferred column on shorter line
        let buf2 = TextBuffer::from_str("Long line here\nShort\nAnother long line");
        let (pos, pref) = CursorController::move_down(&buf2, CursorPosition::new(0, 12), None);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5); // Clamped to line length
        assert_eq!(pref, Some(12)); // Preferred column remembered

        // Move down again with preferred column
        let (pos, _) = CursorController::move_down(&buf2, pos, pref);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 12); // Restored to preferred
    }

    #[test]
    fn test_move_home_end() {
        let buf = TextBuffer::from_str("  Hello world");

        // Home goes to first non-whitespace
        let pos = CursorController::move_home(&buf, CursorPosition::new(0, 8));
        assert_eq!(pos, CursorPosition::new(0, 2));

        // Home again goes to column 0
        let pos = CursorController::move_home(&buf, pos);
        assert_eq!(pos, CursorPosition::new(0, 0));

        // Home from 0 goes to first non-whitespace
        let pos = CursorController::move_home(&buf, pos);
        assert_eq!(pos, CursorPosition::new(0, 2));

        // End goes to line end
        let pos = CursorController::move_end(&buf, CursorPosition::new(0, 0));
        assert_eq!(pos, CursorPosition::new(0, 13));
    }

    #[test]
    fn test_go_to_line() {
        let buf = make_buffer();

        let pos = CursorController::go_to_line(&buf, 3); // User enters 3 (1-indexed)
        assert_eq!(pos, CursorPosition::new(2, 0)); // 0-indexed line 2

        // Out of bounds
        let pos = CursorController::go_to_line(&buf, 100);
        assert_eq!(pos.line, 3); // Clamped to last line
    }

    #[test]
    fn test_calculate_scroll() {
        // Cursor in view
        assert_eq!(calculate_scroll(10, 5, 20, 3), 5);

        // Cursor above viewport
        assert_eq!(calculate_scroll(2, 10, 20, 3), 0);

        // Cursor below viewport
        assert_eq!(calculate_scroll(30, 5, 20, 3), 14);
    }
}
