//! Text buffer implementation using ropey
//!
//! Provides efficient text storage and manipulation for large files
//! with O(log n) insert/delete operations.

use ropey::Rope;
use std::ops::Range;

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    /// Unix-style line endings (LF: \n)
    #[default]
    Lf,
    /// Windows-style line endings (CRLF: \r\n)
    Crlf,
}

impl LineEnding {
    /// Get the string representation of the line ending
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
        }
    }

    /// Detect line ending from text
    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        }
    }
}

/// Text buffer wrapping ropey::Rope with additional metadata
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// The underlying rope data structure
    rope: Rope,

    /// Line ending style for this buffer
    line_ending: LineEnding,

    /// Version number, incremented on each change
    version: u64,

    /// Whether the buffer has been modified since last save
    modified: bool,

    /// Version number when last saved
    saved_version: u64,
}

impl TextBuffer {
    /// Create an empty text buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            line_ending: LineEnding::default(),
            version: 0,
            modified: false,
            saved_version: 0,
        }
    }

    /// Create a buffer from a string
    pub fn from_str(text: &str) -> Self {
        let line_ending = LineEnding::detect(text);
        // Normalize to LF internally
        let normalized = text.replace("\r\n", "\n");
        Self {
            rope: Rope::from_str(&normalized),
            line_ending,
            version: 0,
            modified: false,
            saved_version: 0,
        }
    }

    /// Get the underlying rope
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Get a mutable reference to the rope
    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    /// Get the current version number
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get line ending style
    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// Set line ending style
    pub fn set_line_ending(&mut self, ending: LineEnding) {
        self.line_ending = ending;
        self.version += 1;
        self.modified = true;
    }

    /// Check if buffer has been modified since last save
    pub fn is_modified(&self) -> bool {
        self.modified || self.version != self.saved_version
    }

    /// Mark buffer as saved
    pub fn mark_saved(&mut self) {
        self.modified = false;
        self.saved_version = self.version;
    }

    /// Get total character count
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Get total byte count
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Get total line count
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Get a specific line as a string (0-indexed)
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            return None;
        }
        Some(self.rope.line(line_idx).to_string())
    }

    /// Get a line without the trailing newline
    pub fn line_without_newline(&self, line_idx: usize) -> Option<String> {
        self.line(line_idx).map(|s| {
            s.trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string()
        })
    }

    /// Get the length of a specific line (in characters, excluding newline)
    pub fn line_len(&self, line_idx: usize) -> Option<usize> {
        self.line_without_newline(line_idx).map(|s| s.chars().count())
    }

    /// Convert (line, column) to character offset
    pub fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let line_start = self.rope.line_to_char(line);
        let line_len = self.line_len(line).unwrap_or(0);
        let clamped_col = col.min(line_len);
        Some(line_start + clamped_col)
    }

    /// Convert character offset to (line, column)
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.rope.len_chars());
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    /// Insert text at character position
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        let idx = char_idx.min(self.rope.len_chars());
        self.rope.insert(idx, text);
        self.version += 1;
        self.modified = true;
    }

    /// Insert string at character position (alias for insert)
    pub fn insert_str(&mut self, char_idx: usize, text: &str) {
        self.insert(char_idx, text);
    }

    /// Insert text at (line, column)
    pub fn insert_at(&mut self, line: usize, col: usize, text: &str) {
        if let Some(char_idx) = self.line_col_to_char(line, col) {
            self.insert(char_idx, text);
        } else {
            // Insert at end if position is invalid
            self.insert(self.rope.len_chars(), text);
        }
    }

    /// Insert a single character
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.insert(char_idx, &ch.to_string());
    }

    /// Delete a range of characters
    pub fn delete(&mut self, range: Range<usize>) {
        let start = range.start.min(self.rope.len_chars());
        let end = range.end.min(self.rope.len_chars());
        if start < end {
            self.rope.remove(start..end);
            self.version += 1;
            self.modified = true;
        }
    }

    /// Delete a range by character indices (start inclusive, end exclusive)
    pub fn delete_range(&mut self, start_char: usize, end_char: usize) {
        self.delete(start_char..end_char);
    }

    /// Delete from (line, col) to (line, col)
    pub fn delete_by_line_col(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) {
        let start = self.line_col_to_char(start_line, start_col).unwrap_or(0);
        let end = self.line_col_to_char(end_line, end_col).unwrap_or(self.rope.len_chars());
        self.delete(start..end);
    }

    /// Replace a range of characters
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        self.delete(range.clone());
        self.insert(range.start, text);
    }

    /// Get a slice of the buffer as a string (Range version)
    pub fn slice_range(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.rope.len_chars());
        let end = range.end.min(self.rope.len_chars());
        if start >= end {
            return String::new();
        }
        self.rope.slice(start..end).to_string()
    }

    /// Get a slice of the buffer as a string (start and end char indices)
    pub fn slice(&self, start_char: usize, end_char: usize) -> String {
        self.slice_range(start_char..end_char)
    }

    /// Get the entire buffer contents as a string
    pub fn to_string(&self) -> String {
        let content = self.rope.to_string();
        // Convert back to original line endings if needed
        if self.line_ending == LineEnding::Crlf {
            content.replace('\n', "\r\n")
        } else {
            content
        }
    }

    /// Get contents with specific line ending
    pub fn to_string_with_ending(&self, ending: LineEnding) -> String {
        let content = self.rope.to_string();
        match ending {
            LineEnding::Lf => content,
            LineEnding::Crlf => content.replace('\n', "\r\n"),
        }
    }

    /// Set the entire buffer contents
    pub fn set_content(&mut self, text: &str) {
        self.line_ending = LineEnding::detect(text);
        let normalized = text.replace("\r\n", "\n");
        self.rope = Rope::from_str(&normalized);
        self.version += 1;
    }

    /// Get character at position
    pub fn char_at(&self, char_idx: usize) -> Option<char> {
        if char_idx >= self.rope.len_chars() {
            return None;
        }
        Some(self.rope.char(char_idx))
    }

    /// Get word at position (returns start and end char indices)
    pub fn word_at(&self, char_idx: usize) -> Option<(usize, usize)> {
        if char_idx >= self.rope.len_chars() {
            return None;
        }

        // Find word start
        let mut start = char_idx;
        while start > 0 {
            let ch = self.rope.char(start - 1);
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            start -= 1;
        }

        // Find word end
        let mut end = char_idx;
        while end < self.rope.len_chars() {
            let ch = self.rope.char(end);
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            end += 1;
        }

        if start == end {
            None
        } else {
            Some((start, end))
        }
    }

    /// Find next word boundary from position
    pub fn next_word_boundary(&self, char_idx: usize) -> usize {
        let len = self.rope.len_chars();
        if char_idx >= len {
            return len;
        }

        let mut idx = char_idx;

        // Skip current word/non-word
        let start_is_word = self.rope.char(idx).is_alphanumeric();
        while idx < len {
            let is_word = self.rope.char(idx).is_alphanumeric();
            if is_word != start_is_word {
                break;
            }
            idx += 1;
        }

        // Skip whitespace
        while idx < len && self.rope.char(idx).is_whitespace() && self.rope.char(idx) != '\n' {
            idx += 1;
        }

        idx
    }

    /// Find previous word boundary from position
    pub fn prev_word_boundary(&self, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }

        let mut idx = char_idx;

        // Skip whitespace
        while idx > 0 && self.rope.char(idx - 1).is_whitespace() && self.rope.char(idx - 1) != '\n'
        {
            idx -= 1;
        }

        if idx == 0 {
            return 0;
        }

        // Skip current word/non-word
        let start_is_word = self.rope.char(idx - 1).is_alphanumeric();
        while idx > 0 {
            let is_word = self.rope.char(idx - 1).is_alphanumeric();
            if is_word != start_is_word {
                break;
            }
            idx -= 1;
        }

        idx
    }

    /// Count words in buffer
    pub fn word_count(&self) -> usize {
        let text = self.rope.to_string();
        text.split_whitespace().count()
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        Self::from_str(text)
    }
}

