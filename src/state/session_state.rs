//! Session state for persistence
//!
//! Contains state that should be persisted across application restarts,
//! including window state, open files, and session data.

use crate::config::ViewMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session state that can be serialized and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Window position (x, y)
    pub window_position: Option<(i32, i32)>,

    /// Window size (width, height)
    pub window_size: Option<(u32, u32)>,

    /// Whether window was maximized
    pub window_maximized: bool,

    /// Sidebar visibility
    pub sidebar_visible: bool,

    /// Sidebar width
    pub sidebar_width: u32,

    /// View mode
    pub view_mode: ViewMode,

    /// Open documents (file paths)
    pub open_files: Vec<PathBuf>,

    /// Active document index in open_files
    pub active_file_index: Option<usize>,

    /// Last opened directory
    pub last_directory: Option<PathBuf>,

    /// Recent files (limited list)
    pub recent_files: Vec<RecentFile>,

    /// Session version for migration
    pub version: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            window_position: None,
            window_size: Some((1200, 800)),
            window_maximized: false,
            sidebar_visible: true,
            sidebar_width: 250,
            view_mode: ViewMode::Edit,
            open_files: Vec::new(),
            active_file_index: None,
            last_directory: None,
            recent_files: Vec::new(),
            version: 1,
        }
    }
}

impl SessionState {
    /// Create a new session state
    pub fn new() -> Self {
        Self::default()
    }

    /// Load session state from disk
    pub fn load() -> Result<Self, SessionError> {
        let path = Self::session_file_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| SessionError::LoadError(e.to_string()))?;

        let session: SessionState = serde_json::from_str(&content)
            .map_err(|e| SessionError::ParseError(e.to_string()))?;

        // Migration would happen here if version differs
        Ok(session)
    }

    /// Save session state to disk
    pub fn save(&self) -> Result<(), SessionError> {
        let path = Self::session_file_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SessionError::SaveError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| SessionError::SaveError(e.to_string()))?;

        std::fs::write(&path, content)
            .map_err(|e| SessionError::SaveError(e.to_string()))?;

        Ok(())
    }

    /// Get the session file path
    fn session_file_path() -> Result<PathBuf, SessionError> {
        dirs::data_dir()
            .map(|p| p.join(crate::config::APP_ID).join("session.json"))
            .ok_or(SessionError::DirectoryError)
    }

    /// Add a file to recent files
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists (to move to top)
        self.recent_files.retain(|r| r.path != path);

        // Add to front
        self.recent_files.insert(
            0,
            RecentFile {
                path,
                last_opened: chrono::Utc::now(),
            },
        );

        // Trim to max size
        const MAX_RECENT: usize = 20;
        self.recent_files.truncate(MAX_RECENT);
    }

    /// Remove a file from recent files
    pub fn remove_recent_file(&mut self, path: &PathBuf) {
        self.recent_files.retain(|r| &r.path != path);
    }

    /// Clear recent files
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    /// Get recent files that still exist
    pub fn existing_recent_files(&self) -> Vec<&RecentFile> {
        self.recent_files
            .iter()
            .filter(|r| r.path.exists())
            .collect()
    }

    /// Update window state
    pub fn update_window_state(
        &mut self,
        position: Option<(i32, i32)>,
        size: Option<(u32, u32)>,
        maximized: bool,
    ) {
        if !maximized {
            self.window_position = position;
            self.window_size = size;
        }
        self.window_maximized = maximized;
    }

    /// Update open files from current state
    pub fn update_open_files(&mut self, files: Vec<PathBuf>, active_index: Option<usize>) {
        self.open_files = files;
        self.active_file_index = active_index;
    }
}

/// A recently opened file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    /// File path
    pub path: PathBuf,

    /// When the file was last opened
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

impl RecentFile {
    /// Get display name (filename)
    pub fn display_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }

    /// Get relative time since last opened
    pub fn relative_time(&self) -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.last_opened);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    }
}

/// Session-related errors
#[derive(Debug, Clone)]
pub enum SessionError {
    /// Error loading session
    LoadError(String),

    /// Error parsing session data
    ParseError(String),

    /// Error saving session
    SaveError(String),

    /// Could not determine session directory
    DirectoryError,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::LoadError(e) => write!(f, "Failed to load session: {}", e),
            SessionError::ParseError(e) => write!(f, "Failed to parse session: {}", e),
            SessionError::SaveError(e) => write!(f, "Failed to save session: {}", e),
            SessionError::DirectoryError => write!(f, "Could not determine session directory"),
        }
    }
}

impl std::error::Error for SessionError {}

/// Recovery file information for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryFile {
    /// Original file path (if any)
    pub original_path: Option<PathBuf>,

    /// Recovery file path
    pub recovery_path: PathBuf,

    /// Document display name
    pub display_name: String,

    /// When the recovery was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Document ID (for matching with session)
    pub document_id: Option<String>,
}

impl RecoveryFile {
    /// Create a new recovery file entry
    pub fn new(
        original_path: Option<PathBuf>,
        recovery_path: PathBuf,
        display_name: String,
    ) -> Self {
        Self {
            original_path,
            recovery_path,
            display_name,
            created_at: chrono::Utc::now(),
            document_id: None,
        }
    }

    /// Check if the recovery file still exists
    pub fn exists(&self) -> bool {
        self.recovery_path.exists()
    }

    /// Get age of the recovery file
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now().signed_duration_since(self.created_at)
    }

    /// Check if recovery file is stale (older than threshold)
    pub fn is_stale(&self, max_age_days: i64) -> bool {
        self.age().num_days() > max_age_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_default() {
        let session = SessionState::default();
        assert!(session.sidebar_visible);
        assert!(session.open_files.is_empty());
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_add_recent_file() {
        let mut session = SessionState::new();

        session.add_recent_file(PathBuf::from("/test1.md"));
        session.add_recent_file(PathBuf::from("/test2.md"));

        assert_eq!(session.recent_files.len(), 2);
        assert_eq!(session.recent_files[0].path, PathBuf::from("/test2.md"));

        // Adding same file moves it to top
        session.add_recent_file(PathBuf::from("/test1.md"));
        assert_eq!(session.recent_files.len(), 2);
        assert_eq!(session.recent_files[0].path, PathBuf::from("/test1.md"));
    }

    #[test]
    fn test_recent_file_display_name() {
        let recent = RecentFile {
            path: PathBuf::from("/some/path/readme.md"),
            last_opened: chrono::Utc::now(),
        };

        assert_eq!(recent.display_name(), "readme.md");
    }

    #[test]
    fn test_recovery_file_stale() {
        let recent = RecoveryFile {
            original_path: None,
            recovery_path: PathBuf::from("/tmp/recovery"),
            display_name: "test".to_string(),
            created_at: chrono::Utc::now() - chrono::Duration::days(10),
            document_id: None,
        };

        assert!(recent.is_stale(7));
        assert!(!recent.is_stale(14));
    }
}
