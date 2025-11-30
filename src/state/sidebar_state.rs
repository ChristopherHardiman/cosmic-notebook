//! Sidebar and file browser state
//!
//! Manages the state of the file browser sidebar including
//! file tree entries, expanded folders, and filtering.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

/// A single entry in the file tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full filesystem path
    pub path: PathBuf,

    /// Display name (filename only)
    pub name: String,

    /// Whether this is a directory
    pub is_directory: bool,

    /// Nesting depth from root (0-based)
    pub depth: usize,

    /// Index of parent entry in flat list (-1 for root entries)
    pub parent_index: Option<usize>,

    /// Last modification time
    pub modified_time: Option<SystemTime>,

    /// File size in bytes (0 for directories)
    pub size_bytes: u64,
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(path: PathBuf, depth: usize, parent_index: Option<usize>) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_directory = path.is_dir();

        // Get metadata (best effort)
        let (modified_time, size_bytes) = path
            .metadata()
            .map(|m| (m.modified().ok(), if is_directory { 0 } else { m.len() }))
            .unwrap_or((None, 0));

        Self {
            path,
            name,
            is_directory,
            depth,
            parent_index,
            modified_time,
            size_bytes,
        }
    }

    /// Get file extension (if any)
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// Check if this is a markdown file
    pub fn is_markdown(&self) -> bool {
        matches!(self.extension(), Some("md" | "markdown"))
    }

    /// Check if this is a hidden file (starts with dot)
    pub fn is_hidden(&self) -> bool {
        self.name.starts_with('.')
    }

    /// Get human-readable file size
    pub fn display_size(&self) -> String {
        if self.is_directory {
            return String::new();
        }

        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.1} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.1} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.1} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.size_bytes)
        }
    }
}

/// Sidebar state
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    /// Current working directory (root of file tree)
    pub root: Option<PathBuf>,

    /// Flat list of file entries
    pub entries: Vec<FileEntry>,

    /// Set of expanded directory paths
    pub expanded_folders: HashSet<PathBuf>,

    /// Currently selected/highlighted file path
    pub selected_path: Option<PathBuf>,

    /// Search/filter input text
    pub filter_text: String,

    /// Indices of entries matching current filter
    pub filtered_indices: Vec<usize>,

    /// Whether sidebar is visible
    pub visible: bool,

    /// Sidebar width in pixels
    pub width: u32,

    /// Whether a directory scan is in progress
    pub is_scanning: bool,

    /// Error message from last operation (if any)
    pub error_message: Option<String>,

    /// Focused entry index for keyboard navigation
    pub focused_index: Option<usize>,

    /// Whether the sidebar has keyboard focus
    pub has_focus: bool,

    /// Context menu state
    pub context_menu: Option<ContextMenuState>,
}

impl SidebarState {
    /// Create a new sidebar state
    pub fn new() -> Self {
        Self {
            root: None,
            entries: Vec::new(),
            expanded_folders: HashSet::new(),
            selected_path: None,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
            visible: true,
            width: 250,
            is_scanning: false,
            error_message: None,
            focused_index: None,
            has_focus: false,
            context_menu: None,
        }
    }

    /// Set the root directory
    pub fn set_root(&mut self, path: PathBuf) {
        self.root = Some(path.clone());
        self.expanded_folders.clear();
        self.expanded_folders.insert(path);
        self.clear_filter();
        self.error_message = None;
    }

    /// Clear the root and all entries
    pub fn clear(&mut self) {
        self.root = None;
        self.entries.clear();
        self.expanded_folders.clear();
        self.selected_path = None;
        self.clear_filter();
        self.error_message = None;
    }

