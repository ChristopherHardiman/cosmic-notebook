//! Error types for Cosmic Notebook
//!
//! This module defines all custom error types used throughout the application.
//! Error types are organized by category for clear error handling and user-friendly messages.

use std::path::PathBuf;
use thiserror::Error;

/// Main application error type encompassing all error categories
#[derive(Error, Debug)]
pub enum AppError {
    /// File I/O related errors
    #[error(transparent)]
    FileIO(#[from] FileError),

    /// Configuration errors
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Editor operation errors
    #[error(transparent)]
    Editor(#[from] EditorError),

    /// Clipboard errors
    #[error(transparent)]
    Clipboard(#[from] ClipboardError),

    /// File watcher errors
    #[error(transparent)]
    Watcher(#[from] WatcherError),

    /// Generic unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// File I/O related errors
#[derive(Error, Debug)]
pub enum FileError {
    /// File not found at specified path
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    /// File not found (with path field for compatibility)
    #[error("File not found: {path}")]
    NotFoundPath { path: PathBuf },

    /// Permission denied when accessing file
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// File is too large to open
    #[error("File is too large ({size_mb:.1} MB). Maximum size is {max_mb} MB: {path}")]
    TooLarge {
        path: PathBuf,
        size_mb: f64,
        max_mb: u64,
    },

    /// File is too large (alternate format)
    #[error("File too large: {path} ({size} bytes, max {max_size} bytes)")]
    FileTooLarge {
        path: PathBuf,
        size: u64,
        max_size: u64,
    },

    /// File encoding error (non-UTF-8)
    #[error("Unable to read file as text. File may be binary or use unsupported encoding: {path}")]
    EncodingError { path: PathBuf },

    /// Error reading file
    #[error("Could not read file: {path}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Error writing file
    #[error("Could not save file: {path}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Error during atomic write (temp file creation)
    #[error("Could not create temporary file for safe save: {path}")]
    AtomicWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Error renaming temp file to target
    #[error("Could not complete file save (rename failed): {path}")]
    RenameError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Error creating backup
    #[error("Could not create backup: {path}")]
    BackupError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Directory does not exist
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    /// Error scanning directory
    #[error("Could not read directory: {path}")]
    DirectoryScanError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Directory operation error
    #[error("Directory error: {path}")]
    DirectoryError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// File is read-only
    #[error("File is read-only: {path}")]
    ReadOnly { path: PathBuf },

    /// Path is not a file
    #[error("Path is not a file: {path}")]
    NotAFile { path: PathBuf },

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Error loading configuration file
    #[error("Could not load configuration: {0}")]
    LoadError(String),

    /// Error saving configuration
    #[error("Could not save configuration: {0}")]
    SaveError(String),

    /// Error parsing configuration
    #[error("Invalid configuration format: {0}")]
    ParseError(String),

    /// Missing required configuration value
    #[error("Missing configuration value: {key}")]
    MissingValue { key: String },

    /// Invalid configuration value
    #[error("Invalid value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },

    /// Configuration directory error
    #[error("Could not access configuration directory")]
    DirectoryError,
}

/// Editor operation errors
#[derive(Error, Debug)]
pub enum EditorError {
    /// Invalid cursor position
    #[error("Invalid cursor position: line {line}, column {column}")]
    InvalidCursorPosition { line: usize, column: usize },

    /// Invalid selection range
    #[error("Invalid selection range: {start} to {end}")]
    InvalidSelection { start: usize, end: usize },

    /// Undo stack is empty
    #[error("Nothing to undo")]
    NothingToUndo,

    /// Redo stack is empty
    #[error("Nothing to redo")]
    NothingToRedo,

    /// Document not found
    #[error("Document not found: {id}")]
    DocumentNotFound { id: String },

    /// Buffer operation failed
    #[error("Text operation failed: {0}")]
    BufferError(String),
}

/// Clipboard related errors
#[derive(Error, Debug)]
pub enum ClipboardError {
    /// Could not access clipboard
    #[error("Could not access clipboard")]
    AccessDenied,

    /// Clipboard is empty
    #[error("Clipboard is empty")]
    Empty,

    /// Clipboard content is not text
    #[error("Clipboard does not contain text")]
    NotText,

    /// Error getting clipboard content
    #[error("Could not read from clipboard: {0}")]
    ReadError(String),

    /// Error setting clipboard content
    #[error("Could not write to clipboard: {0}")]
    WriteError(String),
}

/// File watcher errors
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Could not initialize file watcher
    #[error("Could not start file watcher: {0}")]
    InitError(String),

    /// Could not watch path
    #[error("Could not watch path: {path}")]
    WatchError {
        path: PathBuf,
        #[source]
        source: notify::Error,
    },

    /// Too many files to watch
    #[error("Too many files to watch. System limit reached.")]
    TooManyWatches,

    /// Watcher event error
    #[error("File watcher error: {0}")]
    EventError(String),
}

/// Result type alias for operations that can fail with AppError
pub type AppResult<T> = Result<T, AppError>;

/// Result type alias for file operations
pub type FileResult<T> = Result<T, FileError>;

/// Result type alias for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Result type alias for editor operations
pub type EditorResult<T> = Result<T, EditorError>;

impl FileError {
    /// Create a user-friendly error message suitable for display in dialogs
    pub fn user_message(&self) -> String {
        match self {
            FileError::NotFound(_) | FileError::NotFoundPath { .. } => {
                "The file could not be found. It may have been moved or deleted.".to_string()
            }
            FileError::PermissionDenied { .. } => {
                "You don't have permission to access this file. Check file permissions.".to_string()
            }
            FileError::TooLarge { max_mb, .. } => {
                format!(
                    "This file is too large to open. Maximum file size is {} MB.",
                    max_mb
                )
            }
            FileError::FileTooLarge { max_size, .. } => {
                format!(
                    "This file is too large to open. Maximum file size is {} bytes.",
                    max_size
                )
            }
            FileError::EncodingError { .. } => {
                "This file cannot be opened as text. It may be a binary file or use an unsupported encoding.".to_string()
            }
            FileError::WriteError { .. } | FileError::AtomicWriteError { .. } => {
                "Could not save the file. Check disk space and permissions.".to_string()
            }
            FileError::ReadOnly { .. } => {
                "This file is read-only and cannot be modified.".to_string()
            }
            _ => self.to_string(),
        }
    }
}

impl ClipboardError {
    /// Create a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            ClipboardError::AccessDenied => {
                "Could not access the clipboard. Another application may be using it.".to_string()
            }
            ClipboardError::Empty => "The clipboard is empty.".to_string(),
            ClipboardError::NotText => "The clipboard does not contain text.".to_string(),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_error_display() {
        let err = FileError::NotFound(PathBuf::from("/test/file.md"));
        assert!(err.to_string().contains("/test/file.md"));
    }

    #[test]
    fn test_file_error_user_message() {
        let err = FileError::PermissionDenied {
            path: PathBuf::from("/test/file.md"),
        };
        let msg = err.user_message();
        assert!(msg.contains("permission"));
    }

    #[test]
    fn test_app_error_from_file_error() {
        let file_err = FileError::NotFound(PathBuf::from("/test.md"));
        let app_err: AppError = file_err.into();
        assert!(matches!(app_err, AppError::FileIO(_)));
    }
}
