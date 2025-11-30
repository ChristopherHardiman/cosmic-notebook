//! Main window layout and composition
//!
//! Handles the overall window structure including sidebar, editor area,
//! tab bar, find bar, and status bar arrangement.

use crate::config::ViewMode;
use crate::message::{EditorMessage, Message};
use crate::state::{AppState, DocumentId};
use crate::ui::find_bar::{build_find_bar, FindBarState};
use cosmic::iced::Length;
use cosmic::widget::{container, text, text_editor, Column, Row};
use cosmic::Element;
use std::collections::HashMap;

/// Build the main window view
pub fn view<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
) -> Element<'a, Message> {
    match state.view_mode {
        ViewMode::Edit => view_edit_mode(state, editor_contents),
        ViewMode::Preview => view_preview_mode(state),
        ViewMode::Split => view_split_mode(state, editor_contents),
    }
}

/// Standard edit mode with sidebar, tabs, editor, and status bar
fn view_edit_mode<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
) -> Element<'a, Message> {
    let mut main_row = Row::new();

    // Sidebar (if visible)
    if state.sidebar.visible {
        let sidebar = build_sidebar_simple(state);
        main_row = main_row.push(
            container(sidebar)
                .width(Length::Fixed(state.sidebar.width as f32))
                .height(Length::Fill),
        );
    }

    // Editor area (tabs + editor + status)
    let editor_area = build_editor_area(state, editor_contents);
    main_row = main_row.push(editor_area);

    container(main_row)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Preview mode (for viewing rendered markdown)
fn view_preview_mode(state: &AppState) -> Element<'_, Message> {
    // Get content as owned data
    let preview_text = state
        .active_document()
        .map(|doc| doc.content_str())
        .unwrap_or_else(|| "No document to preview".to_string());

    let status_text = build_status_text(state);

    Column::new()
        .push(
            container(text(preview_text))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
        )
        .push(
            container(text(status_text).size(12))
                .width(Length::Fill)
                .padding([4, 12]),
        )
        .into()
}

/// Split mode with editor and preview side by side
fn view_split_mode<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
) -> Element<'a, Message> {
    let doc_content = state
        .active_document()
        .map(|doc| doc.content_str())
        .unwrap_or_default();

    let status_text = build_status_text(state);

    // Editor side
    let editor_view = if let Some(doc_id) = state.active_document {
        if let Some(content) = editor_contents.get(&doc_id) {
            build_text_editor(doc_id, content)
        } else {
            text("No editor content").into()
        }
    } else {
        text("No document selected").into()
    };
    
    // Preview side
    let preview_content = text(doc_content);

    let split_view = Row::new()
        .push(
            container(editor_view)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(8),
        )
        .push(
            container(preview_content)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(8),
        );

    Column::new()
        .push(split_view)
        .push(
            container(text(status_text).size(12))
                .width(Length::Fill)
                .padding([4, 12]),
        )
        .into()
}

/// Build distraction-free mode view
#[allow(dead_code)]
pub fn view_distraction_free<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
    current_title: &str,
    is_modified: bool,
) -> Element<'a, Message> {
    // Minimal header with document title
    let title_display = if is_modified {
        format!("● {}", current_title)
    } else {
        current_title.to_string()
    };

    let cursor_info = state
        .active_document()
        .map(|doc| {
            format!(
                "Ln {}, Col {}",
                doc.editor_state.cursor.line + 1,
                doc.editor_state.cursor.column + 1
            )
        })
        .unwrap_or_default();

    // Editor view
    let editor_view: Element<'a, Message> = if let Some(doc_id) = state.active_document {
        if let Some(content) = editor_contents.get(&doc_id) {
            build_text_editor(doc_id, content)
        } else {
            text("No editor content").into()
        }
    } else {
        text("No document selected").into()
    };

    Column::new()
        .push(
            container(text(title_display).size(12))
                .width(Length::Fill)
                .padding([4, 8])
                .center_x(Length::Fill),
        )
        .push(
            container(editor_view)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([8, 80]), // Wide margins for focus
        )
        .push(
            container(text(cursor_info).size(11))
                .width(Length::Fill)
                .padding([2, 8])
                .center_x(Length::Fill),
        )
        .into()
}

