//! Editor widget for rendering text in the UI
//!
//! This module provides the visual representation of the editor,
//! including text rendering, cursor display, and selection highlighting.

use cosmic::iced::Length;
use cosmic::widget::{column, container, row, scrollable, text};
use cosmic::Element;

use super::Editor;
use crate::message::Message;
use crate::state::CursorPosition;

/// Configuration for the editor widget appearance
#[derive(Debug, Clone)]
pub struct EditorWidgetConfig {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Line number width (in characters)
    pub line_number_width: usize,
    /// Tab size (spaces per tab)
    pub tab_size: usize,
    /// Highlight current line
    pub highlight_current_line: bool,
    /// Show whitespace characters
    pub show_whitespace: bool,
    /// Word wrap mode
    pub word_wrap: bool,
}

impl Default for EditorWidgetConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            line_number_width: 4,
            tab_size: 4,
            highlight_current_line: true,
            show_whitespace: false,
            word_wrap: true,
        }
    }
}

/// Editor widget for displaying and interacting with text
pub struct EditorWidget;

impl EditorWidget {
    /// Create an editor view element
    pub fn view<'a>(
        editor: &'a Editor,
        config: &'a EditorWidgetConfig,
    ) -> Element<'a, Message> {
        let cursor = editor.cursor();
        let scroll_line = editor.scroll_line();
        let line_count = editor.line_count();

        // Calculate visible lines (approximate, will be refined by actual viewport)
        let visible_lines = 50; // Default visible lines
        let end_line = (scroll_line + visible_lines).min(line_count);

        // Build the editor content
        let mut content_column = column::with_capacity(end_line - scroll_line);

        for line_idx in scroll_line..end_line {
            let line_element = Self::render_line(editor, line_idx, cursor, config);
            content_column = content_column.push(line_element);
        }

        let editor_content = container(
            scrollable(content_column)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8);

        editor_content.into()
    }

    /// Render a single line with optional line number
    fn render_line<'a>(
        editor: &'a Editor,
        line_idx: usize,
        cursor: CursorPosition,
        config: &'a EditorWidgetConfig,
    ) -> Element<'a, Message> {
        let line_content = editor.get_line(line_idx).unwrap_or_default();
        let is_current_line = line_idx == cursor.line;

        // Prepare display content (replace tabs with spaces)
        let display_content = if config.show_whitespace {
            line_content
                .replace('\t', &"→".repeat(1))
                .replace(' ', "·")
        } else {
            line_content.replace('\t', &" ".repeat(config.tab_size))
        };

        // Empty lines need at least a space for proper height
        let display_content = if display_content.is_empty() {
            " ".to_string()
        } else {
            display_content
        };

        let mut line_row = row::with_capacity(2);

        // Line number (if enabled)
        if config.show_line_numbers {
            let line_num = format!(
                "{:>width$} ",
                line_idx + 1,
                width = config.line_number_width
            );
            let line_num_text = text(line_num).size(14);
            line_row = line_row.push(
                container(line_num_text)
                    .padding([0, 8, 0, 0]),
            );
        }

        // Line content
        let content_text = text(display_content).size(14);

        // Wrap in container, potentially with current line highlighting
        let content_container = if is_current_line && config.highlight_current_line {
            container(content_text)
                .width(Length::Fill)
                .padding([2, 4])
        } else {
            container(content_text)
                .width(Length::Fill)
                .padding([2, 4])
        };

        line_row = line_row.push(content_container);

        container(line_row)
            .width(Length::Fill)
            .into()
    }

    /// Render the cursor indicator (for overlay)
    pub fn cursor_indicator<'a>(cursor: CursorPosition) -> Element<'a, Message> {
        // Simple text-based cursor position indicator
        let cursor_text = format!("Ln {}, Col {}", cursor.line + 1, cursor.column + 1);
        text(cursor_text).size(12).into()
    }

    /// Calculate the character position from screen coordinates
    /// Returns (line, column) if valid
    pub fn screen_to_position(
        _x: f32,
        y: f32,
        scroll_line: usize,
        line_height: f32,
        _char_width: f32,
        _line_number_width: f32,
    ) -> Option<CursorPosition> {
        // Calculate line from y position
        let relative_y = y.max(0.0);
        let line_offset = (relative_y / line_height) as usize;
        let line = scroll_line + line_offset;

        // For now, return column 0 - proper implementation needs font metrics
        Some(CursorPosition::new(line, 0))
    }
}

/// Editor viewport information for scroll calculations
#[derive(Debug, Clone, Default)]
pub struct EditorViewport {
    /// Width of the viewport in pixels
    pub width: f32,
    /// Height of the viewport in pixels
    pub height: f32,
    /// Approximate line height in pixels
    pub line_height: f32,
    /// Approximate character width in pixels (monospace)
    pub char_width: f32,
    /// Number of visible lines
    pub visible_lines: usize,
    /// Number of visible columns
    pub visible_columns: usize,
}

impl EditorViewport {
    /// Create a new viewport with dimensions
    pub fn new(width: f32, height: f32, line_height: f32, char_width: f32) -> Self {
        let visible_lines = (height / line_height).ceil() as usize;
        let visible_columns = (width / char_width).ceil() as usize;

        Self {
            width,
            height,
            line_height,
            char_width,
            visible_lines,
            visible_columns,
        }
    }

    /// Update dimensions
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.visible_lines = (height / self.line_height).ceil() as usize;
        self.visible_columns = (width / self.char_width).ceil() as usize;
    }
}
