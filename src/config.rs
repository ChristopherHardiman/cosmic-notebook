//! Configuration management for Cosmic Notebook
//!
//! Handles loading, saving, and managing application configuration.
//! Configuration is persisted using cosmic-config for integration with COSMIC desktop.

use crate::error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application identifier following reverse-DNS convention
pub const APP_ID: &str = "com.cosmic.Notebook";

/// Default window width in pixels
pub const DEFAULT_WINDOW_WIDTH: u32 = 1200;

/// Default window height in pixels
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;

/// Minimum window width in pixels
pub const MIN_WINDOW_WIDTH: u32 = 400;

/// Minimum window height in pixels
pub const MIN_WINDOW_HEIGHT: u32 = 300;

/// Maximum file size to open (in bytes) - 10MB
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// File size warning threshold (in bytes) - 1MB
pub const WARNING_FILE_SIZE: u64 = 1024 * 1024;

/// Autosave interval in seconds
pub const DEFAULT_AUTOSAVE_INTERVAL: u64 = 60;

/// Maximum number of recent files to remember
pub const MAX_RECENT_FILES: usize = 20;

/// Maximum undo history entries
pub const MAX_UNDO_HISTORY: usize = 1000;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Editor configuration
    pub editor: EditorConfig,

    /// File handling configuration
    pub files: FileConfig,

    /// UI configuration
    pub ui: UiConfig,

    /// Keyboard shortcuts configuration
    pub keybindings: KeybindingsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            files: FileConfig::default(),
            ui: UiConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from cosmic-config or return defaults
    pub fn load() -> ConfigResult<Self> {
        // Try to load from cosmic-config
        // For now, return defaults - will integrate with cosmic-config in Phase 2
        Ok(Self::default())
    }

    /// Save configuration to cosmic-config
    pub fn save(&self) -> ConfigResult<()> {
        // Will integrate with cosmic-config in Phase 2
        Ok(())
    }

    /// Get the configuration directory path
    pub fn config_dir() -> ConfigResult<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(APP_ID))
            .ok_or(ConfigError::DirectoryError)
    }

    /// Get the data directory path (for session data, backups, etc.)
    pub fn data_dir() -> ConfigResult<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join(APP_ID))
            .ok_or(ConfigError::DirectoryError)
    }

    /// Get the cache directory path
    pub fn cache_dir() -> ConfigResult<PathBuf> {
        dirs::cache_dir()
            .map(|p| p.join(APP_ID))
            .ok_or(ConfigError::DirectoryError)
    }

    /// Get the recovery directory for crash recovery files
    pub fn recovery_dir() -> ConfigResult<PathBuf> {
        Self::data_dir().map(|p| p.join("recovery"))
    }

    /// Get the backup directory for file backups
    pub fn backup_dir() -> ConfigResult<PathBuf> {
        Self::data_dir().map(|p| p.join("backups"))
    }
}

/// Editor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Font family for the editor
    pub font_family: String,

    /// Font size in points
    pub font_size: f32,

    /// Tab width in spaces
    pub tab_width: u8,

    /// Use spaces instead of tabs
    pub use_spaces: bool,

    /// Enable line numbers
    pub show_line_numbers: bool,

    /// Highlight current line
    pub highlight_current_line: bool,

    /// Enable word wrap
    pub word_wrap: bool,

    /// Show whitespace characters
    pub show_whitespace: bool,

    /// Auto-indent on new line
    pub auto_indent: bool,

    /// Enable bracket matching
    pub bracket_matching: bool,

    /// Maximum undo history entries
    pub max_undo_history: usize,

    /// Cursor blink rate in milliseconds (0 to disable)
    pub cursor_blink_rate: u64,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_family: "Fira Code".to_string(),
            font_size: 14.0,
            tab_width: 4,
            use_spaces: true,
            show_line_numbers: true,
            highlight_current_line: true,
            word_wrap: true,
            show_whitespace: false,
            auto_indent: true,
            bracket_matching: true,
            max_undo_history: MAX_UNDO_HISTORY,
            cursor_blink_rate: 530,
        }
    }
}

