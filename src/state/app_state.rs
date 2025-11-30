//! Root application state container
//!
//! Contains the central state for the entire application, including
//! document management, active document tracking, and UI state.

use super::{EditorState, SidebarState, TabState};
use crate::config::ViewMode;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Unique identifier for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(Uuid);

impl DocumentId {
    /// Create a new unique document ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single document in the editor
#[derive(Debug, Clone)]
pub struct Document {
    /// Unique identifier for this document
    pub id: DocumentId,

    /// File path (None for untitled documents)
    pub path: Option<PathBuf>,

    /// Document content as a rope
    pub content: ropey::Rope,

    /// Editor state (cursor, selection, etc.)
    pub editor_state: EditorState,

    /// Whether the document has unsaved changes
    pub modified: bool,

    /// Whether the document is read-only
    pub read_only: bool,

    /// Last known modification time of the file on disk
    pub last_disk_mtime: Option<std::time::SystemTime>,

    /// Display name for the document
    pub display_name: String,

    /// Document encoding
    pub encoding: DocumentEncoding,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            id: DocumentId::new(),
            path: None,
            content: ropey::Rope::new(),
            editor_state: EditorState::default(),
            modified: false,
            read_only: false,
            last_disk_mtime: None,
            display_name: "Untitled".to_string(),
            encoding: DocumentEncoding::default(),
        }
    }

    /// Create a document from file content
    pub fn from_file(path: PathBuf, content: String) -> Self {
        let display_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            id: DocumentId::new(),
            path: Some(path),
            content: ropey::Rope::from_str(&content),
            editor_state: EditorState::default(),
            modified: false,
            read_only: false,
            last_disk_mtime: None,
            display_name,
            encoding: DocumentEncoding::default(),
        }
    }

    /// Get the document title for display (with modification indicator)
    pub fn title(&self) -> String {
        if self.modified {
            format!("• {}", self.display_name)
        } else {
            self.display_name.clone()
        }
    }

    /// Get the full title with path for tooltip
    pub fn full_title(&self) -> String {
        match &self.path {
            Some(p) => p.to_string_lossy().to_string(),
            None => self.display_name.clone(),
        }
    }

    /// Check if document has a file on disk
    pub fn has_file(&self) -> bool {
        self.path.is_some()
    }

    /// Mark the document as modified
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    /// Mark the document as saved
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// Get content as string
    pub fn content_str(&self) -> String {
        self.content.to_string()
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.content.len_lines()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.content.len_chars()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Document encoding information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DocumentEncoding {
    #[default]
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
}

impl DocumentEncoding {
    /// Get display name for the encoding
    pub fn display_name(&self) -> &'static str {
        match self {
            DocumentEncoding::Utf8 => "UTF-8",
            DocumentEncoding::Utf8Bom => "UTF-8 with BOM",
            DocumentEncoding::Utf16Le => "UTF-16 LE",
            DocumentEncoding::Utf16Be => "UTF-16 BE",
        }
    }
}

/// Root application state
#[derive(Debug)]
pub struct AppState {
    /// All open documents indexed by ID
    pub documents: HashMap<DocumentId, Document>,

    /// Currently active document ID
    pub active_document: Option<DocumentId>,

    /// Tab bar state
    pub tabs: TabState,

    /// Sidebar state
    pub sidebar: SidebarState,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Whether command palette is open
    pub command_palette_open: bool,

    /// Whether find dialog is open
    pub find_dialog_open: bool,

    /// Whether find-replace dialog is open
    pub find_replace_open: bool,

    /// Current find query
    pub find_query: String,

    /// Current replace text
    pub replace_text: String,

    /// Find options: case sensitive
    pub find_case_sensitive: bool,

    /// Find options: whole word
    pub find_whole_word: bool,

    /// Find options: use regex
    pub find_use_regex: bool,

    /// Status bar message
    pub status_message: Option<StatusMessage>,

    /// Whether a quit has been requested
    pub quit_requested: bool,

    /// Documents with pending saves (for quit confirmation)
    pub pending_saves: Vec<DocumentId>,

    /// Global search results
    pub global_search_results: Vec<SearchResult>,

