//! Main application module implementing the Cosmic Application trait
//!
//! This is the central hub of the application, implementing libCosmic's
//! Application trait for window management and message routing.

use crate::config::{Config, APP_ID};
use crate::file_handler::RecoveryManager;
use crate::menu::{keyboard_shortcuts_subscription, Action as MenuAction};
use crate::message::{
    ClipboardMessage, DialogMessage, EditorMessage, FileMessage, InternalMessage, Message,
    SearchMessage, SystemMessage, TabMessage, ViewMessage,
};
use crate::state::{AppState, Document, DocumentId, SessionState};
use crate::ui;

use cosmic::app::{Core, Task};
use cosmic::iced::window;
use cosmic::widget::menu::KeyBind;
use cosmic::widget::text_editor;
use cosmic::{Application, ApplicationExt, Element};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Helper function to convert line/column to character index in a rope
fn line_col_to_char(rope: &ropey::Rope, line: usize, col: usize) -> Option<usize> {
    if line >= rope.len_lines() {
        return None;
    }
    let line_start = rope.line_to_char(line);
    let line_len = rope.line(line).len_chars();
    // Handle newline character at end of line
    let max_col = if line < rope.len_lines() - 1 {
        line_len.saturating_sub(1) // Exclude newline for non-last lines
    } else {
        line_len
    };
    let clamped_col = col.min(max_col);
    Some(line_start + clamped_col)
}

/// Cosmic Notebook Application
pub struct CosmicNotebook {
    /// libCosmic core reference
    core: Core,

    /// Application state
    pub state: AppState,

    /// User configuration
    pub config: Config,

    /// Session state (for persistence)
    pub session: SessionState,

    /// Text editor contents for each document (keyed by DocumentId)
    pub editor_contents: HashMap<DocumentId, text_editor::Content>,

    /// Recovery manager for autosave
    pub recovery_manager: RecoveryManager,

    /// Pending autosave flag
    autosave_pending: bool,

    /// Initialization complete flag
    initialized: bool,

    /// Keyboard shortcut bindings
    key_binds: HashMap<KeyBind, MenuAction>,
}

/// Application flags passed during initialization
#[derive(Debug, Clone, Default)]
pub struct Flags {
    /// Files to open on startup
    pub files: Vec<PathBuf>,

    /// Working directory
    pub working_dir: Option<PathBuf>,
}

impl Application for CosmicNotebook {
    /// Executor for async tasks
    type Executor = cosmic::executor::Default;

    /// Application flags
    type Flags = Flags;

    /// Application message type
    type Message = Message;

    /// Application ID following reverse-DNS convention
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initialize the application
    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Load configuration
        let config = Config::load().unwrap_or_default();

        // Load session state
        let session = SessionState::load().unwrap_or_default();

        // Create initial state
        let state = AppState::new();

        // Initialize recovery manager
        let recovery_manager = RecoveryManager::new();

        let mut app = Self {
            core,
            state,
            config,
            session,
            editor_contents: HashMap::new(),
            recovery_manager,
            autosave_pending: false,
            initialized: false,
            key_binds: crate::menu::key_binds(),
        };

        // Set window title
        app.set_header_title("Cosmic Notebook".to_string());

        // Collect tasks for opening initial files
        let mut tasks: Vec<Task<Message>> = Vec::new();

        // Open files from command line
        for path in flags.files {
            tasks.push(Task::perform(
                async move { path },
                |path| cosmic::Action::App(Message::File(FileMessage::OpenPath(path))),
            ));
        }

        // Set working directory for sidebar
        if let Some(dir) = flags.working_dir {
            app.state.sidebar.set_root(dir);
        }

        // Mark as initialized after initial setup
        app.initialized = true;

