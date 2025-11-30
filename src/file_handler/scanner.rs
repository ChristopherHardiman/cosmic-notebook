//! Directory scanner for file browser sidebar
//!
//! Provides asynchronous recursive directory scanning with:
//! - Configurable depth limits
//! - File type filtering
//! - Hidden file handling
//! - Ignored directory patterns

use crate::state::FileEntry;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Configuration for directory scanning
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Maximum directory depth to scan (0 = root only)
    pub max_depth: usize,
    
    /// File extensions to include (empty = all files)
    pub include_extensions: HashSet<String>,
    
    /// Whether to show hidden files (starting with .)
    pub show_hidden: bool,
    
    /// Maximum number of entries to return
    pub max_entries: usize,
    
    /// Directories to always ignore
    pub ignored_dirs: HashSet<String>,
    
    /// Whether to include directories in results
    pub include_directories: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let mut ignored_dirs = HashSet::new();
        // Common directories to ignore
        ignored_dirs.insert(".git".to_string());
        ignored_dirs.insert("node_modules".to_string());
        ignored_dirs.insert("target".to_string());
        ignored_dirs.insert("__pycache__".to_string());
        ignored_dirs.insert(".venv".to_string());
        ignored_dirs.insert("venv".to_string());
        ignored_dirs.insert("build".to_string());
        ignored_dirs.insert("dist".to_string());
        ignored_dirs.insert(".cache".to_string());
        ignored_dirs.insert(".npm".to_string());
        ignored_dirs.insert(".cargo".to_string());
        
        let mut include_extensions = HashSet::new();
        include_extensions.insert("md".to_string());
        include_extensions.insert("markdown".to_string());
        
        Self {
            max_depth: 10,
            include_extensions,
            show_hidden: false,
            max_entries: 10_000,
            ignored_dirs,
            include_directories: true,
        }
    }
}

impl ScanConfig {
    /// Create a config that shows all files
    pub fn all_files() -> Self {
        Self {
            include_extensions: HashSet::new(),
            ..Self::default()
        }
    }
    
    /// Create a config for markdown files only
    pub fn markdown_only() -> Self {
        Self::default()
    }
    
    /// Set whether to show hidden files
    pub fn with_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }
    
    /// Set maximum depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
    
    /// Add an extension to include
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.include_extensions.insert(ext.into());
        self
    }
    
    /// Add a directory to ignore
    pub fn with_ignored_dir(mut self, dir: impl Into<String>) -> Self {
        self.ignored_dirs.insert(dir.into());
        self
    }
}

/// Result of a directory scan
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// The scanned entries
    pub entries: Vec<FileEntry>,
    
    /// Whether the scan was truncated due to max_entries
    pub truncated: bool,
    
    /// Number of directories scanned
    pub dirs_scanned: usize,
    
    /// Number of files found
    pub files_found: usize,
    
    /// Total scan time in milliseconds
    pub scan_time_ms: u64,
}

/// Scan a directory and return file entries
pub fn scan_directory(root: impl AsRef<Path>, config: &ScanConfig) -> ScanResult {
    let start = std::time::Instant::now();
    let root = root.as_ref();
    
    let mut entries = Vec::new();
    let mut dirs_scanned = 0;
    let mut files_found = 0;
    let mut truncated = false;
    
    // Track parent indices for building tree structure
    let mut path_to_index: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
    
    let walker = WalkDir::new(root)
        .max_depth(config.max_depth + 1)
        .follow_links(false)
        .sort_by(|a, b| {
            // Sort directories first, then by name
            let a_is_dir = a.file_type().is_dir();
            let b_is_dir = b.file_type().is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(b.file_name()),
            }
        });
    
    for entry in walker.into_iter().filter_entry(|e| should_include_dir(e, config)) {
        if entries.len() >= config.max_entries {
            truncated = true;
            break;
        }
        
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        let path = entry.path().to_path_buf();
        
        // Skip root directory itself
        if path == root {
            path_to_index.insert(path, 0);
            continue;
        }
        
        let is_dir = entry.file_type().is_dir();
        
        if is_dir {
            dirs_scanned += 1;
        } else {
            files_found += 1;
        }
        
        // Check file extension filter (only for files)
        if !is_dir && !config.include_extensions.is_empty() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            
            if let Some(ext) = ext {
                if !config.include_extensions.contains(&ext) {
                    continue;
                }
            } else {
                continue;
            }
        }
        
        // Skip hidden files if configured
        if !config.show_hidden && is_hidden(&entry) {
            continue;
        }
        
        // Skip directories if not including them
        if is_dir && !config.include_directories {
            continue;
        }
        
        // Calculate depth relative to root
        let depth = entry.depth();
        
        // Find parent index
        let parent_index = path.parent()
            .and_then(|p| path_to_index.get(p))
            .copied();
        
        let index = entries.len();
        
        // Store mapping for this path
        if is_dir {
            path_to_index.insert(path.clone(), index);
        }
        
        entries.push(FileEntry::new(path, depth.saturating_sub(1), parent_index));
    }
    
    let scan_time_ms = start.elapsed().as_millis() as u64;
    
    ScanResult {
        entries,
        truncated,
        dirs_scanned,
        files_found,
        scan_time_ms,
    }
}

