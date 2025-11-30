//! Menu bar and keyboard shortcut handling
//!
//! Provides the application menu bar and keyboard shortcut definitions.

use cosmic::iced::{event, keyboard, Event, Subscription};
use cosmic::iced_futures::event::listen_raw;
use cosmic::widget::menu::{KeyBind, Item};
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::menu::key_bind::Modifier;
use cosmic::iced::keyboard::Key;
use std::collections::HashMap;

use crate::message::{
    ClipboardMessage, DialogMessage, EditorMessage, FileMessage, Message, SearchMessage,
    SystemMessage, ViewMessage,
};

/// Menu actions that can be triggered from the menu bar or keyboard shortcuts
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    // File actions
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    SaveAll,
    CloseFile,
    CloseAll,
    Quit,

    // Edit actions
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    FindReplace,

    // View actions
    ToggleSidebar,
    ToggleViewMode,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFullscreen,

    // Help actions
    CommandPalette,
    About,
}

impl MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Self::Message {
        self.to_message()
    }
}

impl Action {
    /// Convert action to application message
    pub fn to_message(self) -> Message {
        match self {
            // File
            Action::NewFile => Message::File(FileMessage::New),
            Action::OpenFile => Message::File(FileMessage::Open),
            Action::Save => Message::File(FileMessage::Save),
            Action::SaveAs => Message::File(FileMessage::SaveAs),
            Action::SaveAll => Message::File(FileMessage::SaveAll),
            Action::CloseFile => Message::File(FileMessage::Close),
            Action::CloseAll => Message::File(FileMessage::CloseAll),
            Action::Quit => Message::System(SystemMessage::CloseRequested),

            // Edit
            Action::Undo => Message::Editor(EditorMessage::Undo),
            Action::Redo => Message::Editor(EditorMessage::Redo),
            Action::Cut => Message::Clipboard(ClipboardMessage::Cut),
            Action::Copy => Message::Clipboard(ClipboardMessage::Copy),
            Action::Paste => Message::Clipboard(ClipboardMessage::Paste),
            Action::SelectAll => Message::Editor(EditorMessage::SelectAll),
            Action::Find => Message::Search(SearchMessage::OpenFind),
            Action::FindReplace => Message::Search(SearchMessage::OpenFindReplace),

            // View
            Action::ToggleSidebar => Message::View(ViewMessage::ToggleSidebar),
            Action::ToggleViewMode => Message::View(ViewMessage::ToggleViewMode),
            Action::ZoomIn => Message::View(ViewMessage::ZoomIn),
            Action::ZoomOut => Message::View(ViewMessage::ZoomOut),
            Action::ZoomReset => Message::View(ViewMessage::ZoomReset),
            Action::ToggleFullscreen => Message::View(ViewMessage::ToggleFullscreen),

            // Help
            Action::CommandPalette => Message::Dialog(DialogMessage::OpenCommandPalette),
            Action::About => Message::Dialog(DialogMessage::ShowAbout),
        }
    }
}

/// Create default keyboard shortcuts
pub fn key_binds() -> HashMap<KeyBind, Action> {
    let mut binds = HashMap::new();

    // File shortcuts
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("n".into()),
        },
        Action::NewFile,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("o".into()),
        },
        Action::OpenFile,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("s".into()),
        },
        Action::Save,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("s".into()),
        },
        Action::SaveAs,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("w".into()),
        },
        Action::CloseFile,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("q".into()),
        },
        Action::Quit,
    );

    // Edit shortcuts
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("z".into()),
        },
        Action::Undo,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("y".into()),
        },
        Action::Redo,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("z".into()),
        },
        Action::Redo,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("x".into()),
        },
        Action::Cut,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("c".into()),
        },
        Action::Copy,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("v".into()),
        },
        Action::Paste,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("a".into()),
        },
        Action::SelectAll,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("f".into()),
        },
        Action::Find,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("h".into()),
        },
        Action::FindReplace,
    );

    // View shortcuts
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("b".into()),
        },
        Action::ToggleSidebar,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("e".into()),
        },
        Action::ToggleViewMode,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("=".into()),
        },
        Action::ZoomIn,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("-".into()),
        },
        Action::ZoomOut,
    );
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: Key::Character("0".into()),
        },
        Action::ZoomReset,
    );

    // Help shortcuts
    binds.insert(
        KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Key::Character("p".into()),
        },
        Action::CommandPalette,
    );

    binds
}

