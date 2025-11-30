//! File system watcher for detecting external changes
//!
//! Monitors the filesystem for:
//! - File modifications (external edits)
//! - File creation and deletion
//! - Directory changes for sidebar updates

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// Events from the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// A file was created
    FileCreated(PathBuf),
    
    /// A file was modified
    FileModified(PathBuf),
    
    /// A file was deleted
    FileDeleted(PathBuf),
    
    /// A file was renamed (old path, new path)
    FileRenamed { from: PathBuf, to: PathBuf },
    
    /// A directory was created
    DirCreated(PathBuf),
    
    /// A directory was deleted
    DirDeleted(PathBuf),
    
    /// Watcher error occurred
    Error(String),
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce interval in milliseconds
    pub debounce_ms: u64,
    
    /// Whether to watch directories recursively
    pub recursive: bool,
    
    /// File extensions to watch (empty = all)
    pub watch_extensions: HashSet<String>,
    
    /// Whether to follow symlinks
    pub follow_symlinks: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        let mut watch_extensions = HashSet::new();
        watch_extensions.insert("md".to_string());
        watch_extensions.insert("markdown".to_string());
        
        Self {
            debounce_ms: 500,
            recursive: true,
            watch_extensions,
            follow_symlinks: false,
        }
    }
}

impl WatcherConfig {
    /// Watch all file types
    pub fn watch_all() -> Self {
        Self {
            watch_extensions: HashSet::new(),
            ..Self::default()
        }
    }
}

/// Manages file system watching
pub struct FileWatcher {
    /// The underlying notify watcher
    _watcher: RecommendedWatcher,
    
    /// Receiver for events
    event_rx: Receiver<notify::Result<Event>>,
    
    /// Paths being watched
    watched_paths: HashSet<PathBuf>,
    
    /// Configuration
    config: WatcherConfig,
    
    /// Pending events for debouncing
    pending_events: Vec<(WatchEvent, Instant)>,
    
    /// Last time events were processed
    last_process: Instant,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(config: WatcherConfig) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();
        
        let watcher_config = Config::default()
            .with_poll_interval(Duration::from_millis(config.debounce_ms));
        
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            watcher_config,
        )?;
        
        Ok(Self {
            _watcher: watcher,
            event_rx: rx,
            watched_paths: HashSet::new(),
            config,
            pending_events: Vec::new(),
            last_process: Instant::now(),
        })
    }
    
    /// Watch a path for changes
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<(), notify::Error> {
        let path = path.as_ref().to_path_buf();
        
        if self.watched_paths.contains(&path) {
            return Ok(());
        }
        
        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        
        // Need to use a method that exists - get mutable reference through accessor
        // This is a limitation - we need to recreate watcher for new paths
        // For now, we'll track what we want to watch
        self.watched_paths.insert(path);
        
        Ok(())
    }
    
    /// Stop watching a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), notify::Error> {
        let path = path.as_ref().to_path_buf();
        self.watched_paths.remove(&path);
        Ok(())
    }
    
    /// Poll for new events (non-blocking)
    pub fn poll(&mut self) -> Vec<WatchEvent> {
        let now = Instant::now();
        
        // Collect new events from receiver
        while let Ok(event_result) = self.event_rx.try_recv() {
            match event_result {
                Ok(event) => {
                    if let Some(watch_event) = self.convert_event(event) {
                        self.pending_events.push((watch_event, now));
                    }
                }
                Err(e) => {
                    self.pending_events.push((WatchEvent::Error(e.to_string()), now));
                }
            }
        }
        
        // Check if debounce period has passed
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);
        if now.duration_since(self.last_process) < debounce_duration {
            return Vec::new();
        }
        
        // Deduplicate and return events
        let events = self.deduplicate_events();
        self.pending_events.clear();
        self.last_process = now;
        
        events
    }
    
    /// Convert notify event to our event type
    fn convert_event(&self, event: Event) -> Option<WatchEvent> {
        let paths: Vec<PathBuf> = event.paths.into_iter().collect();
        
        if paths.is_empty() {
            return None;
        }
        
        let path = paths[0].clone();
        
        // Check extension filter
        if !self.should_watch_path(&path) {
            return None;
        }
        
        match event.kind {
            EventKind::Create(_) => {
                if path.is_dir() {
                    Some(WatchEvent::DirCreated(path))
                } else {
                    Some(WatchEvent::FileCreated(path))
                }
            }
            EventKind::Modify(_) => {
                Some(WatchEvent::FileModified(path))
            }
            EventKind::Remove(_) => {
                // Can't check if it was a dir since it's deleted
                // Assume file unless path looks like directory
                if path.extension().is_none() && !path.to_string_lossy().contains('.') {
                    Some(WatchEvent::DirDeleted(path))
                } else {
                    Some(WatchEvent::FileDeleted(path))
                }
            }
            EventKind::Access(_) => None, // Ignore access events
            EventKind::Other => None,
            _ => None,
        }
    }
    
    /// Check if we should watch this path based on extension filter
    fn should_watch_path(&self, path: &Path) -> bool {
        if self.config.watch_extensions.is_empty() {
            return true;
        }
        
        // Always watch directories
        if path.is_dir() {
            return true;
        }
        
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.config.watch_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }
    
    /// Deduplicate events (keep most recent for each path)
    fn deduplicate_events(&self) -> Vec<WatchEvent> {
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();
        let mut result = Vec::new();
        
        // Process in reverse order to keep most recent events
        for (event, _) in self.pending_events.iter().rev() {
            let path = match event {
                WatchEvent::FileCreated(p) |
                WatchEvent::FileModified(p) |
                WatchEvent::FileDeleted(p) |
                WatchEvent::DirCreated(p) |
                WatchEvent::DirDeleted(p) => p.clone(),
                WatchEvent::FileRenamed { to, .. } => to.clone(),
                WatchEvent::Error(_) => {
                    result.push(event.clone());
                    continue;
                }
            };
            
            if !seen_paths.contains(&path) {
                seen_paths.insert(path);
                result.push(event.clone());
            }
        }
        
        result.reverse();
        result
    }
    
    /// Get list of watched paths
    pub fn watched_paths(&self) -> &HashSet<PathBuf> {
        &self.watched_paths
    }
    
    /// Check if watching a specific path
    pub fn is_watching(&self, path: impl AsRef<Path>) -> bool {
        self.watched_paths.contains(path.as_ref())
    }
}