/// Scan a directory asynchronously
pub async fn scan_directory_async(root: PathBuf, config: ScanConfig) -> ScanResult {
    // Run the blocking scan in a separate thread
    tokio::task::spawn_blocking(move || scan_directory(&root, &config))
        .await
        .unwrap_or_else(|_| ScanResult {
            entries: Vec::new(),
            truncated: false,
            dirs_scanned: 0,
            files_found: 0,
            scan_time_ms: 0,
        })
}

/// Check if a directory entry should be included during traversal
fn should_include_dir(entry: &DirEntry, config: &ScanConfig) -> bool {
    // Check if it's a directory we should skip
    if entry.file_type().is_dir() {
        if let Some(name) = entry.file_name().to_str() {
            if config.ignored_dirs.contains(name) {
                return false;
            }
            // Skip hidden directories unless configured to show
            if !config.show_hidden && name.starts_with('.') {
                return false;
            }
        }
    }
    true
}

/// Check if an entry is hidden
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Get only immediate children of a directory (non-recursive)
pub fn scan_children(parent: impl AsRef<Path>, config: &ScanConfig) -> Vec<FileEntry> {
    let parent = parent.as_ref();
    let mut entries = Vec::new();
    
    let read_dir = match std::fs::read_dir(parent) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };
    
    let mut children: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .collect();
    
    // Sort: directories first, then alphabetically
    children.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
    
    for child in children {
        let path = child.path();
        let is_dir = path.is_dir();
        
        // Check hidden
        let name = child.file_name();
        let name_str = name.to_string_lossy();
        if !config.show_hidden && name_str.starts_with('.') {
            continue;
        }
        
        // Check ignored directories
        if is_dir && config.ignored_dirs.contains(name_str.as_ref()) {
            continue;
        }
        
        // Check extension filter for files
        if !is_dir && !config.include_extensions.is_empty() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            
            if let Some(ext) = ext {
                if !config.include_extensions.contains(&ext) {
                    continue;
                }
            } else {
                continue;
            }
        }
        
        // Skip directories if not including them
        if is_dir && !config.include_directories {
            continue;
        }
        
        entries.push(FileEntry::new(path, 0, None));
    }
    
    entries
}

/// Count markdown files in a directory (for quick stats)
pub fn count_markdown_files(root: impl AsRef<Path>) -> usize {
    let config = ScanConfig::markdown_only();
    WalkDir::new(root.as_ref())
        .max_depth(config.max_depth + 1)
        .into_iter()
        .filter_entry(|e| should_include_dir(e, &config))
        .filter_map(|e| e.ok())
        .filter(|e| {
            !e.file_type().is_dir() && 
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let base = dir.path();
        
        // Create test structure
        fs::write(base.join("readme.md"), "# Test").unwrap();
        fs::write(base.join("notes.md"), "Notes").unwrap();
        fs::write(base.join("config.toml"), "").unwrap();
        fs::create_dir(base.join("docs")).unwrap();
        fs::write(base.join("docs/guide.md"), "Guide").unwrap();
        fs::create_dir(base.join(".hidden")).unwrap();
        fs::write(base.join(".hidden/secret.md"), "Secret").unwrap();
        fs::create_dir(base.join("node_modules")).unwrap();
        fs::write(base.join("node_modules/pkg.md"), "Pkg").unwrap();
        
        dir
    }
    
    #[test]
    fn test_scan_markdown_only() {
        let dir = setup_test_dir();
        let config = ScanConfig::markdown_only();
        let result = scan_directory(dir.path(), &config);
        
        // Should find: docs folder, readme.md, notes.md, docs/guide.md
        // Should NOT find: config.toml, .hidden/*, node_modules/*
        let md_files: Vec<_> = result.entries.iter()
            .filter(|e| !e.is_directory)
            .collect();
        
        assert_eq!(md_files.len(), 3);
    }
    
    #[test]
    fn test_scan_all_files() {
        let dir = setup_test_dir();
        let config = ScanConfig::all_files();
        let result = scan_directory(dir.path(), &config);
        
        // Should include config.toml
        let has_toml = result.entries.iter()
            .any(|e| e.name == "config.toml");
        assert!(has_toml);
    }
    
    #[test]
    fn test_scan_ignores_node_modules() {
        let dir = setup_test_dir();
        let config = ScanConfig::all_files();
        let result = scan_directory(dir.path(), &config);
        
        // Should not include anything from node_modules
        let in_node_modules = result.entries.iter()
            .any(|e| e.path.to_string_lossy().contains("node_modules"));
        assert!(!in_node_modules);
    }
    
    #[test]
    fn test_scan_hidden_files() {
        let dir = setup_test_dir();
        let config = ScanConfig::all_files().with_hidden(true);
        let result = scan_directory(dir.path(), &config);
        
        // Should include hidden directory
        let has_hidden = result.entries.iter()
            .any(|e| e.name == ".hidden");
        assert!(has_hidden);
    }
    
    #[test]
    fn test_count_markdown_files() {
        let dir = setup_test_dir();
        let count = count_markdown_files(dir.path());
        assert_eq!(count, 3); // readme.md, notes.md, docs/guide.md
    }
}