    /// Whether global search is in progress
    pub global_search_in_progress: bool,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            active_document: None,
            tabs: TabState::new(),
            sidebar: SidebarState::new(),
            view_mode: ViewMode::Edit,
            command_palette_open: false,
            find_dialog_open: false,
            find_replace_open: false,
            find_query: String::new(),
            replace_text: String::new(),
            find_case_sensitive: false,
            find_whole_word: false,
            find_use_regex: false,
            status_message: None,
            quit_requested: false,
            pending_saves: Vec::new(),
            global_search_results: Vec::new(),
            global_search_in_progress: false,
        }
    }

    /// Get the currently active document
    pub fn active_document(&self) -> Option<&Document> {
        self.active_document
            .and_then(|id| self.documents.get(&id))
    }

    /// Get the currently active document mutably
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_document
            .and_then(|id| self.documents.get_mut(&id))
    }

    /// Add a new document and make it active
    pub fn add_document(&mut self, document: Document) -> DocumentId {
        let id = document.id;
        self.tabs.add_tab(id, document.display_name.clone());
        self.documents.insert(id, document);
        self.active_document = Some(id);
        id
    }

    /// Close a document by ID
    pub fn close_document(&mut self, id: DocumentId) -> Option<Document> {
        let doc = self.documents.remove(&id);
        self.tabs.remove_tab(id);

        // Update active document if we closed the active one
        if self.active_document == Some(id) {
            self.active_document = self.tabs.active_tab();
        }

        doc
    }

    /// Set the active document
    pub fn set_active_document(&mut self, id: DocumentId) {
        if self.documents.contains_key(&id) {
            self.active_document = Some(id);
            self.tabs.set_active(id);
        }
    }

    /// Get a document by ID
    pub fn get_document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    /// Get a document mutably by ID
    pub fn get_document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }

    /// Find a document by its file path
    pub fn find_document_by_path(&self, path: &PathBuf) -> Option<DocumentId> {
        self.documents.iter().find_map(|(id, doc)| {
            if doc.path.as_ref() == Some(path) {
                Some(*id)
            } else {
                None
            }
        })
    }

    /// Check if any documents have unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.documents.values().any(|doc| doc.modified)
    }

    /// Get all documents with unsaved changes
    pub fn unsaved_documents(&self) -> Vec<DocumentId> {
        self.documents
            .iter()
            .filter(|(_, doc)| doc.modified)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get document count
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Check if there are any open documents
    pub fn has_documents(&self) -> bool {
        !self.documents.is_empty()
    }

    /// Set a status message
    pub fn set_status(&mut self, message: impl Into<String>, level: StatusLevel) {
        self.status_message = Some(StatusMessage {
            text: message.into(),
            level,
            timestamp: std::time::Instant::now(),
        });
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.sidebar.visible = !self.sidebar.visible;
    }

    /// Toggle view mode
    pub fn cycle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Edit => ViewMode::Preview,
            ViewMode::Preview => ViewMode::Split,
            ViewMode::Split => ViewMode::Edit,
        };
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Status message for the status bar
#[derive(Debug, Clone)]
pub struct StatusMessage {
    /// Message text
    pub text: String,

    /// Message level (info, warning, error)
    pub level: StatusLevel,

    /// When the message was set
    pub timestamp: std::time::Instant,
}

/// Status message level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Warning,
    Error,
}

/// Global search result entry
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// File path
    pub path: PathBuf,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// The matching text
    pub match_text: String,

    /// Context around the match
    pub context: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = Document::new();
        assert!(doc.path.is_none());
        assert!(!doc.modified);
        assert_eq!(doc.display_name, "Untitled");
    }

    #[test]
    fn test_document_from_file() {
        let path = PathBuf::from("/test/file.md");
        let content = "# Hello\n\nWorld".to_string();
        let doc = Document::from_file(path.clone(), content);

        assert_eq!(doc.path, Some(path));
        assert_eq!(doc.display_name, "file.md");
        assert!(!doc.modified);
    }

    #[test]
    fn test_document_title() {
        let mut doc = Document::new();
        assert_eq!(doc.title(), "Untitled");

        doc.mark_modified();
        assert_eq!(doc.title(), "• Untitled");
    }

    #[test]
    fn test_app_state_add_document() {
        let mut state = AppState::new();
        let doc = Document::new();
        let id = state.add_document(doc);

        assert!(state.documents.contains_key(&id));
        assert_eq!(state.active_document, Some(id));
        assert_eq!(state.document_count(), 1);
    }

    #[test]
    fn test_app_state_close_document() {
        let mut state = AppState::new();
        let doc = Document::new();
        let id = state.add_document(doc);

        let closed = state.close_document(id);
        assert!(closed.is_some());
        assert_eq!(state.document_count(), 0);
        assert!(state.active_document.is_none());
    }

    #[test]
    fn test_app_state_unsaved_changes() {
        let mut state = AppState::new();
        let mut doc = Document::new();
        doc.mark_modified();
        state.add_document(doc);

        assert!(state.has_unsaved_changes());
        assert_eq!(state.unsaved_documents().len(), 1);
    }
}