/// Type alias for menu items with our action type
pub type MenuItems = Vec<(&'static str, Vec<Item<Action, &'static str>>)>;

/// Create menu bar items
pub fn menu_items(_key_binds: &HashMap<KeyBind, Action>) -> MenuItems {
    vec![
        (
            "File",
            vec![
                Item::Button("New", None, Action::NewFile),
                Item::Button("Open", None, Action::OpenFile),
                Item::Divider,
                Item::Button("Save", None, Action::Save),
                Item::Button("Save As...", None, Action::SaveAs),
                Item::Button("Save All", None, Action::SaveAll),
                Item::Divider,
                Item::Button("Close", None, Action::CloseFile),
                Item::Button("Close All", None, Action::CloseAll),
                Item::Divider,
                Item::Button("Quit", None, Action::Quit),
            ],
        ),
        (
            "Edit",
            vec![
                Item::Button("Undo", None, Action::Undo),
                Item::Button("Redo", None, Action::Redo),
                Item::Divider,
                Item::Button("Cut", None, Action::Cut),
                Item::Button("Copy", None, Action::Copy),
                Item::Button("Paste", None, Action::Paste),
                Item::Divider,
                Item::Button("Select All", None, Action::SelectAll),
                Item::Divider,
                Item::Button("Find", None, Action::Find),
                Item::Button("Find & Replace", None, Action::FindReplace),
            ],
        ),
        (
            "View",
            vec![
                Item::Button("Toggle Sidebar", None, Action::ToggleSidebar),
                Item::Button("Toggle Preview", None, Action::ToggleViewMode),
                Item::Divider,
                Item::Button("Zoom In", None, Action::ZoomIn),
                Item::Button("Zoom Out", None, Action::ZoomOut),
                Item::Button("Reset Zoom", None, Action::ZoomReset),
                Item::Divider,
                Item::Button("Fullscreen", None, Action::ToggleFullscreen),
            ],
        ),
        (
            "Help",
            vec![
                Item::Button("Command Palette", None, Action::CommandPalette),
                Item::Divider,
                Item::Button("About", None, Action::About),
            ],
        ),
    ]
}

/// Keyboard shortcuts subscription
/// 
/// Listens for keyboard events and matches against defined shortcuts.
pub fn keyboard_shortcuts_subscription() -> Subscription<Message> {
    listen_raw(|event, status, _| {
        // Only process if event wasn't already handled
        if event::Status::Ignored != status {
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                ..
            }) => {
                // Handle character keys with Ctrl modifier
                if let Key::Character(ref c) = key {
                    let c_lower = c.to_lowercase();
                    
                    if modifiers.control() && !modifiers.alt() {
                        // Ctrl+Shift combinations
                        if modifiers.shift() {
                            match c_lower.as_str() {
                                "s" => return Some(Action::SaveAs.to_message()),
                                "z" => return Some(Action::Redo.to_message()),
                                "p" => return Some(Action::CommandPalette.to_message()),
                                _ => {}
                            }
                        } else {
                            // Plain Ctrl combinations
                            match c_lower.as_str() {
                                "n" => return Some(Action::NewFile.to_message()),
                                "o" => return Some(Action::OpenFile.to_message()),
                                "s" => return Some(Action::Save.to_message()),
                                "w" => return Some(Action::CloseFile.to_message()),
                                "q" => return Some(Action::Quit.to_message()),
                                "z" => return Some(Action::Undo.to_message()),
                                "y" => return Some(Action::Redo.to_message()),
                                "x" => return Some(Action::Cut.to_message()),
                                "c" => return Some(Action::Copy.to_message()),
                                "v" => return Some(Action::Paste.to_message()),
                                "a" => return Some(Action::SelectAll.to_message()),
                                "f" => return Some(Action::Find.to_message()),
                                "h" => return Some(Action::FindReplace.to_message()),
                                "b" => return Some(Action::ToggleSidebar.to_message()),
                                "e" => return Some(Action::ToggleViewMode.to_message()),
                                "=" | "+" => return Some(Action::ZoomIn.to_message()),
                                "-" => return Some(Action::ZoomOut.to_message()),
                                "0" => return Some(Action::ZoomReset.to_message()),
                                _ => {}
                            }
                        }
                    }
                }
                
                // Handle F keys
                if let Key::Named(keyboard::key::Named::F11) = key {
                    return Some(Action::ToggleFullscreen.to_message());
                }
            }
            _ => {}
        }

        None
    })
}
