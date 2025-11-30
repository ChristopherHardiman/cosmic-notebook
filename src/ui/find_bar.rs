//! Find and Replace bar component
//!
//! Provides a search bar UI for finding and replacing text in the editor.

use crate::message::{Message, SearchMessage};
use cosmic::iced::Length;
use cosmic::widget::{button, container, row, text, text_input, toggler, Column, Row};
use cosmic::Element;

/// Find bar state from AppState
pub struct FindBarState<'a> {
    pub is_open: bool,
    pub show_replace: bool,
    pub query: &'a str,
    pub replace_text: &'a str,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub result_count: usize,
    pub current_result: Option<usize>,
}

/// Build the find bar widget
pub fn build_find_bar<'a>(state: &FindBarState<'a>) -> Element<'a, Message> {
    if !state.is_open {
        return container(Column::new()).height(Length::Fixed(0.0)).into();
    }

    let mut content = Column::new().spacing(4).padding(8);

    // Find row
    let find_row = build_find_row(state);
    content = content.push(find_row);

    // Replace row (if enabled)
    if state.show_replace {
        let replace_row = build_replace_row(state);
        content = content.push(replace_row);
    }

    container(content)
        .width(Length::Fill)
        .class(cosmic::theme::Container::Card)
        .into()
}

/// Build the find input row
fn build_find_row<'a>(state: &FindBarState<'a>) -> Element<'a, Message> {
    // Find input
    let find_input = text_input("Find...", state.query)
        .on_input(|s| Message::Search(SearchMessage::UpdateQuery(s)))
        .on_submit(|_| Message::Search(SearchMessage::FindNext))
        .width(Length::Fixed(250.0));

    // Result count display
    let result_text = if state.result_count > 0 {
        if let Some(current) = state.current_result {
            format!("{} of {}", current, state.result_count)
        } else {
            format!("{} results", state.result_count)
        }
    } else if !state.query.is_empty() {
        "No results".to_string()
    } else {
        String::new()
    };

    // Navigation buttons
    let prev_button = button::icon(cosmic::widget::icon::from_name("go-up-symbolic"))
        .on_press(Message::Search(SearchMessage::FindPrevious))
        .padding(4);

    let next_button = button::icon(cosmic::widget::icon::from_name("go-down-symbolic"))
        .on_press(Message::Search(SearchMessage::FindNext))
        .padding(4);

    // Option toggles with shorter labels
    let case_toggle = button::text(if state.case_sensitive { "Aa" } else { "Aa" })
        .on_press(Message::Search(SearchMessage::ToggleCaseSensitive))
        .class(if state.case_sensitive {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Standard
        })
        .padding([4, 8]);

    let word_toggle = button::text("W")
        .on_press(Message::Search(SearchMessage::ToggleWholeWord))
        .class(if state.whole_word {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Standard
        })
        .padding([4, 8]);

    let regex_toggle = button::text(".*")
        .on_press(Message::Search(SearchMessage::ToggleRegex))
        .class(if state.use_regex {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Standard
        })
        .padding([4, 8]);

    // Close button
    let close_button = button::icon(cosmic::widget::icon::from_name("window-close-symbolic"))
        .on_press(Message::Search(SearchMessage::CloseFind))
        .padding(4);

    Row::new()
        .push(find_input)
        .push(text(result_text).size(12))
        .push(prev_button)
        .push(next_button)
        .push(container(Row::new()).width(Length::Fixed(16.0))) // Spacer
        .push(case_toggle)
        .push(word_toggle)
        .push(regex_toggle)
        .push(container(Row::new()).width(Length::Fill)) // Flex spacer
        .push(close_button)
        .spacing(4)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

/// Build the replace input row
fn build_replace_row<'a>(state: &FindBarState<'a>) -> Element<'a, Message> {
    // Replace input
    let replace_input = text_input("Replace with...", state.replace_text)
        .on_input(|s| Message::Search(SearchMessage::UpdateReplaceText(s)))
        .width(Length::Fixed(250.0));

    // Replace buttons
    let replace_button = button::text("Replace")
        .on_press(Message::Search(SearchMessage::Replace))
        .padding([4, 8]);

    let replace_all_button = button::text("Replace All")
        .on_press(Message::Search(SearchMessage::ReplaceAll))
        .padding([4, 8]);

    Row::new()
        .push(replace_input)
        .push(replace_button)
        .push(replace_all_button)
        .spacing(4)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}
