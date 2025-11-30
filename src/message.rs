//! Application message types
//!
//! Defines all messages that can be sent to the application's update function.
//! Messages are organized by category for clear handling and routing.

use crate::state::{DocumentId, FileEntry};
use cosmic::widget::text_editor;
use std::path::PathBuf;

/// Main application message enum
#[derive(Debug, Clone)]
pub enum Message {
    /// File operations
    File(FileMessage),

    /// Tab operations
    Tab(TabMessage),

    /// Editor operations
    Editor(EditorMessage),

    /// Clipboard operations
    Clipboard(ClipboardMessage),

    /// Search operations
    Search(SearchMessage),

    /// View operations
    View(ViewMessage),

    /// Dialog operations
    Dialog(DialogMessage),

    /// System/window events
    System(SystemMessage),

    /// Internal async operation results
    Internal(InternalMessage),
    
    /// Surface actions (for menu bar support)
    Surface(cosmic::surface::Action),

    /// No-op message (for subscriptions that don't need action)
    None,
}

/// File-related messages
#[derive(Debug, Clone)]
pub enum FileMessage {
    /// Create a new empty document
    New,

    /// Open a file (shows file picker)
    Open,

    /// Open a specific file path
    OpenPath(PathBuf),

    /// File was loaded from disk
    Loaded {
        path: PathBuf,
        content: String,
    },

    /// Error loading file
    LoadError {
        path: PathBuf,
        error: String,
    },

    /// Save the active document
    Save,

    /// Save the active document to a new path
    SaveAs,

    /// Save to a specific path
    SaveToPath {
        document_id: DocumentId,
        path: PathBuf,
    },

    /// File was saved successfully
    Saved {
        document_id: DocumentId,
        path: PathBuf,
    },

    /// Error saving file
    SaveError {
        document_id: DocumentId,
        error: String,
    },

    /// Save all modified documents
    SaveAll,

    /// Close the active document
    Close,

    /// Close a specific document
    CloseDocument(DocumentId),

    /// Close all documents
    CloseAll,

    /// Reload file from disk
    Reload(DocumentId),

    /// File changed externally
    ExternalChange {
        path: PathBuf,
        kind: ExternalChangeKind,
    },

    /// Open recent file
    OpenRecent(PathBuf),

    /// Clear recent files
    ClearRecent,

    /// Reveal file in system file manager
    RevealInFileManager(PathBuf),
}

/// Kind of external file change
#[derive(Debug, Clone)]
pub enum ExternalChangeKind {
    Modified,
    Deleted,
    Renamed(PathBuf),
}

/// Tab-related messages
#[derive(Debug, Clone)]
pub enum TabMessage {
    /// Select a tab by document ID
    Select(DocumentId),

    /// Select a tab by index
    SelectIndex(usize),

    /// Switch to next tab
    Next,

    /// Switch to previous tab
    Previous,

    /// Close a tab
    Close(DocumentId),

    /// Close current tab
    CloseCurrent,

    /// Close all tabs
    CloseAll,

    /// Close other tabs
    CloseOthers(DocumentId),

    /// Close tabs to the right
    CloseToRight(DocumentId),

    /// Start dragging a tab
    StartDrag(usize),

    /// Update drag position
    DragOver(usize),

    /// End drag operation
    EndDrag,

    /// Cancel drag operation
    CancelDrag,

    /// Toggle tab pin
    TogglePin(DocumentId),
}

/// Editor-related messages
#[derive(Debug, Clone)]
pub enum EditorMessage {
    /// Text content changed
    TextChanged {
        document_id: DocumentId,
        content: String,
    },

    /// Text was inserted
    Insert {
        document_id: DocumentId,
        position: usize,
        text: String,
    },

    /// Text was deleted
    Delete {
        document_id: DocumentId,
        start: usize,
        end: usize,
    },

    /// Cursor moved
    CursorMoved {
        document_id: DocumentId,
        line: usize,
        column: usize,
    },

    /// Selection changed
    SelectionChanged {
        document_id: DocumentId,
        start: (usize, usize),
        end: (usize, usize),
    },

    /// Undo last edit
    Undo,

    /// Redo last undone edit
    Redo,

    /// Select all text
    SelectAll,

    /// Go to a specific line
    GoToLine(usize),

    /// Indent selection
    Indent,

    /// Outdent selection
    Outdent,

    /// Toggle comment on selection
    ToggleComment,

    /// Duplicate current line(s)
    DuplicateLine,

    /// Move line(s) up
    MoveLineUp,

    /// Move line(s) down
    MoveLineDown,

    /// Delete current line
    DeleteLine,

    /// Insert line below
    InsertLineBelow,

    /// Insert line above
    InsertLineAbove,

    /// Format document
    Format,

    /// Scroll to position
    ScrollTo {
        document_id: DocumentId,
        line: usize,
    },

    /// Text editor action from iced's text_editor widget
    TextEditorAction {
        document_id: DocumentId,
        action: text_editor::Action,
    },
}

/// Clipboard-related messages
#[derive(Debug, Clone)]
pub enum ClipboardMessage {
    /// Cut selection to clipboard
    Cut,

    /// Copy selection to clipboard
    Copy,

    /// Paste from clipboard
    Paste,

    /// Clipboard content received
    Content(String),

    /// Clipboard error
    Error(String),
}

/// Search-related messages
#[derive(Debug, Clone)]
pub enum SearchMessage {
    /// Open find dialog
    OpenFind,

    /// Open find and replace dialog
    OpenFindReplace,

    /// Close search dialogs
    CloseFind,

    /// Update find query
    UpdateQuery(String),

    /// Update replace text
    UpdateReplaceText(String),

    /// Toggle case sensitivity
    ToggleCaseSensitive,