        // Combine tasks
        let task = if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        };

        (app, task)
    }

    /// Handle incoming messages
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::File(msg) => self.handle_file_message(msg),
            Message::Tab(msg) => self.handle_tab_message(msg),
            Message::Editor(msg) => self.handle_editor_message(msg),
            Message::Clipboard(msg) => self.handle_clipboard_message(msg),
            Message::Search(msg) => self.handle_search_message(msg),
            Message::View(msg) => self.handle_view_message(msg),
            Message::Dialog(msg) => self.handle_dialog_message(msg),
            Message::System(msg) => self.handle_system_message(msg),
            Message::Internal(msg) => self.handle_internal_message(msg),
            Message::Surface(_) => Task::none(), // Surface actions are handled by libcosmic
            Message::None => Task::none(),
        }
    }

    /// Render the application view
    fn view(&self) -> Element<'_, Self::Message> {
        // Use the main window view from ui module, passing editor contents
        ui::view(&self.state, &self.editor_contents)
    }

    /// Handle subscription events
    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        use cosmic::iced::time;
        
        let mut subscriptions = vec![
            // Keyboard shortcut subscription
            keyboard_shortcuts_subscription(),
        ];
        
        // Add autosave timer if enabled
        if self.config.files.autosave_enabled && self.initialized {
            let interval = Duration::from_secs(self.config.files.autosave_interval);
            subscriptions.push(
                time::every(interval).map(|_| Message::Internal(InternalMessage::AutosaveTrigger))
            );
        }
        
        cosmic::iced::Subscription::batch(subscriptions)
    }

    /// Elements to show at the start of the header bar (menu bar)
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        use cosmic::widget::menu::Item as MenuItem;
        use cosmic::widget::menu::ItemHeight;
        use cosmic::widget::responsive_menu_bar;
        
        let menu_bar = responsive_menu_bar()
            .item_height(ItemHeight::Dynamic(40))
            .into_element(
                self.core(),
                &self.key_binds,
                cosmic::widget::Id::new("menu-bar"),
                Message::Surface, // Surface action wrapper
                vec![
                    (
                        "File",
                        vec![
                            MenuItem::Button("New", None, MenuAction::NewFile),
                            MenuItem::Button("Open", None, MenuAction::OpenFile),
                            MenuItem::Divider,
                            MenuItem::Button("Save", None, MenuAction::Save),
                            MenuItem::Button("Save As...", None, MenuAction::SaveAs),
                            MenuItem::Divider,
                            MenuItem::Button("Close", None, MenuAction::CloseFile),
                            MenuItem::Button("Quit", None, MenuAction::Quit),
                        ],
                    ),
                    (
                        "Edit",
                        vec![
                            MenuItem::Button("Undo", None, MenuAction::Undo),
                            MenuItem::Button("Redo", None, MenuAction::Redo),
                            MenuItem::Divider,
                            MenuItem::Button("Cut", None, MenuAction::Cut),
                            MenuItem::Button("Copy", None, MenuAction::Copy),
                            MenuItem::Button("Paste", None, MenuAction::Paste),
                            MenuItem::Divider,
                            MenuItem::Button("Select All", None, MenuAction::SelectAll),
                            MenuItem::Divider,
                            MenuItem::Button("Find & Replace", None, MenuAction::FindReplace),
                        ],
                    ),
                    (
                        "View",
                        vec![
                            MenuItem::Button("Toggle Sidebar", None, MenuAction::ToggleSidebar),
                            MenuItem::Button("Toggle Preview", None, MenuAction::ToggleViewMode),
                            MenuItem::Divider,
                            MenuItem::Button("Zoom In", None, MenuAction::ZoomIn),
                            MenuItem::Button("Zoom Out", None, MenuAction::ZoomOut),
                            MenuItem::Button("Reset Zoom", None, MenuAction::ZoomReset),
                        ],
                    ),
                    (
                        "Help",
                        vec![
                            MenuItem::Button("About", None, MenuAction::About),
                        ],
                    ),
                ],
            );

        vec![menu_bar]
    }

    /// Called when the application is about to close
    fn on_close_requested(&self, _id: window::Id) -> Option<Self::Message> {
        // Check for unsaved changes
        if self.state.has_unsaved_changes() {
            Some(Message::System(SystemMessage::CloseRequested))
        } else {
            None
        }
    }
}

impl CosmicNotebook {
    /// Get the window title based on current document
    fn get_window_title(&self) -> String {
        match self.state.active_document() {
            Some(doc) => {
                let modified = if doc.modified { "• " } else { "" };
                format!("{}{} - Cosmic Notebook", modified, doc.display_name)
            }
            None => "Cosmic Notebook".to_string(),
        }
    }

    /// Update window title based on current document
    fn update_window_title(&mut self) {
        let title = self.get_window_title();
        self.set_header_title(title);
    }