/// Build a simple sidebar view
fn build_sidebar_simple(state: &AppState) -> Element<'_, Message> {
    use cosmic::widget::divider;
    
    let entries_count = state.sidebar.entries.len();
    
    // Header with folder name
    let header = if let Some(root) = state.sidebar.root.as_ref() {
        let root_name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Files".to_string());
        
        Column::new()
            .push(text(root_name).size(14))
            .push(text(format!("{} items", entries_count)).size(12))
            .spacing(4)
    } else {
        // No folder open - show prompt
        Column::new()
            .push(text("No folder open").size(14))
            .push(text("Use Ctrl+Shift+O to").size(11))
            .push(text("open a folder").size(11))
            .spacing(4)
    };

    // File list content
    let file_list_text = if entries_count > 0 {
        "Files will appear here"
    } else if state.sidebar.root.is_some() {
        "Folder is empty"
    } else {
        ""
    };

    let content = Column::new()
        .push(
            container(header)
                .width(Length::Fill)
                .padding(8)
        )
        .push(divider::horizontal::default())
        .push(
            container(text(file_list_text).size(11))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(8)
        )
        .spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .class(cosmic::theme::Container::Card)
        .into()
}

/// Build a simple editor area with the text_editor widget
fn build_editor_area<'a>(
    state: &'a AppState,
    editor_contents: &'a HashMap<DocumentId, text_editor::Content>,
) -> Element<'a, Message> {
    // Tab bar info
    let tab_info = build_tab_bar_text(state);

    // Status bar info
    let status_text = build_status_text(state);

    let mut column = Column::new();

    // Tab bar
    column = column.push(
        container(text(tab_info).size(13))
            .width(Length::Fill)
            .padding([6, 12]),
    );

    // Find bar (if open)
    if state.find_dialog_open {
        let find_result_count = state
            .active_document()
            .map(|doc| doc.editor_state.find_results.len())
            .unwrap_or(0);
        let current_find_result = state
            .active_document()
            .and_then(|doc| doc.editor_state.current_find_index.map(|i| i + 1));

        let find_state = FindBarState {
            is_open: true,
            show_replace: state.find_replace_open,
            query: &state.find_query,
            replace_text: &state.replace_text,
            case_sensitive: state.find_case_sensitive,
            whole_word: state.find_whole_word,
            use_regex: state.find_use_regex,
            result_count: find_result_count,
            current_result: current_find_result,
        };
        column = column.push(build_find_bar(&find_state));
    }

    // Editor content
    if state.documents.is_empty() {
        // Welcome screen
        let welcome = Column::new()
            .push(text("Welcome to Cosmic Notebook").size(24))
            .push(text(""))
            .push(text("Press Ctrl+N to create a new file"))
            .push(text("Press Ctrl+O to open a file"))
            .spacing(8);

        column = column.push(
            container(welcome)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        );
    } else if let Some(doc_id) = state.active_document {
        // Show interactive text editor
        if let Some(content) = editor_contents.get(&doc_id) {
            let editor_widget = build_text_editor(doc_id, content);
            column = column.push(
                container(editor_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(8),
            );
        } else {
            // Fallback if editor content not found
            column = column.push(
                container(text("Loading editor..."))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            );
        }
    }

    // Status bar
    column = column.push(
        container(text(status_text).size(12))
            .width(Length::Fill)
            .padding([4, 12]),
    );

    container(column)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build the text editor widget
fn build_text_editor<'a>(
    doc_id: DocumentId,
    content: &'a text_editor::Content,
) -> Element<'a, Message> {
    text_editor(content)
        .on_action(move |action| {
            Message::Editor(EditorMessage::TextEditorAction {
                document_id: doc_id,
                action,
            })
        })
        .height(Length::Fill)
        .padding(10)
        .into()
}

/// Build tab bar text representation
fn build_tab_bar_text(state: &AppState) -> String {
    if state.tabs.tabs.is_empty() {
        return "No documents open".to_string();
    }

    let tabs: Vec<String> = state
        .tabs
        .tabs
        .iter()
        .map(|tab_entry| {
            let doc = state.get_document(tab_entry.document_id);
            let (name, modified) = doc
                .map(|d| (d.display_name.clone(), d.modified))
                .unwrap_or(("?".to_string(), false));

            let prefix = if Some(tab_entry.document_id) == state.active_document {
                "▸ "
            } else {
                "  "
            };
            let suffix = if modified { " ●" } else { "" };

            format!("{}{}{}", prefix, name, suffix)
        })
        .collect();

    tabs.join("  |  ")
}

/// Build status bar text
fn build_status_text(state: &AppState) -> String {
    match state.active_document() {
        Some(doc) => {
            let cursor = &doc.editor_state.cursor;
            let lines = doc.line_count();
            let chars = doc.char_count();
            let modified = if doc.modified { " ●" } else { "" };

            format!(
                "Ln {}, Col {}  |  {} lines, {} chars  |  UTF-8  |  LF  |  Markdown{}",
                cursor.line + 1,
                cursor.column + 1,
                lines,
                chars,
                modified
            )
        }
        None => "Ready".to_string(),
    }
}
