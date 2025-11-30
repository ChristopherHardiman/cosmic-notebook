//! File handler module for Cosmic Notebook
//!
//! Handles all file system operations including:
//! - Reading and writing files with encoding detection
//! - Atomic save operations for data safety
//! - File watching for external changes
//! - Backup and recovery system
//! - Directory scanning for sidebar

pub mod io;
pub mod scanner;
pub mod watcher;
pub mod recovery;

pub use io::*;
pub use scanner::*;
pub use watcher::*;
pub use recovery::*;