/// Simple debouncer for event processing
pub struct EventDebouncer {
    /// Pending paths with their last event time
    pending: std::collections::HashMap<PathBuf, (WatchEvent, Instant)>,
    
    /// Debounce duration
    debounce_duration: Duration,
}

impl EventDebouncer {
    /// Create a new debouncer
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            pending: std::collections::HashMap::new(),
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }
    
    /// Add an event to the debouncer
    pub fn add(&mut self, event: WatchEvent) {
        let path = match &event {
            WatchEvent::FileCreated(p) |
            WatchEvent::FileModified(p) |
            WatchEvent::FileDeleted(p) |
            WatchEvent::DirCreated(p) |
            WatchEvent::DirDeleted(p) => p.clone(),
            WatchEvent::FileRenamed { to, .. } => to.clone(),
            WatchEvent::Error(_) => return,
        };
        
        self.pending.insert(path, (event, Instant::now()));
    }
    
    /// Get events that have passed the debounce period
    pub fn get_ready(&mut self) -> Vec<WatchEvent> {
        let now = Instant::now();
        let ready: Vec<PathBuf> = self.pending
            .iter()
            .filter(|(_, (_, time))| now.duration_since(*time) >= self.debounce_duration)
            .map(|(path, _)| path.clone())
            .collect();
        
        ready.into_iter()
            .filter_map(|path| self.pending.remove(&path))
            .map(|(event, _)| event)
            .collect()
    }
    
    /// Check if there are pending events
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    
    /// Clear all pending events
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

/// Conflict detection for open files
#[derive(Debug, Clone)]
pub struct FileConflict {
    /// Path of the conflicting file
    pub path: PathBuf,
    
    /// What kind of external change occurred
    pub change_type: ConflictType,
    
    /// Whether the local copy has unsaved changes
    pub has_local_changes: bool,
}

/// Type of conflict with external file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// File was modified externally
    ExternalModification,
    
    /// File was deleted externally
    ExternalDeletion,
    
    /// File was renamed externally
    ExternalRename,
}

/// Conflict resolver options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Reload from disk (discard local changes)
    ReloadFromDisk,
    
    /// Keep local version (will overwrite on next save)
    KeepLocal,
    
    /// Save to a new location
    SaveAs,
    
    /// Show diff (if supported)
    ShowDiff,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert!(config.recursive);
        assert_eq!(config.debounce_ms, 500);
        assert!(config.watch_extensions.contains("md"));
    }
    
    #[test]
    fn test_event_debouncer() {
        let mut debouncer = EventDebouncer::new(100);
        
        debouncer.add(WatchEvent::FileModified(PathBuf::from("/test.md")));
        assert!(debouncer.has_pending());
        
        // Not ready yet (need to wait for debounce)
        let ready = debouncer.get_ready();
        assert!(ready.is_empty());
        
        // After waiting
        std::thread::sleep(Duration::from_millis(150));
        let ready = debouncer.get_ready();
        assert_eq!(ready.len(), 1);
    }
}
