//! State management module for Cosmic Notebook
//!
//! This module contains all application state types organized by concern:
//! - `app_state`: Root application state container
//! - `editor_state`: Per-document editor state (cursor, selection, undo)
//! - `tab_state`: Tab bar management
//! - `sidebar_state`: File browser state
//! - `session_state`: Persistent session data

mod app_state;
mod editor_state;
mod session_state;
mod sidebar_state;
mod tab_state;

pub use app_state::*;
pub use editor_state::*;
pub use session_state::*;
pub use sidebar_state::*;
pub use tab_state::*;