    /// Render the main view
    fn view_main(&self) -> Element<'_, Message> {
        use cosmic::widget::{container, text, Column};
        use cosmic::iced::Length;

        // Simple placeholder for Phase 1
        let content: Element<Message> = if self.state.has_documents() {
            match self.state.active_document() {
                Some(doc) => {
                    let info = format!(
                        "File: {}\nLines: {}\nCharacters: {}",
                        doc.display_name,
                        doc.line_count(),
                        doc.char_count()
                    );
                    text(info).into()
                }
                None => text("No document selected").into(),
            }
        } else {
            Column::new()
                .push(text("Welcome to Cosmic Notebook").size(24))
                .push(text(""))
                .push(text("Press Ctrl+N to create a new file"))
                .push(text("Press Ctrl+O to open a file"))
                .spacing(8)
                .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    /// Helper to wrap message in cosmic Action
    fn app_message(msg: Message) -> cosmic::Action<Message> {
        cosmic::Action::App(msg)
    }

    /// Handle file-related messages
    fn handle_file_message(&mut self, msg: FileMessage) -> Task<Message> {
        match msg {
            FileMessage::New => {
                let doc = Document::new();
                let id = doc.id;
                // Create text_editor::Content for this document
                self.editor_contents.insert(id, text_editor::Content::new());
                self.state.add_document(doc);
                self.update_window_title();
                Task::none()
            }

            FileMessage::Open => {
                // TODO: Open file picker dialog
                // For Phase 1, this is a placeholder
                Task::none()
            }

            FileMessage::OpenPath(path) => {
                // Check if already open
                if let Some(id) = self.state.find_document_by_path(&path) {
                    self.state.set_active_document(id);
                    self.update_window_title();
                    return Task::none();
                }

                // Load file asynchronously
                Task::perform(
                    async move {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => FileMessage::Loaded { path, content },
                            Err(e) => FileMessage::LoadError {
                                path,
                                error: e.to_string(),
                            },
                        }
                    },
                    |msg| Self::app_message(Message::File(msg)),
                )
            }

            FileMessage::Loaded { path, content } => {
                let doc = Document::from_file(path.clone(), content.clone());
                let id = doc.id;
                // Create text_editor::Content with the file content
                self.editor_contents.insert(id, text_editor::Content::with_text(&content));
                self.state.add_document(doc);
                self.session.add_recent_file(path);
                self.update_window_title();
                Task::none()
            }

            FileMessage::LoadError { path, error } => {
                log::error!("Failed to load {}: {}", path.display(), error);
                self.state.set_status(
                    format!("Failed to open: {}", path.display()),
                    crate::state::StatusLevel::Error,
                );
                Task::none()
            }

            FileMessage::Save => {
                if let Some(doc) = self.state.active_document() {
                    if let Some(path) = doc.path.clone() {
                        let id = doc.id;
                        let content = doc.content_str();
                        return Task::perform(
                            async move {
                                match std::fs::write(&path, &content) {
                                    Ok(_) => FileMessage::Saved {
                                        document_id: id,
                                        path,
                                    },
                                    Err(e) => FileMessage::SaveError {
                                        document_id: id,
                                        error: e.to_string(),
                                    },
                                }
                            },
                            |msg| Self::app_message(Message::File(msg)),
                        );
                    }
                    // No path - need SaveAs
                    return Task::done(Self::app_message(Message::File(FileMessage::SaveAs)));
                }
                Task::none()
            }

            FileMessage::SaveAs => {
                // TODO: Show save dialog
                // For Phase 1, this is a placeholder
                Task::none()
            }

            FileMessage::SaveToPath { document_id, path } => {
                if let Some(doc) = self.state.get_document(document_id) {
                    let content = doc.content_str();
                    return Task::perform(
                        async move {
                            match std::fs::write(&path, &content) {
                                Ok(_) => FileMessage::Saved { document_id, path },
                                Err(e) => FileMessage::SaveError {
                                    document_id,
                                    error: e.to_string(),
                                },
                            }
                        },
                        |msg| Self::app_message(Message::File(msg)),
                    );
                }
                Task::none()
            }

            FileMessage::Saved { document_id, path } => {
                let title = {
                    if let Some(doc) = self.state.get_document_mut(document_id) {
                        doc.path = Some(path.clone());
                        doc.display_name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        doc.mark_saved();
                        Some(doc.display_name.clone())
                    } else {
                        None
                    }
                };
                
                // Clear recovery file since document is now saved
                if let Err(e) = self.recovery_manager.clear_recovery(&document_id.to_string()) {
                    log::warn!("Failed to clear recovery for {}: {}", document_id, e);
                }
                
                // Update tab title
                if let Some(title) = title {
                    self.state.tabs.update_title(document_id, title);
                }
                
                self.update_window_title();
                self.state.set_status(
                    format!("Saved: {}", path.display()),
                    crate::state::StatusLevel::Info,
                );
                Task::none()
            }

            FileMessage::SaveError { document_id, error } => {
                log::error!("Failed to save document {}: {}", document_id, error);
                self.state
                    .set_status("Failed to save file", crate::state::StatusLevel::Error);
                Task::none()
            }

            FileMessage::SaveAll => {
                // Save all modified documents
                let modified: Vec<_> = self
                    .state
                    .documents
                    .iter()
                    .filter(|(_, doc)| doc.modified && doc.path.is_some())
                    .map(|(id, doc)| (*id, doc.path.clone().unwrap(), doc.content_str()))
                    .collect();

                let tasks: Vec<_> = modified
                    .into_iter()
                    .map(|(id, path, content)| {
                        Task::perform(
                            async move {
                                match std::fs::write(&path, &content) {
                                    Ok(_) => FileMessage::Saved {
                                        document_id: id,
                                        path,
                                    },
                                    Err(e) => FileMessage::SaveError {
                                        document_id: id,
                                        error: e.to_string(),
                                    },
                                }
                            },
                            |msg| Self::app_message(Message::File(msg)),
                        )
                    })
                    .collect();

                Task::batch(tasks)
            }

            FileMessage::Close => {
                if let Some(id) = self.state.active_document {
                    return Task::done(Self::app_message(Message::File(FileMessage::CloseDocument(id))));
                }
                Task::none()
            }

            FileMessage::CloseDocument(id) => {
                if let Some(doc) = self.state.get_document(id) {
                    if doc.modified {
                        // TODO: Show save confirmation dialog
                        // For now, just close
                    }
                }
                // Remove the text_editor content
                self.editor_contents.remove(&id);
                self.state.close_document(id);
                self.update_window_title();
                Task::none()
            }

            FileMessage::CloseAll => {
                // TODO: Check for unsaved changes
                let ids: Vec<_> = self.state.documents.keys().copied().collect();
                for id in ids {
                    self.state.close_document(id);
                }
                self.update_window_title();
                Task::none()
            }

            _ => Task::none(),
        }
    }

