//! Recovery and backup system for crash protection
//!
//! Provides:
//! - Automatic recovery file creation
//! - Recovery manifest management
//! - Startup recovery detection and restoration
//! - Stale recovery file cleanup

use crate::config::APP_ID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Get the recovery directory path
pub fn recovery_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join(APP_ID).join("recovery"))
}

/// Ensure recovery directory exists
pub fn ensure_recovery_dir() -> std::io::Result<PathBuf> {
    let dir = recovery_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Could not determine data directory")
    })?;
    
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    
    Ok(dir)
}

/// Recovery manifest containing all recovery file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryManifest {
    /// Schema version for future compatibility
    pub version: u32,
    
    /// Map of document UUID to recovery entry
    pub files: HashMap<String, RecoveryEntry>,
    
    /// When the manifest was last updated
    pub last_updated: SystemTime,
}

impl Default for RecoveryManifest {
    fn default() -> Self {
        Self {
            version: 1,
            files: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
}

impl RecoveryManifest {
    /// Load manifest from disk
    pub fn load() -> Result<Self, RecoveryError> {
        let dir = recovery_dir().ok_or(RecoveryError::NoRecoveryDir)?;
        let manifest_path = dir.join("manifest.json");
        
        if !manifest_path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| RecoveryError::ParseError(e.to_string()))
    }
    
    /// Save manifest to disk
    pub fn save(&self) -> Result<(), RecoveryError> {
        let dir = ensure_recovery_dir()
            .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        
        let manifest_path = dir.join("manifest.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        
        std::fs::write(&manifest_path, content)
            .map_err(|e| RecoveryError::IoError(e.to_string()))
    }
    
    /// Add or update a recovery entry
    pub fn add_entry(&mut self, document_id: &str, entry: RecoveryEntry) {
        self.files.insert(document_id.to_string(), entry);
        self.last_updated = SystemTime::now();
    }
    
    /// Remove a recovery entry
    pub fn remove_entry(&mut self, document_id: &str) -> Option<RecoveryEntry> {
        let entry = self.files.remove(document_id);
        if entry.is_some() {
            self.last_updated = SystemTime::now();
        }
        entry
    }
    
    /// Get a recovery entry
    pub fn get_entry(&self, document_id: &str) -> Option<&RecoveryEntry> {
        self.files.get(document_id)
    }
    
    /// Check if there are any recovery files
    pub fn has_recovery_files(&self) -> bool {
        !self.files.is_empty()
    }
    
    /// Get all entries that need recovery
    pub fn recoverable_entries(&self) -> Vec<(&String, &RecoveryEntry)> {
        self.files
            .iter()
            .filter(|(_, entry)| entry.recovery_path_exists())
            .collect()
    }
    
    /// Remove stale entries (older than max_age_days)
    pub fn remove_stale_entries(&mut self, max_age_days: u64) -> Vec<RecoveryEntry> {
        let now = SystemTime::now();
        let max_age_secs = max_age_days * 24 * 60 * 60;
        
        let stale_ids: Vec<String> = self.files
            .iter()
            .filter(|(_, entry)| {
                now.duration_since(entry.last_modified)
                    .map(|d| d.as_secs() > max_age_secs)
                    .unwrap_or(false)
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut removed = Vec::new();
        for id in stale_ids {
            if let Some(entry) = self.files.remove(&id) {
                removed.push(entry);
            }
        }
        
        if !removed.is_empty() {
            self.last_updated = SystemTime::now();
        }
        
        removed
    }
}

/// A single recovery entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEntry {
    /// Document UUID
    pub document_id: String,
    
    /// Original file path (None for new/unsaved files)
    pub original_path: Option<PathBuf>,
    
    /// Path to the recovery file
    pub recovery_path: PathBuf,
    
    /// Display name for the document
    pub display_name: String,
    
    /// When recovery file was first created
    pub created_at: SystemTime,
    
    /// When recovery file was last updated
    pub last_modified: SystemTime,
    
    /// Simple hash of content for change detection
    pub content_hash: Option<u64>,
}

impl RecoveryEntry {
    /// Create a new recovery entry
    pub fn new(document_id: &str, original_path: Option<PathBuf>, display_name: &str) -> Self {
        let recovery_path = recovery_dir()
            .map(|d| d.join(format!("{}.md.recovery", document_id)))
            .unwrap_or_else(|| PathBuf::from(format!("{}.md.recovery", document_id)));
        
        Self {
            document_id: document_id.to_string(),
            original_path,
            recovery_path,
            display_name: display_name.to_string(),
            created_at: SystemTime::now(),
            last_modified: SystemTime::now(),
            content_hash: None,
        }
    }
    
    /// Check if the recovery file exists
    pub fn recovery_path_exists(&self) -> bool {
        self.recovery_path.exists()
    }
    
    /// Read content from recovery file
    pub fn read_content(&self) -> Result<String, RecoveryError> {
        std::fs::read_to_string(&self.recovery_path)
            .map_err(|e| RecoveryError::IoError(e.to_string()))
    }
    
    /// Write content to recovery file
    pub fn write_content(&mut self, content: &str) -> Result<(), RecoveryError> {
        ensure_recovery_dir()
            .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        
        std::fs::write(&self.recovery_path, content)
            .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        
        self.last_modified = SystemTime::now();
        self.content_hash = Some(simple_hash(content));
        
        Ok(())
    }
    
    /// Delete the recovery file
    pub fn delete(&self) -> Result<(), RecoveryError> {
        if self.recovery_path.exists() {
            std::fs::remove_file(&self.recovery_path)
                .map_err(|e| RecoveryError::IoError(e.to_string()))?;
        }
        Ok(())
    }
    
    /// Check if original file has changed since recovery was created
    pub fn original_has_changed(&self) -> bool {
        if let Some(ref original) = self.original_path {
            if let Ok(metadata) = original.metadata() {
                if let Ok(modified) = metadata.modified() {
                    return modified > self.created_at;
                }
            }
        }
        false
    }
    
    /// Get age of recovery in days
    pub fn age_days(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.created_at)
            .map(|d| d.as_secs() / (24 * 60 * 60))
            .unwrap_or(0)
    }
}

/// Recovery manager for handling document recovery
pub struct RecoveryManager {
    /// The manifest
    manifest: RecoveryManifest,
    
    /// Whether changes have been made since last save
    dirty: bool,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        let manifest = RecoveryManifest::load().unwrap_or_default();
        Self {
            manifest,
            dirty: false,
        }
    }
    
    /// Check if there are files to recover
    pub fn has_recovery_files(&self) -> bool {
        self.manifest.has_recovery_files()
    }
    
    /// Get files that need recovery
    pub fn get_recoverable(&self) -> Vec<&RecoveryEntry> {
        self.manifest
            .recoverable_entries()
            .into_iter()
            .map(|(_, entry)| entry)
            .collect()
    }
    
    /// Create or update a recovery entry for a document
    pub fn save_recovery(
        &mut self,
        document_id: &str,
        content: &str,
        original_path: Option<&Path>,
        display_name: &str,
    ) -> Result<(), RecoveryError> {
        let mut entry = self.manifest
            .get_entry(document_id)
            .cloned()
            .unwrap_or_else(|| {
                RecoveryEntry::new(
                    document_id,
                    original_path.map(|p| p.to_path_buf()),
                    display_name,
                )
            });
        
        // Check if content actually changed
        let new_hash = simple_hash(content);
        if entry.content_hash == Some(new_hash) {
            return Ok(()); // No change, skip write
        }
        
        entry.write_content(content)?;
        self.manifest.add_entry(document_id, entry);
        self.dirty = true;
        
        Ok(())
    }
    
    /// Remove recovery for a document (e.g., after successful save)
    pub fn clear_recovery(&mut self, document_id: &str) -> Result<(), RecoveryError> {
        if let Some(entry) = self.manifest.remove_entry(document_id) {
            entry.delete()?;
            self.dirty = true;
        }
        Ok(())
    }
    
    /// Recover a document's content
    pub fn recover(&self, document_id: &str) -> Result<String, RecoveryError> {
        let entry = self.manifest
            .get_entry(document_id)
            .ok_or(RecoveryError::NotFound)?;
        
        entry.read_content()
    }
    
    /// Discard recovery for a document
    pub fn discard(&mut self, document_id: &str) -> Result<(), RecoveryError> {
        self.clear_recovery(document_id)
    }
    
    /// Clean up stale recovery files
    pub fn cleanup_stale(&mut self, max_age_days: u64) -> Result<usize, RecoveryError> {
        let stale = self.manifest.remove_stale_entries(max_age_days);
        let count = stale.len();
        
        for entry in stale {
            let _ = entry.delete(); // Best effort cleanup
        }
        
        if count > 0 {
            self.dirty = true;
        }
        
        Ok(count)
    }
    
    /// Save manifest if dirty
    pub fn save_if_dirty(&mut self) -> Result<(), RecoveryError> {
        if self.dirty {
            self.manifest.save()?;
            self.dirty = false;
        }
        Ok(())
    }
    
    /// Force save manifest
    pub fn save(&mut self) -> Result<(), RecoveryError> {
        self.manifest.save()?;
        self.dirty = false;
        Ok(())
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple hash function for content comparison
fn simple_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Recovery-related errors
#[derive(Debug, Clone)]
pub enum RecoveryError {
    /// No recovery directory available
    NoRecoveryDir,
    
    /// I/O error
    IoError(String),
    
    /// Parse error
    ParseError(String),
    
    /// Recovery entry not found
    NotFound,
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::NoRecoveryDir => write!(f, "Recovery directory not available"),
            RecoveryError::IoError(e) => write!(f, "I/O error: {}", e),
            RecoveryError::ParseError(e) => write!(f, "Parse error: {}", e),
            RecoveryError::NotFound => write!(f, "Recovery entry not found"),
        }
    }
}

impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        let h3 = simple_hash("world");
        
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
    
    #[test]
    fn test_recovery_manifest_default() {
        let manifest = RecoveryManifest::default();
        assert_eq!(manifest.version, 1);
        assert!(manifest.files.is_empty());
    }
    
    #[test]
    fn test_recovery_entry_age() {
        let entry = RecoveryEntry {
            document_id: "test".to_string(),
            original_path: None,
            recovery_path: PathBuf::from("/tmp/test.recovery"),
            display_name: "Test".to_string(),
            created_at: SystemTime::now(),
            last_modified: SystemTime::now(),
            content_hash: None,
        };
        
        // Just created, should be 0 days old
        assert_eq!(entry.age_days(), 0);
    }
}