/// File handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Enable autosave
    pub autosave_enabled: bool,

    /// Autosave interval in seconds
    pub autosave_interval: u64,

    /// Create backup before saving
    pub create_backups: bool,

    /// Maximum file size to open (in bytes)
    pub max_file_size: u64,

    /// Default file extension for new files
    pub default_extension: String,

    /// File extensions to show in sidebar
    pub visible_extensions: Vec<String>,

    /// Show hidden files in sidebar
    pub show_hidden_files: bool,

    /// Recent files list
    pub recent_files: Vec<PathBuf>,

    /// Maximum recent files to track
    pub max_recent_files: usize,

    /// Enable file watching for external changes
    pub watch_files: bool,

    /// Directories to ignore when scanning
    pub ignored_directories: Vec<String>,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
            autosave_interval: DEFAULT_AUTOSAVE_INTERVAL,
            create_backups: true,
            max_file_size: MAX_FILE_SIZE,
            default_extension: "md".to_string(),
            visible_extensions: vec!["md".to_string(), "markdown".to_string()],
            show_hidden_files: false,
            recent_files: Vec::new(),
            max_recent_files: MAX_RECENT_FILES,
            watch_files: true,
            ignored_directories: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "build".to_string(),
                "dist".to_string(),
            ],
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Default window width
    pub window_width: u32,

    /// Default window height
    pub window_height: u32,

    /// Sidebar visible by default
    pub sidebar_visible: bool,

    /// Sidebar width in pixels
    pub sidebar_width: u32,

    /// Default view mode
    pub default_view_mode: ViewMode,

    /// Show status bar
    pub show_status_bar: bool,

    /// Show toolbar
    pub show_toolbar: bool,

    /// Remember window position and size
    pub remember_window_state: bool,

    /// Theme preference (follows system by default)
    pub theme: ThemePreference,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
            sidebar_visible: true,
            sidebar_width: 250,
            default_view_mode: ViewMode::Edit,
            show_status_bar: true,
            show_toolbar: true,
            remember_window_state: true,
            theme: ThemePreference::System,
        }
    }
}

/// View mode for the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ViewMode {
    /// Edit mode - raw markdown editing
    #[default]
    Edit,
    /// Preview mode - rendered markdown
    Preview,
    /// Split mode - side-by-side edit and preview
    Split,
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreference {
    /// Follow system theme
    #[default]
    System,
    /// Always use light theme
    Light,
    /// Always use dark theme
    Dark,
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    // File operations
    pub new_file: String,
    pub open_file: String,
    pub save_file: String,
    pub save_file_as: String,
    pub close_tab: String,

    // Edit operations
    pub undo: String,
    pub redo: String,
    pub cut: String,
    pub copy: String,
    pub paste: String,
    pub select_all: String,

    // Search operations
    pub find: String,
    pub find_replace: String,
    pub find_next: String,
    pub find_previous: String,

    // Navigation
    pub go_to_line: String,
    pub next_tab: String,
    pub previous_tab: String,
    pub command_palette: String,

    // View operations
    pub toggle_sidebar: String,
    pub toggle_preview: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub zoom_reset: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            // File operations
            new_file: "Ctrl+N".to_string(),
            open_file: "Ctrl+O".to_string(),
            save_file: "Ctrl+S".to_string(),
            save_file_as: "Ctrl+Shift+S".to_string(),
            close_tab: "Ctrl+W".to_string(),

            // Edit operations
            undo: "Ctrl+Z".to_string(),
            redo: "Ctrl+Y".to_string(),
            cut: "Ctrl+X".to_string(),
            copy: "Ctrl+C".to_string(),
            paste: "Ctrl+V".to_string(),
            select_all: "Ctrl+A".to_string(),

            // Search operations
            find: "Ctrl+F".to_string(),
            find_replace: "Ctrl+H".to_string(),
            find_next: "F3".to_string(),
            find_previous: "Shift+F3".to_string(),

            // Navigation
            go_to_line: "Ctrl+G".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            previous_tab: "Ctrl+Shift+Tab".to_string(),
            command_palette: "Ctrl+Shift+P".to_string(),

            // View operations
            toggle_sidebar: "Ctrl+B".to_string(),
            toggle_preview: "Ctrl+Shift+V".to_string(),
            zoom_in: "Ctrl+=".to_string(),
            zoom_out: "Ctrl+-".to_string(),
            zoom_reset: "Ctrl+0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.editor.show_line_numbers);
        assert!(config.files.autosave_enabled);
        assert!(config.ui.sidebar_visible);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.editor.font_size, deserialized.editor.font_size);
    }

    #[test]
    fn test_view_mode_default() {
        assert_eq!(ViewMode::default(), ViewMode::Edit);
    }
}
