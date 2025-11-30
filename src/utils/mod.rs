//! Utilities module for Cosmic Notebook
//!
//! Shared helper functions and utilities including:
//! - Debouncing
//! - Path utilities
//! - Platform-specific helpers
//! - Text utilities

use std::path::{Path, PathBuf};

/// Debounce helper for rate-limiting operations
pub struct Debouncer {
    delay_ms: u64,
    last_trigger: Option<std::time::Instant>,
}

impl Debouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            last_trigger: None,
        }
    }

    /// Check if enough time has passed since last trigger
    pub fn should_trigger(&mut self) -> bool {
        let now = std::time::Instant::now();
        match self.last_trigger {
            Some(last) => {
                if now.duration_since(last).as_millis() >= self.delay_ms as u128 {
                    self.last_trigger = Some(now);
                    true
                } else {
                    false
                }
            }
            None => {
                self.last_trigger = Some(now);
                true
            }
        }
    }

    /// Reset the debouncer
    pub fn reset(&mut self) {
        self.last_trigger = None;
    }
}

/// Path utilities
pub mod path {
    use super::*;

    /// Get the file name without extension
    pub fn file_stem(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// Get the file extension
    pub fn extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// Check if path has a markdown extension
    pub fn is_markdown(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("md" | "markdown" | "mdown" | "mkd")
        )
    }

    /// Make a path relative to a base path
    pub fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
        path.strip_prefix(base).ok().map(|p| p.to_path_buf())
    }

    /// Expand tilde to home directory
    pub fn expand_tilde(path: &Path) -> PathBuf {
        if let Ok(stripped) = path.strip_prefix("~") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        path.to_path_buf()
    }
}

/// Text utilities
pub mod text {
    /// Count words in text
    pub fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Count lines in text
    pub fn line_count(text: &str) -> usize {
        if text.is_empty() {
            0
        } else {
            text.lines().count()
        }
    }

    /// Get the line at a given character offset
    pub fn line_at_offset(text: &str, offset: usize) -> Option<usize> {
        if offset > text.len() {
            return None;
        }

        Some(text[..offset].matches('\n').count())
    }

    /// Truncate string with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debouncer() {
        let mut debouncer = Debouncer::new(100);
        assert!(debouncer.should_trigger());
        assert!(!debouncer.should_trigger());
    }

    #[test]
    fn test_is_markdown() {
        assert!(path::is_markdown(Path::new("test.md")));
        assert!(path::is_markdown(Path::new("test.markdown")));
        assert!(!path::is_markdown(Path::new("test.txt")));
    }

    #[test]
    fn test_word_count() {
        assert_eq!(text::word_count("hello world"), 2);
        assert_eq!(text::word_count("  "), 0);
        assert_eq!(text::word_count("one"), 1);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(text::truncate("hello", 10), "hello");
        assert_eq!(text::truncate("hello world", 8), "hello...");
    }
}
