//! Status bar UI component
//!
//! Displays document information including cursor position,
//! line/column, word count, file encoding, and line endings.

use cosmic::iced::Length;
use cosmic::widget::{container, horizontal_space, row, text, Row};
use cosmic::Element;

use crate::editor::buffer::LineEnding;
use crate::message::Message;
use crate::state::CursorPosition;

/// Information to display in the status bar
#[derive(Debug, Clone, Default)]
pub struct StatusBarInfo {
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Whether there's an active selection
    pub has_selection: bool,
    /// Number of selected characters (if selection)
    pub selection_chars: Option<usize>,
    /// Number of selected lines (if selection)
    pub selection_lines: Option<usize>,
    /// Total line count
    pub line_count: usize,
    /// Total character count
    pub char_count: usize,
    /// Word count
    pub word_count: usize,
    /// File encoding
    pub encoding: String,
    /// Line ending style
    pub line_ending: LineEnding,
    /// Language/file type
    pub language: String,
    /// Whether document is modified
    pub is_modified: bool,
    /// Read-only status
    pub is_readonly: bool,
}

impl StatusBarInfo {
    /// Create a new status bar info
    pub fn new() -> Self {
        Self {
            encoding: "UTF-8".to_string(),
            language: "Markdown".to_string(),
            ..Default::default()
        }
    }

    /// Format cursor position for display
    pub fn cursor_display(&self) -> String {
        format!("Ln {}, Col {}", self.cursor.line + 1, self.cursor.column + 1)
    }

    /// Format selection info for display
    pub fn selection_display(&self) -> Option<String> {
        if self.has_selection {
            match (self.selection_chars, self.selection_lines) {
                (Some(chars), Some(lines)) if lines > 1 => {
                    Some(format!("{} chars, {} lines selected", chars, lines))
                }
                (Some(chars), _) => Some(format!("{} chars selected", chars)),
                _ => Some("Selection".to_string()),
            }
        } else {
            None
        }
    }

    /// Format document statistics
    pub fn stats_display(&self) -> String {
        format!(
            "{} lines, {} words",
            self.line_count, self.word_count
        )
    }
}

/// Status bar widget
pub struct StatusBar;

impl StatusBar {
    /// Create the status bar view
    pub fn view<'a>(info: &'a StatusBarInfo) -> Element<'a, Message> {
        let mut status_row: Row<'_, Message> = Row::new().spacing(16);

        // Left section: cursor position and selection
        let cursor_text = text(info.cursor_display()).size(12);
        status_row = status_row.push(cursor_text);

        if let Some(sel_info) = info.selection_display() {
            status_row = status_row.push(text(sel_info).size(12));
        }

        // Spacer
        status_row = status_row.push(horizontal_space());

        // Right section: document info
        // Modified indicator
        if info.is_modified {
            status_row = status_row.push(text("●").size(12));
        }

        // Read-only indicator
        if info.is_readonly {
            status_row = status_row.push(text("[Read Only]").size(12));
        }

        // Document statistics
        status_row = status_row.push(text(info.stats_display()).size(12));

        // Encoding
        status_row = status_row.push(text(&info.encoding).size(12));

        // Line ending
        status_row = status_row.push(text(info.line_ending.display_name()).size(12));

        // Language
        status_row = status_row.push(text(&info.language).size(12));

        container(status_row)
            .width(Length::Fill)
            .padding([4, 12])
            .into()
    }

    /// Create a minimal status bar for distraction-free mode
    pub fn view_minimal<'a>(info: &'a StatusBarInfo) -> Element<'a, Message> {
        let mut status_row: Row<'_, Message> = Row::new().spacing(16);

        // Just show cursor position and modification status
        status_row = status_row.push(text(info.cursor_display()).size(11));

        if info.is_modified {
            status_row = status_row.push(text("●").size(11));
        }

        container(status_row)
            .width(Length::Fill)
            .padding([2, 8])
            .center_x(Length::Fill)
            .into()
    }

    /// Create status bar for find/replace mode
    pub fn view_with_find<'a>(
        info: &'a StatusBarInfo,
        find_count: usize,
        current_match: Option<usize>,
    ) -> Element<'a, Message> {
        let mut status_row: Row<'_, Message> = Row::new().spacing(16);

        // Find results
        let find_text = if find_count > 0 {
            match current_match {
                Some(idx) => format!("{} of {} matches", idx + 1, find_count),
                None => format!("{} matches", find_count),
            }
        } else {
            "No matches".to_string()
        };
        status_row = status_row.push(text(find_text).size(12));

        // Spacer
        status_row = status_row.push(horizontal_space());

        // Cursor position
        status_row = status_row.push(text(info.cursor_display()).size(12));

        container(status_row)
            .width(Length::Fill)
            .padding([4, 12])
            .into()
    }
}

/// Build StatusBarInfo from an Editor
pub fn build_status_info(
    editor: &crate::editor::Editor,
    language: &str,
    is_readonly: bool,
) -> StatusBarInfo {
    let cursor = editor.cursor();
    let has_selection = editor.has_selection();

    let (selection_chars, selection_lines) = if has_selection {
        if let Some(text) = editor.selected_text() {
            let chars = text.chars().count();
            let lines = text.lines().count();
            (Some(chars), Some(lines))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    StatusBarInfo {
        cursor,
        has_selection,
        selection_chars,
        selection_lines,
        line_count: editor.line_count(),
        char_count: editor.char_count(),
        word_count: editor.buffer().word_count(),
        encoding: "UTF-8".to_string(),
        line_ending: editor.buffer().line_ending(),
        language: language.to_string(),
        is_modified: editor.is_modified(),
        is_readonly,
    }
}
