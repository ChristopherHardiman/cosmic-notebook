//! Tab bar UI component
//!
//! Displays open documents as tabs with close buttons and
//! indicates modified documents.

use cosmic::iced::Length;
use cosmic::widget::{button, container, horizontal_space, row, text, Row};
use cosmic::Element;

use crate::message::{FileMessage, Message, TabMessage};
use crate::state::DocumentId;

/// Information about a single tab
#[derive(Debug, Clone)]
pub struct TabInfo {
    /// Document ID
    pub id: DocumentId,
    /// Display title (filename or "Untitled")
    pub title: String,
    /// Whether the document has unsaved changes
    pub is_modified: bool,
    /// Full file path (if saved)
    pub path: Option<String>,
}

impl TabInfo {
    /// Create a new tab info
    pub fn new(id: DocumentId, title: String, is_modified: bool, path: Option<String>) -> Self {
        Self {
            id,
            title,
            is_modified,
            path,
        }
    }

    /// Get display title with modification indicator
    pub fn display_title(&self) -> String {
        if self.is_modified {
            format!("● {}", self.title)
        } else {
            self.title.clone()
        }
    }
}

/// Tab bar widget
pub struct TabBar;

impl TabBar {
    /// Create the tab bar view
    pub fn view<'a>(
        tabs: &'a [TabInfo],
        active_tab: Option<DocumentId>,
    ) -> Element<'a, Message> {
        if tabs.is_empty() {
            return container(text("No documents open"))
                .width(Length::Fill)
                .padding(8)
                .into();
        }

        let mut tab_row: Row<'_, Message> = Row::new();

        for tab in tabs {
            let is_active = active_tab.map_or(false, |id| id == tab.id);
            let tab_element = Self::render_tab(tab, is_active);
            tab_row = tab_row.push(tab_element);
        }

        // Add spacer to push tabs to the left
        tab_row = tab_row.push(horizontal_space());

        container(tab_row)
            .width(Length::Fill)
            .padding([4, 8])
            .into()
    }

    /// Render a single tab
    fn render_tab(tab: &TabInfo, is_active: bool) -> Element<'static, Message> {
        let title = tab.display_title();
        let tab_id = tab.id;

        // Tab content: title and close button
        let tab_content = row::with_capacity(2)
            .push(text(title).size(13))
            .push(
                button::text("×")
                    .on_press(Message::File(FileMessage::CloseDocument(tab_id)))
                    .padding([2, 6]),
            )
            .spacing(4)
            .align_y(cosmic::iced::Alignment::Center);

        // Tab button
        let tab_button = if is_active {
            button::custom(tab_content)
                .on_press(Message::Tab(TabMessage::Select(tab_id)))
                .padding([6, 12])
        } else {
            button::custom(tab_content)
                .on_press(Message::Tab(TabMessage::Select(tab_id)))
                .padding([6, 12])
        };

        container(tab_button)
            .padding([0, 2])
            .into()
    }

    /// Create a minimal tab bar for distraction-free mode
    pub fn view_minimal<'a>(
        current_title: &'a str,
        is_modified: bool,
    ) -> Element<'a, Message> {
        let title = if is_modified {
            format!("● {}", current_title)
        } else {
            current_title.to_string()
        };

        container(text(title).size(12))
            .width(Length::Fill)
            .padding([4, 8])
            .center_x(Length::Fill)
            .into()
    }
}

/// Tab context menu options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabContextAction {
    /// Close this tab
    Close,
    /// Close all other tabs
    CloseOthers,
    /// Close tabs to the right
    CloseToRight,
    /// Close all tabs
    CloseAll,
    /// Reveal in file browser
    RevealInSidebar,
    /// Copy file path
    CopyPath,
}

impl TabContextAction {
    /// Get display label for the action
    pub fn label(&self) -> &'static str {
        match self {
            Self::Close => "Close",
            Self::CloseOthers => "Close Others",
            Self::CloseToRight => "Close to the Right",
            Self::CloseAll => "Close All",
            Self::RevealInSidebar => "Reveal in Sidebar",
            Self::CopyPath => "Copy Path",
        }
    }
}
