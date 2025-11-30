//! Find and replace functionality
//!
//! Provides text search with support for:
//! - Plain text search
//! - Case-insensitive search
//! - Whole word matching
//! - Regular expressions

use std::ops::Range;

/// Search direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchDirection {
    #[default]
    Forward,
    Backward,
}

/// Options for search operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindOptions {
    /// Case-sensitive matching
    pub case_sensitive: bool,
    /// Match whole words only
    pub whole_word: bool,
    /// Use regular expressions
    pub use_regex: bool,
    /// Wrap around at document boundaries
    pub wrap_around: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            wrap_around: true,
        }
    }
}

/// A single search result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindResult {
    /// Start character offset in the document
    pub start: usize,
    /// End character offset (exclusive)
    pub end: usize,
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// The matched text
    pub matched_text: String,
}

impl FindResult {
    /// Create a new find result
    pub fn new(start: usize, end: usize, line: usize, column: usize, matched_text: String) -> Self {
        Self {
            start,
            end,
            line,
            column,
            matched_text,
        }
    }

    /// Get the range as a Rust Range
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Get the length of the match
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if the match is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Search engine for finding text in documents
#[derive(Debug, Clone)]
pub struct SearchEngine {
    /// Compiled regex pattern (if using regex mode)
    regex_pattern: Option<regex::Regex>,
    /// Last query used (for caching)
    last_query: String,
    /// Last options used (for caching)
    last_options: FindOptions,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            regex_pattern: None,
            last_query: String::new(),
            last_options: FindOptions::default(),
        }
    }

    /// Find all matches in the given text
    pub fn find_all(&mut self, text: &str, query: &str, options: &FindOptions) -> Vec<FindResult> {
        if query.is_empty() {
            return Vec::new();
        }

        // Update cached pattern if needed
        if query != self.last_query || options != &self.last_options {
            self.update_pattern(query, options);
            self.last_query = query.to_string();
            self.last_options = options.clone();
        }

        if options.use_regex {
            self.find_regex(text)
        } else {
            self.find_plain(text, query, options)
        }
    }

    /// Update the regex pattern based on query and options
    fn update_pattern(&mut self, query: &str, options: &FindOptions) {
        if options.use_regex {
            let pattern = if options.case_sensitive {
                query.to_string()
            } else {
                format!("(?i){}", query)
            };

            self.regex_pattern = regex::Regex::new(&pattern).ok();
        } else {
            // Build regex from plain text for consistency
            let escaped = regex::escape(query);
            let pattern = if options.whole_word {
                format!(r"\b{}\b", escaped)
            } else {
                escaped
            };

            let pattern = if options.case_sensitive {
                pattern
            } else {
                format!("(?i){}", pattern)
            };

            self.regex_pattern = regex::Regex::new(&pattern).ok();
        }
    }

    /// Find matches using regex
    fn find_regex(&self, text: &str) -> Vec<FindResult> {
        let Some(ref regex) = self.regex_pattern else {
            return Vec::new();
        };

        let mut results = Vec::new();
        let mut line_starts: Vec<usize> = vec![0];
        
        // Build line start index
        for (i, c) in text.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        for mat in regex.find_iter(text) {
            let start = mat.start();
            let end = mat.end();
            
            // Find line and column
            let line = line_starts.partition_point(|&ls| ls <= start).saturating_sub(1);
            let line_start = line_starts.get(line).copied().unwrap_or(0);
            let column = text[line_start..start].chars().count();

            results.push(FindResult::new(
                start,
                end,
                line,
                column,
                mat.as_str().to_string(),
            ));
        }

        results
    }

    /// Find matches using plain text search
    fn find_plain(&self, text: &str, query: &str, options: &FindOptions) -> Vec<FindResult> {
        let mut results = Vec::new();
        
        let (search_text, search_query): (String, String) = if options.case_sensitive {
            (text.to_string(), query.to_string())
        } else {
            (text.to_lowercase(), query.to_lowercase())
        };

        // Build line start index
        let mut line_starts: Vec<usize> = vec![0];
        for (i, c) in text.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        let mut start = 0;
        while let Some(pos) = search_text[start..].find(&search_query) {
            let match_start = start + pos;
            let match_end = match_start + query.len();

            // Check whole word if needed
            if options.whole_word {
                let before_ok = match_start == 0 
                    || !text.chars().nth(match_start - 1).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                let after_ok = match_end >= text.len()
                    || !text.chars().nth(match_end).map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
                
                if !before_ok || !after_ok {
                    start = match_start + 1;
                    continue;
                }
            }

            // Find line and column
            let line = line_starts.partition_point(|&ls| ls <= match_start).saturating_sub(1);
            let line_start = line_starts.get(line).copied().unwrap_or(0);
            let column = text[line_start..match_start].chars().count();

            results.push(FindResult::new(
                match_start,
                match_end,
                line,
                column,
                text[match_start..match_end].to_string(),
            ));

            start = match_start + 1;
        }

        results
    }

    /// Find next match from a given position
    pub fn find_next(
        &mut self,
        text: &str,
        query: &str,
        from_pos: usize,
        options: &FindOptions,
    ) -> Option<FindResult> {
        let all_results = self.find_all(text, query, options);
        
        // Find first result after from_pos
        for result in &all_results {
            if result.start >= from_pos {
                return Some(result.clone());
            }
        }

        // Wrap around if enabled
        if options.wrap_around {
            all_results.into_iter().next()
        } else {
            None
        }
    }

    /// Find previous match from a given position
    pub fn find_prev(
        &mut self,
        text: &str,
        query: &str,
        from_pos: usize,
        options: &FindOptions,
    ) -> Option<FindResult> {
        let all_results = self.find_all(text, query, options);
        
        // Find last result before from_pos
        for result in all_results.iter().rev() {
            if result.end <= from_pos {
                return Some(result.clone());
            }
        }

        // Wrap around if enabled
        if options.wrap_around {
            all_results.into_iter().last()
        } else {
            None
        }
    }

    /// Replace all occurrences and return the new text
    pub fn replace_all(
        &mut self,
        text: &str,
        query: &str,
        replacement: &str,
        options: &FindOptions,
    ) -> (String, usize) {
        let results = self.find_all(text, query, options);
        let count = results.len();

        if results.is_empty() {
            return (text.to_string(), 0);
        }

        // Build new text by replacing matches
        let mut new_text = String::with_capacity(text.len());
        let mut last_end = 0;

        for result in results {
            new_text.push_str(&text[last_end..result.start]);
            new_text.push_str(replacement);
            last_end = result.end;
        }
        new_text.push_str(&text[last_end..]);

        (new_text, count)
    }

    /// Replace a single occurrence at the given range
    pub fn replace_at(text: &str, range: Range<usize>, replacement: &str) -> String {
        let mut new_text = String::with_capacity(text.len());
        new_text.push_str(&text[..range.start]);
        new_text.push_str(replacement);
        new_text.push_str(&text[range.end..]);
        new_text
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_plain_text() {
        let mut engine = SearchEngine::new();
        let text = "Hello world, hello universe";
        let options = FindOptions::default();

        let results = engine.find_all(text, "hello", &options);
        assert_eq!(results.len(), 2); // case insensitive
        assert_eq!(results[0].start, 0);
        assert_eq!(results[1].start, 13);
    }

    #[test]
    fn test_find_case_sensitive() {
        let mut engine = SearchEngine::new();
        let text = "Hello world, hello universe";
        let options = FindOptions {
            case_sensitive: true,
            ..Default::default()
        };

        let results = engine.find_all(text, "hello", &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].start, 13);
    }

    #[test]
    fn test_find_whole_word() {
        let mut engine = SearchEngine::new();
        let text = "hello helloworld hello";
        let options = FindOptions {
            whole_word: true,
            ..Default::default()
        };

        let results = engine.find_all(text, "hello", &options);
        assert_eq!(results.len(), 2); // Only standalone "hello"
    }

    #[test]
    fn test_find_line_column() {
        let mut engine = SearchEngine::new();
        let text = "line one\nline two\nfind me here";
        let options = FindOptions::default();

        let results = engine.find_all(text, "me", &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, 2);
        assert_eq!(results[0].column, 5);
    }

    #[test]
    fn test_replace_all() {
        let mut engine = SearchEngine::new();
        let text = "foo bar foo baz foo";
        let options = FindOptions::default();

        let (new_text, count) = engine.replace_all(text, "foo", "qux", &options);
        assert_eq!(new_text, "qux bar qux baz qux");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_replace_at() {
        let text = "Hello world";
        let new_text = SearchEngine::replace_at(text, 6..11, "Rust");
        assert_eq!(new_text, "Hello Rust");
    }

    #[test]
    fn test_find_next_wrap() {
        let mut engine = SearchEngine::new();
        let text = "one two one three";
        let options = FindOptions::default();

        // Find from end - should wrap to first
        let result = engine.find_next(text, "one", 15, &options);
        assert!(result.is_some());
        assert_eq!(result.unwrap().start, 0);
    }

    #[test]
    fn test_empty_query() {
        let mut engine = SearchEngine::new();
        let text = "some text";
        let options = FindOptions::default();

        let results = engine.find_all(text, "", &options);
        assert!(results.is_empty());
    }
}