    /// Toggle whole word matching
    ToggleWholeWord,

    /// Toggle regex mode
    ToggleRegex,

    /// Find next match
    FindNext,

    /// Find previous match
    FindPrevious,

    /// Replace current match
    Replace,

    /// Replace all matches
    ReplaceAll,

    /// Global search across files
    GlobalSearch(String),

    /// Global search results received
    GlobalSearchResults(Vec<GlobalSearchResult>),

    /// Navigate to global search result
    GoToResult(GlobalSearchResult),

    /// Clear search results
    ClearResults,
}

/// A global search result
#[derive(Debug, Clone)]
pub struct GlobalSearchResult {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub match_text: String,
    pub context: String,
}

/// View-related messages
#[derive(Debug, Clone)]
pub enum ViewMessage {
    /// Toggle sidebar visibility
    ToggleSidebar,

    /// Set sidebar width
    SetSidebarWidth(u32),

    /// Toggle view mode (edit/preview/split)
    ToggleViewMode,

    /// Set specific view mode
    SetViewMode(crate::config::ViewMode),

    /// Zoom in (increase font size)
    ZoomIn,

    /// Zoom out (decrease font size)
    ZoomOut,

    /// Reset zoom level
    ZoomReset,

    /// Toggle fullscreen
    ToggleFullscreen,

    /// Toggle line numbers
    ToggleLineNumbers,

    /// Toggle word wrap
    ToggleWordWrap,

    /// Toggle status bar
    ToggleStatusBar,

    /// Focus editor
    FocusEditor,

    /// Focus sidebar
    FocusSidebar,
}

/// Dialog-related messages
#[derive(Debug, Clone)]
pub enum DialogMessage {
    /// Open command palette
    OpenCommandPalette,

    /// Close command palette
    CloseCommandPalette,

    /// Command palette input changed
    CommandPaletteInput(String),

    /// Execute command from palette
    ExecuteCommand(String),

    /// Show confirmation dialog
    ShowConfirm {
        title: String,
        message: String,
        on_confirm: Box<Message>,
    },

    /// Confirmation result
    ConfirmResult(bool),

    /// Show error dialog
    ShowError {
        title: String,
        message: String,
    },

    /// Close dialog
    CloseDialog,

    /// Show about dialog
    ShowAbout,

    /// Show settings
    ShowSettings,
}

/// System/window messages
#[derive(Debug, Clone)]
pub enum SystemMessage {
    /// Window resize event
    WindowResized {
        width: u32,
        height: u32,
    },

    /// Window focus changed
    WindowFocused(bool),

    /// Window close requested
    CloseRequested,

    /// Quit application
    Quit,

    /// Force quit (skip unsaved check)
    ForceQuit,

    /// Theme changed
    ThemeChanged,

    /// Tick for periodic tasks (autosave, etc.)
    Tick,

    /// Keyboard shortcut triggered
    Shortcut(String),

    /// Drop files on window
    FilesDropped(Vec<PathBuf>),

    /// Application error
    Error(String),

    /// Clear status message
    ClearStatus,
}

/// Internal messages for async operations
#[derive(Debug, Clone)]
pub enum InternalMessage {
    /// Directory scan completed
    DirectoryScanComplete(Vec<FileEntry>),

    /// Directory scan failed
    DirectoryScanError(String),

    /// Autosave triggered
    AutosaveTrigger,

    /// Recovery save triggered
    RecoverySave,

    /// Recovery files found
    RecoveryFilesFound(Vec<PathBuf>),

    /// Config changed
    ConfigChanged,
}

/// Sidebar-specific messages (can be nested in other messages)
#[derive(Debug, Clone)]
pub enum SidebarMessage {
    /// Select a file
    SelectFile(PathBuf),

    /// Toggle folder expansion
    ToggleFolder(PathBuf),

    /// Open folder as root
    OpenFolder(PathBuf),

    /// Set filter text
    SetFilter(String),

    /// Refresh file list
    Refresh,

    /// Navigate to parent folder
    NavigateToParent,

    /// Create new file in current folder
    NewFile,

    /// Create new folder
    NewFolder,

    /// Rename entry
    Rename(PathBuf),

    /// Delete entry
    Delete(PathBuf),

    /// Show context menu
    ShowContextMenu {
        entry_index: usize,
        x: f32,
        y: f32,
    },

    /// Hide context menu
    HideContextMenu,

    /// Keyboard navigation up
    FocusUp,

    /// Keyboard navigation down
    FocusDown,

    /// Activate selected (enter key)
    ActivateSelected,
}

impl From<FileMessage> for Message {
    fn from(msg: FileMessage) -> Self {
        Message::File(msg)
    }
}

impl From<TabMessage> for Message {
    fn from(msg: TabMessage) -> Self {
        Message::Tab(msg)
    }
}

impl From<EditorMessage> for Message {
    fn from(msg: EditorMessage) -> Self {
        Message::Editor(msg)
    }
}

impl From<ClipboardMessage> for Message {
    fn from(msg: ClipboardMessage) -> Self {
        Message::Clipboard(msg)
    }
}

impl From<SearchMessage> for Message {
    fn from(msg: SearchMessage) -> Self {
        Message::Search(msg)
    }
}

impl From<ViewMessage> for Message {
    fn from(msg: ViewMessage) -> Self {
        Message::View(msg)
    }
}

impl From<DialogMessage> for Message {
    fn from(msg: DialogMessage) -> Self {
        Message::Dialog(msg)
    }
}

impl From<SystemMessage> for Message {
    fn from(msg: SystemMessage) -> Self {
        Message::System(msg)
    }
}

impl From<InternalMessage> for Message {
    fn from(msg: InternalMessage) -> Self {
        Message::Internal(msg)
    }
}