impl From<String> for TextBuffer {
    fn from(text: String) -> Self {
        Self::from_str(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buf = TextBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len_chars(), 0);
        assert_eq!(buf.len_lines(), 1); // Rope always has at least 1 line
    }

    #[test]
    fn test_from_str() {
        let buf = TextBuffer::from_str("Hello\nWorld");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.line(0), Some("Hello\n".to_string()));
        assert_eq!(buf.line(1), Some("World".to_string()));
    }

    #[test]
    fn test_line_ending_detection() {
        let lf_buf = TextBuffer::from_str("Hello\nWorld");
        assert_eq!(lf_buf.line_ending(), LineEnding::Lf);

        let crlf_buf = TextBuffer::from_str("Hello\r\nWorld");
        assert_eq!(crlf_buf.line_ending(), LineEnding::Crlf);
    }

    #[test]
    fn test_insert() {
        let mut buf = TextBuffer::from_str("Hello World");
        buf.insert(5, ",");
        assert_eq!(buf.to_string(), "Hello, World");
    }

    #[test]
    fn test_delete() {
        let mut buf = TextBuffer::from_str("Hello, World");
        buf.delete(5..7);
        assert_eq!(buf.to_string(), "Hello World");
    }

    #[test]
    fn test_line_col_conversion() {
        let buf = TextBuffer::from_str("Line 1\nLine 2\nLine 3");

        assert_eq!(buf.line_col_to_char(0, 0), Some(0));
        assert_eq!(buf.line_col_to_char(1, 0), Some(7));
        assert_eq!(buf.line_col_to_char(1, 4), Some(11));

        assert_eq!(buf.char_to_line_col(0), (0, 0));
        assert_eq!(buf.char_to_line_col(7), (1, 0));
        assert_eq!(buf.char_to_line_col(11), (1, 4));
    }

    #[test]
    fn test_word_boundaries() {
        let buf = TextBuffer::from_str("Hello world  test");

        assert_eq!(buf.next_word_boundary(0), 6); // After "Hello "
        assert_eq!(buf.next_word_boundary(6), 13); // After "world  "
        assert_eq!(buf.prev_word_boundary(17), 13); // Before "test"
        assert_eq!(buf.prev_word_boundary(6), 0); // Before "Hello"
    }

    #[test]
    fn test_word_at() {
        let buf = TextBuffer::from_str("Hello world");

        assert_eq!(buf.word_at(2), Some((0, 5))); // "Hello"
        assert_eq!(buf.word_at(7), Some((6, 11))); // "world"
        assert_eq!(buf.word_at(5), None); // On space
    }

    #[test]
    fn test_word_count() {
        let buf = TextBuffer::from_str("Hello world, this is a test.");
        assert_eq!(buf.word_count(), 6);
    }

    #[test]
    fn test_version_increments() {
        let mut buf = TextBuffer::new();
        assert_eq!(buf.version(), 0);

        buf.insert(0, "test");
        assert_eq!(buf.version(), 1);

        buf.delete(0..2);
        assert_eq!(buf.version(), 2);
    }
}
