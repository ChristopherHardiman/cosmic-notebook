//! UI module for Cosmic Notebook
//!
//! Contains all user interface components including:
//! - Main window layout
//! - Editor widget
//! - Sidebar file browser
//! - Tab bar
//! - Status bar
//! - Find bar
//! - Dialogs and modals

mod find_bar;
mod main_window;
mod sidebar;
mod status_bar;
mod tab_bar;

use crate::message::Message;
use crate::state::{AppState, DocumentId};
use cosmic::widget::text_editor;
use cosmic::Element;
use std::collections::HashMap;

pub use find_bar::{build_find_bar, FindBarState};
pub use sidebar::*;
pub use status_bar::{build_status_info, StatusBar, StatusBarInfo};
pub use tab_bar::{TabBar, TabContextAction, TabInfo};

/// Build the main application view
pub fn view<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
) -> Element<'a, Message> {
    main_window::view(state, editor_contents)
}
