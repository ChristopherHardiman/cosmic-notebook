//! Sidebar widget for file browser
//!
//! Provides a file tree view with:
//! - Search/filter input
//! - Expandable folders
//! - File icons
//! - Click handlers
//! - Keyboard navigation

use crate::message::{Message, FileMessage};
use crate::state::SidebarState;
use cosmic::iced::Length;
use cosmic::widget::{container, scrollable, text, Column, Row};
use cosmic::Element;

/// Indentation per depth level in pixels
const INDENT_PER_LEVEL: u16 = 16;

/// Row height for file entries
const ROW_HEIGHT: u16 = 28;

/// Build the sidebar view
pub fn view_sidebar<'a>(state: &'a SidebarState) -> Element<'a, Message> {
    if !state.visible {
        return container(Column::new()).width(Length::Shrink).into();
    }

    let width = state.width as u16;

    // Build the content
    let content = Column::new()
        .push(view_search_bar(state))
        .push(view_file_list(state))
        .spacing(4)
        .padding(4);

    container(content)
        .width(Length::Fixed(width as f32))
        .height(Length::Fill)
        .into()
}

/// Build the search/filter input bar
fn view_search_bar<'a>(state: &'a SidebarState) -> Element<'a, Message> {
    // Simple text display for the filter (placeholder for now)
    let filter_display = if state.filter_text.is_empty() {
        "🔍 Search files...".to_string()
    } else {
        format!("🔍 {}", state.filter_text)
    };
    
    container(text(filter_display).size(14))
        .width(Length::Fill)
        .padding(8)
        .into()
}

/// Build the file list view
fn view_file_list<'a>(state: &'a SidebarState) -> Element<'a, Message> {
    // Handle different states
    if state.is_scanning {
        return container(text("Scanning...").size(14))
            .width(Length::Fill)
            .padding(16)
            .into();
    }

    if let Some(ref error) = state.error_message {
        return container(text(error).size(14))
            .width(Length::Fill)
            .padding(16)
            .into();
    }

    if state.root.is_none() {
        return view_no_folder_open();
    }

    let visible = state.visible_entries();

    if visible.is_empty() {
        return container(
            text(if state.filter_text.is_empty() {
                "No files found"
            } else {
                "No matching files"
            })
            .size(14),
        )
        .width(Length::Fill)
        .padding(16)
        .into();
    }

    // Build file list
    let mut items = Column::new().spacing(2);

    for (index, entry) in visible {
        let is_focused = state.focused_index == Some(index);
        let is_selected = state.selected_path.as_ref() == Some(&entry.path);

        items = items.push(view_file_entry(entry, state, is_focused, is_selected));
    }

    scrollable(items)
        .height(Length::Fill)
        .into()
}

/// View when no folder is open
fn view_no_folder_open<'a>() -> Element<'a, Message> {
    container(
        Column::new()
            .push(text("No folder open").size(16))
            .push(text("Open a folder to browse files").size(12))
            .spacing(4)
            .align_x(cosmic::iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(16)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Build a single file entry row
fn view_file_entry<'a>(
    entry: &'a crate::state::FileEntry,
    state: &'a SidebarState,
    is_focused: bool,
    is_selected: bool,
) -> Element<'a, Message> {
    let indent = (entry.depth as u16) * INDENT_PER_LEVEL;

    // Icon based on type
    let icon = if entry.is_directory {
        if state.is_expanded(&entry.path) {
            "📂" // Open folder
        } else {
            "📁" // Closed folder
        }
    } else if entry.is_markdown() {
        "📝" // Markdown file
    } else {
        "📄" // Generic file
    };

    // Build the row content
    let row_content = Row::new()
        .push(text(icon).size(14))
        .push(text(&entry.name).size(14))
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center)
        .padding([4, 8]);

    // Wrap with indentation
    let indented = Row::new()
        .push(cosmic::widget::horizontal_space().width(Length::Fixed(indent as f32)))
        .push(row_content);

    // Create clickable button
    let path = entry.path.clone();
    let is_dir = entry.is_directory;

    // Use cosmic button for clickability
    let clickable = cosmic::widget::button::custom(indented)
        .class(if is_selected {
            cosmic::theme::Button::Suggested
        } else if is_focused {
            cosmic::theme::Button::Standard
        } else {
            cosmic::theme::Button::Text
        })
        .on_press(if is_dir {
            Message::Internal(crate::message::InternalMessage::DirectoryScanComplete(vec![]))
            // TODO: Proper message for folder toggle
        } else {
            Message::File(FileMessage::OpenPath(path))
        })
        .width(Length::Fill)
        .padding(0);

    container(clickable)
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT as f32))
        .into()
}

/// Build the sidebar header with folder name and actions
pub fn view_sidebar_header<'a>(state: &'a SidebarState) -> Element<'a, Message> {
    let title = state
        .root
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Files".to_string());

    Row::new()
        .push(text(title).size(14))
        .push(cosmic::widget::horizontal_space())
        .push(
            cosmic::widget::button::text("⟳")
                .class(cosmic::theme::Button::Text)
                .padding([4, 8])
                .on_press(Message::Internal(
                    crate::message::InternalMessage::DirectoryScanComplete(vec![]),
                )),
        )
        .spacing(8)
        .padding(8)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}