    /// Set entries from a directory scan
    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
        self.is_scanning = false;
        self.apply_filter();
    }

    /// Check if a folder is expanded
    pub fn is_expanded(&self, path: &PathBuf) -> bool {
        self.expanded_folders.contains(path)
    }

    /// Toggle folder expansion
    pub fn toggle_folder(&mut self, path: &PathBuf) {
        if self.expanded_folders.contains(path) {
            self.expanded_folders.remove(path);
        } else {
            self.expanded_folders.insert(path.clone());
        }
    }

    /// Expand a folder
    pub fn expand_folder(&mut self, path: &PathBuf) {
        self.expanded_folders.insert(path.clone());
    }

    /// Collapse a folder
    pub fn collapse_folder(&mut self, path: &PathBuf) {
        self.expanded_folders.remove(path);
    }

    /// Set the selected path
    pub fn set_selected(&mut self, path: Option<PathBuf>) {
        self.selected_path = path;
    }

    /// Set filter text and apply filtering
    pub fn set_filter(&mut self, text: String) {
        self.filter_text = text;
        self.apply_filter();
    }

    /// Clear the filter
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filtered_indices.clear();
    }

    /// Apply current filter to entries
    fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_indices.clear();
            return;
        }

        let filter_lower = self.filter_text.to_lowercase();
        self.filtered_indices = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.name.to_lowercase().contains(&filter_lower))
            .map(|(i, _)| i)
            .collect();
    }

    /// Get visible entries (respecting expansion and filter)
    pub fn visible_entries(&self) -> Vec<(usize, &FileEntry)> {
        if !self.filter_text.is_empty() {
            // When filtering, show all matching entries
            return self
                .filtered_indices
                .iter()
                .filter_map(|&i| self.entries.get(i).map(|e| (i, e)))
                .collect();
        }

        // Otherwise, respect folder expansion
        let mut visible = Vec::new();
        let mut skip_depth: Option<usize> = None;

        for (index, entry) in self.entries.iter().enumerate() {
            // Check if we're skipping collapsed folder contents
            if let Some(skip) = skip_depth {
                if entry.depth > skip {
                    continue;
                } else {
                    skip_depth = None;
                }
            }

            visible.push((index, entry));

            // If this is a collapsed folder, skip its contents
            if entry.is_directory && !self.is_expanded(&entry.path) {
                skip_depth = Some(entry.depth);
            }
        }

        visible
    }

    /// Get entry by index
    pub fn get_entry(&self, index: usize) -> Option<&FileEntry> {
        self.entries.get(index)
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Move focus up in the visible entries
    pub fn focus_up(&mut self) {
        let visible = self.visible_entries();
        if visible.is_empty() {
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        let current_visible_pos = visible.iter().position(|(i, _)| *i == current);

        let new_pos = match current_visible_pos {
            Some(pos) if pos > 0 => pos - 1,
            Some(_) => visible.len() - 1, // Wrap to bottom
            None => 0,
        };

        self.focused_index = Some(visible[new_pos].0);
    }

    /// Move focus down in the visible entries
    pub fn focus_down(&mut self) {
        let visible = self.visible_entries();
        if visible.is_empty() {
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        let current_visible_pos = visible.iter().position(|(i, _)| *i == current);

        let new_pos = match current_visible_pos {
            Some(pos) if pos < visible.len() - 1 => pos + 1,
            Some(_) => 0, // Wrap to top
            None => 0,
        };

        self.focused_index = Some(visible[new_pos].0);
    }

    /// Get focused entry
    pub fn focused_entry(&self) -> Option<&FileEntry> {
        self.focused_index.and_then(|i| self.entries.get(i))
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set error message
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Show context menu at position
    pub fn show_context_menu(&mut self, entry_index: usize, x: f32, y: f32) {
        self.context_menu = Some(ContextMenuState {
            entry_index,
            x,
            y,
        });
    }

    /// Hide context menu
    pub fn hide_context_menu(&mut self) {
        self.context_menu = None;
    }
}

/// Context menu state
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Index of entry the menu is for
    pub entry_index: usize,

    /// X position of menu
    pub x: f32,

    /// Y position of menu
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_display_size() {
        let mut entry = FileEntry {
            path: PathBuf::from("/test.md"),
            name: "test.md".to_string(),
            is_directory: false,
            depth: 0,
            parent_index: None,
            modified_time: None,
            size_bytes: 1024,
        };

        assert_eq!(entry.display_size(), "1.0 KB");

        entry.size_bytes = 1024 * 1024;
        assert_eq!(entry.display_size(), "1.0 MB");

        entry.size_bytes = 500;
        assert_eq!(entry.display_size(), "500 B");
    }

    #[test]
    fn test_file_entry_is_markdown() {
        let md_entry = FileEntry {
            path: PathBuf::from("/test.md"),
            name: "test.md".to_string(),
            is_directory: false,
            depth: 0,
            parent_index: None,
            modified_time: None,
            size_bytes: 0,
        };
        assert!(md_entry.is_markdown());

        let txt_entry = FileEntry {
            path: PathBuf::from("/test.txt"),
            name: "test.txt".to_string(),
            is_directory: false,
            depth: 0,
            parent_index: None,
            modified_time: None,
            size_bytes: 0,
        };
        assert!(!txt_entry.is_markdown());
    }

    #[test]
    fn test_sidebar_expand_collapse() {
        let mut state = SidebarState::new();
        let path = PathBuf::from("/test");

        state.expand_folder(&path);
        assert!(state.is_expanded(&path));

        state.collapse_folder(&path);
        assert!(!state.is_expanded(&path));

        state.toggle_folder(&path);
        assert!(state.is_expanded(&path));
    }

    #[test]
    fn test_sidebar_filter() {
        let mut state = SidebarState::new();
        state.entries = vec![
            FileEntry {
                path: PathBuf::from("/readme.md"),
                name: "readme.md".to_string(),
                is_directory: false,
                depth: 0,
                parent_index: None,
                modified_time: None,
                size_bytes: 0,
            },
            FileEntry {
                path: PathBuf::from("/config.toml"),
                name: "config.toml".to_string(),
                is_directory: false,
                depth: 0,
                parent_index: None,
                modified_time: None,
                size_bytes: 0,
            },
        ];

        state.set_filter("read".to_string());
        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.filtered_indices[0], 0);
    }
}
