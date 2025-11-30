//! Clipboard integration for copy, cut, and paste operations
//!
//! Uses the arboard crate for cross-platform clipboard access,
//! with special handling for Wayland and X11 on Linux.

use arboard::Clipboard;
use std::sync::Mutex;
use thiserror::Error;

/// Clipboard-related errors
#[derive(Error, Debug, Clone)]
pub enum ClipboardError {
    #[error("Failed to access clipboard: {0}")]
    AccessError(String),
    
    #[error("Clipboard is empty")]
    Empty,
    
    #[error("Clipboard contents are not text")]
    NotText,
    
    #[error("Failed to write to clipboard: {0}")]
    WriteError(String),
}

/// Thread-safe clipboard wrapper
/// 
/// Note: arboard's Clipboard is not Send/Sync on all platforms,
/// so we use a lazy initialization pattern per-operation.
pub struct ClipboardManager {
    /// Last known clipboard content (for fallback)
    last_content: Mutex<Option<String>>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        Self {
            last_content: Mutex::new(None),
        }
    }

    /// Get text from clipboard
    pub fn get_text(&self) -> Result<String, ClipboardError> {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.get_text() {
                    Ok(text) => {
                        // Cache the content
                        if let Ok(mut cache) = self.last_content.lock() {
                            *cache = Some(text.clone());
                        }
                        Ok(text)
                    }
                    Err(arboard::Error::ContentNotAvailable) => Err(ClipboardError::Empty),
                    Err(e) => Err(ClipboardError::AccessError(e.to_string())),
                }
            }
            Err(e) => {
                // Try to return cached content as fallback
                if let Ok(cache) = self.last_content.lock() {
                    if let Some(ref text) = *cache {
                        return Ok(text.clone());
                    }
                }
                Err(ClipboardError::AccessError(e.to_string()))
            }
        }
    }

    /// Set text to clipboard
    pub fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        // Cache the content first
        if let Ok(mut cache) = self.last_content.lock() {
            *cache = Some(text.to_string());
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard
                    .set_text(text)
                    .map_err(|e| ClipboardError::WriteError(e.to_string()))
            }
            Err(e) => Err(ClipboardError::AccessError(e.to_string())),
        }
    }

    /// Check if clipboard has text content
    pub fn has_text(&self) -> bool {
        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.get_text().is_ok()
        } else {
            // Check cache
            if let Ok(cache) = self.last_content.lock() {
                cache.is_some()
            } else {
                false
            }
        }
    }

    /// Clear the clipboard
    pub fn clear(&self) -> Result<(), ClipboardError> {
        if let Ok(mut cache) = self.last_content.lock() {
            *cache = None;
        }

        match Clipboard::new() {
            Ok(mut clipboard) => {
                clipboard
                    .clear()
                    .map_err(|e| ClipboardError::WriteError(e.to_string()))
            }
            Err(e) => Err(ClipboardError::AccessError(e.to_string())),
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global clipboard instance for synchronous operations
/// 
/// For async clipboard operations in cosmic/iced, prefer using
/// the Task-based approach in the app module.
static CLIPBOARD: std::sync::OnceLock<ClipboardManager> = std::sync::OnceLock::new();

/// Get the global clipboard manager
pub fn clipboard() -> &'static ClipboardManager {
    CLIPBOARD.get_or_init(ClipboardManager::new)
}

/// Convenience function to copy text to clipboard
pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
    clipboard().set_text(text)
}

/// Convenience function to get text from clipboard
pub fn paste_text() -> Result<String, ClipboardError> {
    clipboard().get_text()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        // Just verify it creates without panic
        assert!(manager.last_content.lock().unwrap().is_none());
    }

    // Note: Full clipboard tests require a display server
    // and are better suited for integration tests
}