    /// Handle tab-related messages
    fn handle_tab_message(&mut self, msg: TabMessage) -> Task<Message> {
        match msg {
            TabMessage::Select(id) => {
                self.state.set_active_document(id);
                self.update_window_title();
            }

            TabMessage::SelectIndex(index) => {
                self.state.tabs.set_active_index(index);
                if let Some(id) = self.state.tabs.active_tab() {
                    self.state.active_document = Some(id);
                }
                self.update_window_title();
            }

            TabMessage::Next => {
                self.state.tabs.next_tab();
                if let Some(id) = self.state.tabs.active_tab() {
                    self.state.active_document = Some(id);
                }
                self.update_window_title();
            }

            TabMessage::Previous => {
                self.state.tabs.prev_tab();
                if let Some(id) = self.state.tabs.active_tab() {
                    self.state.active_document = Some(id);
                }
                self.update_window_title();
            }

            TabMessage::Close(id) => {
                return Task::done(Self::app_message(Message::File(FileMessage::CloseDocument(id))));
            }

            TabMessage::CloseCurrent => {
                if let Some(id) = self.state.active_document {
                    return Task::done(Self::app_message(Message::File(FileMessage::CloseDocument(id))));
                }
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle editor-related messages
    fn handle_editor_message(&mut self, msg: EditorMessage) -> Task<Message> {
        match msg {
            EditorMessage::TextEditorAction { document_id, action } => {
                // Handle the text_editor::Action from the widget
                if let Some(content) = self.editor_contents.get_mut(&document_id) {
                    // Check if this is an edit action that modifies content
                    let is_edit = action.is_edit();
                    
                    // Apply the action to the text_editor content
                    content.perform(action);
                    
                    if is_edit {
                        // Update the document's rope content from the editor
                        let new_text = content.text();
                        if let Some(doc) = self.state.get_document_mut(document_id) {
                            doc.content = ropey::Rope::from_str(&new_text);
                            doc.mark_modified();
                            
                            let title = doc.title();
                            self.state.tabs.update_title(document_id, title);
                        }
                        self.update_window_title();
                        self.autosave_pending = true;
                    }
                }
            }

            EditorMessage::TextChanged { document_id, content } => {
                let title = {
                    if let Some(doc) = self.state.get_document_mut(document_id) {
                        doc.content = ropey::Rope::from_str(&content);
                        doc.mark_modified();
                        Some(doc.title())
                    } else {
                        None
                    }
                };
                
                if let Some(title) = title {
                    self.state.tabs.update_title(document_id, title);
                }
                
                self.update_window_title();
                self.autosave_pending = true;
            }

            EditorMessage::Undo => {
                // Note: text_editor widget doesn't have built-in undo
                // For now, show a status message
                self.state.set_status(
                    "Undo not yet implemented for text editor".to_string(),
                    crate::state::StatusLevel::Info,
                );
            }

            EditorMessage::Redo => {
                // Note: text_editor widget doesn't have built-in redo  
                // For now, show a status message
                self.state.set_status(
                    "Redo not yet implemented for text editor".to_string(),
                    crate::state::StatusLevel::Info,
                );
            }

            EditorMessage::SelectAll => {
                // Use text_editor's SelectAll action
                if let Some(doc_id) = self.state.active_document {
                    if let Some(content) = self.editor_contents.get_mut(&doc_id) {
                        content.perform(cosmic::widget::text_editor::Action::SelectAll);
                    }
                }
            }

            EditorMessage::GoToLine(line) => {
                if let Some(doc) = self.state.active_document_mut() {
                    let target_line = line.saturating_sub(1).min(doc.line_count().saturating_sub(1));
                    doc.editor_state.set_cursor(crate::state::CursorPosition::new(target_line, 0));
                }
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle clipboard-related messages
    fn handle_clipboard_message(&mut self, msg: ClipboardMessage) -> Task<Message> {
        use cosmic::widget::text_editor::{Action, Edit};
        use std::sync::Arc;
        
        match msg {
            ClipboardMessage::Cut => {
                // Get selection from text_editor widget, copy to clipboard, then delete
                if let Some(doc_id) = self.state.active_document {
                    if let Some(content) = self.editor_contents.get_mut(&doc_id) {
                        // Get selected text from text_editor widget
                        if let Some(selected_text) = content.selection() {
                            // Copy to system clipboard
                            if let Err(e) = crate::editor::copy_text(&selected_text) {
                                log::error!("Failed to copy to clipboard: {}", e);
                                self.state.set_status(
                                    "Failed to copy to clipboard".to_string(),
                                    crate::state::StatusLevel::Error,
                                );
                            } else {
                                // Delete the selection using text_editor's Edit action
                                content.perform(Action::Edit(Edit::Delete));
                                
                                // Sync content back to document
                                let new_text = content.text();
                                if let Some(doc) = self.state.get_document_mut(doc_id) {
                                    doc.content = ropey::Rope::from_str(&new_text);
                                    doc.mark_modified();
                                    let title = doc.title();
                                    self.state.tabs.update_title(doc_id, title);
                                }
                                
                                self.state.set_status(
                                    format!("Cut {} characters", selected_text.len()),
                                    crate::state::StatusLevel::Info,
                                );
                                self.update_window_title();
                            }
                        } else {
                            self.state.set_status(
                                "Nothing selected to cut".to_string(),
                                crate::state::StatusLevel::Info,
                            );
                        }
                    }
                }
            }

            ClipboardMessage::Copy => {
                // Get selection from text_editor widget and copy to clipboard
                if let Some(doc_id) = self.state.active_document {
                    if let Some(content) = self.editor_contents.get(&doc_id) {
                        if let Some(selected_text) = content.selection() {
                            if let Err(e) = crate::editor::copy_text(&selected_text) {
                                log::error!("Failed to copy to clipboard: {}", e);
                                self.state.set_status(
                                    "Failed to copy to clipboard".to_string(),
                                    crate::state::StatusLevel::Error,
                                );
                            } else {
                                self.state.set_status(
                                    format!("Copied {} characters", selected_text.len()),
                                    crate::state::StatusLevel::Info,
                                );
                            }
                        } else {
                            self.state.set_status(
                                "Nothing selected to copy".to_string(),
                                crate::state::StatusLevel::Info,
                            );
                        }
                    }
                }
            }

            ClipboardMessage::Paste => {
                // Get text from system clipboard and paste into text_editor
                match crate::editor::paste_text() {
                    Ok(text) => {
                        if let Some(doc_id) = self.state.active_document {
                            if let Some(content) = self.editor_contents.get_mut(&doc_id) {
                                // Use text_editor's Paste action
                                content.perform(Action::Edit(Edit::Paste(Arc::new(text.clone()))));
                                
                                // Sync content back to document
                                let new_text = content.text();
                                if let Some(doc) = self.state.get_document_mut(doc_id) {
                                    doc.content = ropey::Rope::from_str(&new_text);
                                    doc.mark_modified();
                                    let title = doc.title();
                                    self.state.tabs.update_title(doc_id, title);
                                }
                                
                                self.state.set_status(
                                    format!("Pasted {} characters", text.len()),
                                    crate::state::StatusLevel::Info,
                                );
                                self.update_window_title();
                            }
                        }
                    }
                    Err(crate::editor::ClipboardError::Empty) => {
                        self.state.set_status(
                            "Clipboard is empty".to_string(),
                            crate::state::StatusLevel::Info,
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to paste from clipboard: {}", e);
                        self.state.set_status(
                            "Failed to paste from clipboard".to_string(),
                            crate::state::StatusLevel::Error,
                        );
                    }
                }
            }

            ClipboardMessage::Content(text) => {
                // Handle paste content (legacy path, also used for programmatic paste)
                if let Some(doc_id) = self.state.active_document {
                    if let Some(content) = self.editor_contents.get_mut(&doc_id) {
                        content.perform(Action::Edit(Edit::Paste(Arc::new(text.clone()))));
                        
                        let new_text = content.text();
                        if let Some(doc) = self.state.get_document_mut(doc_id) {
                            doc.content = ropey::Rope::from_str(&new_text);
                            doc.mark_modified();
                            let title = doc.title();
                            self.state.tabs.update_title(doc_id, title);
                        }
                        
                        self.state.set_status(
                            format!("Pasted {} characters", text.len()),
                            crate::state::StatusLevel::Info,
                        );
                        self.update_window_title();
                    }
                }
            }

            ClipboardMessage::Error(error) => {
                log::error!("Clipboard error: {}", error);
                self.state
                    .set_status("Clipboard error".to_string(), crate::state::StatusLevel::Error);
            }
        }
        Task::none()
    }

    /// Handle search-related messages
    fn handle_search_message(&mut self, msg: SearchMessage) -> Task<Message> {
        match msg {
            SearchMessage::OpenFind => {
                self.state.find_dialog_open = true;
                self.state.find_replace_open = false;
            }

            SearchMessage::OpenFindReplace => {
                self.state.find_dialog_open = true;
                self.state.find_replace_open = true;
            }

            SearchMessage::CloseFind => {
                self.state.find_dialog_open = false;
                self.state.find_replace_open = false;
            }

            SearchMessage::UpdateQuery(query) => {
                self.state.find_query = query.clone();
                
                // Perform search in active document
                if let Some(doc) = self.state.active_document() {
                    let content = doc.content_str();
                    let options = crate::search::FindOptions {
                        case_sensitive: self.state.find_case_sensitive,
                        whole_word: self.state.find_whole_word,
                        use_regex: self.state.find_use_regex,
                        wrap_around: true,
                    };
                    
                    let mut engine = crate::search::SearchEngine::new();
                    let results = engine.find_all(&content, &query, &options);
                    
                    // Store results in editor state
                    if let Some(doc) = self.state.active_document_mut() {
                        doc.editor_state.find_results = results.iter()
                            .map(|r| (r.start, r.end))
                            .collect();
                        doc.editor_state.current_find_index = if results.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                    }
                }
            }

            SearchMessage::UpdateReplaceText(text) => {
                self.state.replace_text = text;
            }

            SearchMessage::ToggleCaseSensitive => {
                self.state.find_case_sensitive = !self.state.find_case_sensitive;
                // Re-run search with new options
                if !self.state.find_query.is_empty() {
                    return Task::done(Self::app_message(Message::Search(
                        SearchMessage::UpdateQuery(self.state.find_query.clone()),
                    )));
                }
            }

            SearchMessage::ToggleWholeWord => {
                self.state.find_whole_word = !self.state.find_whole_word;
                if !self.state.find_query.is_empty() {
                    return Task::done(Self::app_message(Message::Search(
                        SearchMessage::UpdateQuery(self.state.find_query.clone()),
                    )));
                }
            }

            SearchMessage::ToggleRegex => {
                self.state.find_use_regex = !self.state.find_use_regex;
                if !self.state.find_query.is_empty() {
                    return Task::done(Self::app_message(Message::Search(
                        SearchMessage::UpdateQuery(self.state.find_query.clone()),
                    )));
                }
            }

            SearchMessage::FindNext => {
                if let Some(doc) = self.state.active_document_mut() {
                    if let Some((start, _end)) = doc.editor_state.next_find_result() {
                        // Move cursor to the match
                        let (line, col) = {
                            let line = doc.content.char_to_line(start);
                            let line_start = doc.content.line_to_char(line);
                            (line, start - line_start)
                        };
                        doc.editor_state.set_cursor(crate::state::CursorPosition::new(line, col));
                    }
                }
            }

            SearchMessage::FindPrevious => {
                if let Some(doc) = self.state.active_document_mut() {
                    if let Some((start, _end)) = doc.editor_state.prev_find_result() {
                        let (line, col) = {
                            let line = doc.content.char_to_line(start);
                            let line_start = doc.content.line_to_char(line);
                            (line, start - line_start)
                        };
                        doc.editor_state.set_cursor(crate::state::CursorPosition::new(line, col));
                    }
                }
            }

            SearchMessage::Replace => {
                // Replace current match
                let replacement = self.state.replace_text.clone();
                let query = self.state.find_query.clone();
                
                if let Some(doc) = self.state.active_document_mut() {
                    if let Some(idx) = doc.editor_state.current_find_index {
                        if let Some(&(start, end)) = doc.editor_state.find_results.get(idx) {
                            doc.content.remove(start..end);
                            doc.content.insert(start, &replacement);
                            doc.mark_modified();
                            
                            // Re-run search after replacement
                            return Task::done(Self::app_message(Message::Search(
                                SearchMessage::UpdateQuery(query),
                            )));
                        }
                    }
                }
            }

            SearchMessage::ReplaceAll => {
                // Replace all matches
                if let Some(doc) = self.state.active_document() {
                    let content = doc.content_str();
                    let options = crate::search::FindOptions {
                        case_sensitive: self.state.find_case_sensitive,
                        whole_word: self.state.find_whole_word,
                        use_regex: self.state.find_use_regex,
                        wrap_around: true,
                    };
                    
                    let mut engine = crate::search::SearchEngine::new();
                    let (new_content, count) = engine.replace_all(
                        &content,
                        &self.state.find_query,
                        &self.state.replace_text,
                        &options,
                    );
                    
                    if count > 0 {
                        if let Some(doc) = self.state.active_document_mut() {
                            doc.content = ropey::Rope::from_str(&new_content);
                            doc.mark_modified();
                            doc.editor_state.find_results.clear();
                            doc.editor_state.current_find_index = None;
                            
                            self.state.set_status(
                                format!("Replaced {} occurrences", count),
                                crate::state::StatusLevel::Info,
                            );
                        }
                    }
                }
            }

            SearchMessage::ClearResults => {
                if let Some(doc) = self.state.active_document_mut() {
                    doc.editor_state.clear_find_results();
                }
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle view-related messages
    fn handle_view_message(&mut self, msg: ViewMessage) -> Task<Message> {
        match msg {
            ViewMessage::ToggleSidebar => {
                self.state.toggle_sidebar();
            }

            ViewMessage::SetSidebarWidth(width) => {
                self.state.sidebar.width = width;
            }

            ViewMessage::ToggleViewMode => {
                self.state.cycle_view_mode();
            }

            ViewMessage::SetViewMode(mode) => {
                self.state.set_view_mode(mode);
            }

            ViewMessage::ZoomIn => {
                self.config.editor.font_size = (self.config.editor.font_size + 1.0).min(48.0);
            }

            ViewMessage::ZoomOut => {
                self.config.editor.font_size = (self.config.editor.font_size - 1.0).max(8.0);
            }

            ViewMessage::ZoomReset => {
                self.config.editor.font_size = 14.0;
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle dialog-related messages
    fn handle_dialog_message(&mut self, msg: DialogMessage) -> Task<Message> {
        match msg {
            DialogMessage::OpenCommandPalette => {
                self.state.command_palette_open = true;
            }

            DialogMessage::CloseCommandPalette => {
                self.state.command_palette_open = false;
            }

            DialogMessage::CloseDialog => {
                self.state.command_palette_open = false;
                self.state.find_dialog_open = false;
                self.state.find_replace_open = false;
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle system-related messages
    fn handle_system_message(&mut self, msg: SystemMessage) -> Task<Message> {
        match msg {
            SystemMessage::CloseRequested => {
                if self.state.has_unsaved_changes() {
                    // TODO: Show confirmation dialog
                    self.state.quit_requested = true;
                    self.state.pending_saves = self.state.unsaved_documents();
                } else {
                    return Task::done(Self::app_message(Message::System(SystemMessage::Quit)));
                }
            }

            SystemMessage::Quit => {
                // Save session before quitting
                if let Err(e) = self.session.save() {
                    log::error!("Failed to save session: {}", e);
                }
                // Exit application
                std::process::exit(0);
            }

            SystemMessage::ForceQuit => {
                std::process::exit(0);
            }

            SystemMessage::WindowResized { width, height } => {
                self.session.update_window_state(None, Some((width, height)), false);
            }

            SystemMessage::WindowFocused(focused) => {
                if focused {
                    // TODO: Check for external file changes
                }
            }

            SystemMessage::Tick => {
                // Handle periodic tasks
                if self.autosave_pending && self.config.files.autosave_enabled {
                    // TODO: Trigger autosave
                    self.autosave_pending = false;
                }
            }

            SystemMessage::Error(error) => {
                log::error!("Application error: {}", error);
                self.state
                    .set_status(error, crate::state::StatusLevel::Error);
            }

            SystemMessage::ClearStatus => {
                self.state.clear_status();
            }

            _ => {}
        }
        Task::none()
    }

    /// Handle internal messages
    fn handle_internal_message(&mut self, msg: InternalMessage) -> Task<Message> {
        match msg {
            InternalMessage::DirectoryScanComplete(entries) => {
                self.state.sidebar.set_entries(entries);
            }

            InternalMessage::DirectoryScanError(error) => {
                log::error!("Directory scan error: {}", error);
                self.state.sidebar.set_error(error);
            }

            InternalMessage::AutosaveTrigger => {
                // Save recovery files for all modified documents
                if self.autosave_pending {
                    let mut save_count = 0;
                    
                    for (doc_id, doc) in &self.state.documents {
                        if doc.modified {
                            let content = doc.content_str();
                            let original_path = doc.path.as_deref();
                            let display_name = &doc.display_name;
                            
                            if let Err(e) = self.recovery_manager.save_recovery(
                                &doc_id.to_string(),
                                &content,
                                original_path,
                                display_name,
                            ) {
                                log::error!("Failed to save recovery for {}: {}", display_name, e);
                            } else {
                                save_count += 1;
                            }
                        }
                    }
                    
                    // Save the manifest
                    if let Err(e) = self.recovery_manager.save_if_dirty() {
                        log::error!("Failed to save recovery manifest: {}", e);
                    }
                    
                    if save_count > 0 {
                        log::debug!("Autosaved {} document(s)", save_count);
                    }
                    
                    self.autosave_pending = false;
                }
            }

            InternalMessage::ConfigChanged => {
                // Reload config if needed
                if let Ok(config) = Config::load() {
                    self.config = config;
                }
            }

            _ => {}
        }
        Task::none()
    }
}
