//! File I/O operations with encoding detection and atomic writes
//!
//! Provides safe file reading and writing with:
//! - UTF-8 and UTF-16 encoding detection
//! - Atomic writes to prevent data loss
//! - File size limits and warnings

use crate::error::{FileError, FileResult};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Maximum file size allowed (10 MB)
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// File size that triggers a warning (1 MB)
pub const WARNING_FILE_SIZE: u64 = 1024 * 1024;

/// Detected encoding of a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEncoding {
    /// UTF-8 without BOM
    Utf8,
    /// UTF-8 with BOM
    Utf8Bom,
    /// UTF-16 Little Endian with BOM
    Utf16Le,
    /// UTF-16 Big Endian with BOM
    Utf16Be,
    /// Unknown/binary (lossy UTF-8 conversion used)
    Unknown,
}

impl Default for FileEncoding {
    fn default() -> Self {
        Self::Utf8
    }
}

/// Result of reading a file
#[derive(Debug, Clone)]
pub struct FileReadResult {
    /// The file content as a string
    pub content: String,
    /// Detected encoding
    pub encoding: FileEncoding,
    /// Original file size in bytes
    pub size_bytes: u64,
    /// Whether lossy conversion was used
    pub lossy: bool,
}

/// File metadata information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Canonical file path
    pub path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modification time
    pub modified: Option<SystemTime>,
    /// Creation time (if available)
    pub created: Option<SystemTime>,
    /// Whether file is read-only
    pub is_readonly: bool,
    /// Whether the file exists
    pub exists: bool,
}

impl FileInfo {
    /// Get file info for a path
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        match path.metadata() {
            Ok(metadata) => Self {
                path: canonical,
                size_bytes: metadata.len(),
                modified: metadata.modified().ok(),
                created: metadata.created().ok(),
                is_readonly: metadata.permissions().readonly(),
                exists: true,
            },
            Err(_) => Self {
                path: canonical,
                size_bytes: 0,
                modified: None,
                created: None,
                is_readonly: false,
                exists: false,
            },
        }
    }
    
    /// Check if file is too large
    pub fn is_too_large(&self) -> bool {
        self.size_bytes > MAX_FILE_SIZE
    }
    
    /// Check if file should show size warning
    pub fn should_warn_size(&self) -> bool {
        self.size_bytes > WARNING_FILE_SIZE && self.size_bytes <= MAX_FILE_SIZE
    }
    
    /// Human-readable time since last modification
    pub fn modified_ago(&self) -> String {
        match self.modified {
            Some(time) => {
                let now = SystemTime::now();
                match now.duration_since(time) {
                    Ok(duration) => {
                        let secs = duration.as_secs();
                        if secs < 60 {
                            "Just now".to_string()
                        } else if secs < 3600 {
                            format!("{} minutes ago", secs / 60)
                        } else if secs < 86400 {
                            format!("{} hours ago", secs / 3600)
                        } else {
                            format!("{} days ago", secs / 86400)
                        }
                    }
                    Err(_) => "Unknown".to_string(),
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

/// Detect file encoding from raw bytes
fn detect_encoding(bytes: &[u8]) -> FileEncoding {
    // Check for BOM markers
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return FileEncoding::Utf8Bom;
    }
    if bytes.len() >= 2 {
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return FileEncoding::Utf16Le;
        }
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return FileEncoding::Utf16Be;
        }
    }
    
    // Try to validate as UTF-8
    if std::str::from_utf8(bytes).is_ok() {
        FileEncoding::Utf8
    } else {
        FileEncoding::Unknown
    }
}

/// Decode bytes to string based on detected encoding
fn decode_content(bytes: &[u8], encoding: FileEncoding) -> (String, bool) {
    match encoding {
        FileEncoding::Utf8 => {
            match std::str::from_utf8(bytes) {
                Ok(s) => (s.to_string(), false),
                Err(_) => (String::from_utf8_lossy(bytes).to_string(), true),
            }
        }
        FileEncoding::Utf8Bom => {
            // Skip BOM bytes
            let content = &bytes[3..];
            match std::str::from_utf8(content) {
                Ok(s) => (s.to_string(), false),
                Err(_) => (String::from_utf8_lossy(content).to_string(), true),
            }
        }
        FileEncoding::Utf16Le => {
            // Skip BOM and decode
            let content = &bytes[2..];
            decode_utf16_le(content)
        }
        FileEncoding::Utf16Be => {
            // Skip BOM and decode
            let content = &bytes[2..];
            decode_utf16_be(content)
        }
        FileEncoding::Unknown => {
            // Lossy UTF-8 as fallback
            (String::from_utf8_lossy(bytes).to_string(), true)
        }
    }
}

/// Decode UTF-16 Little Endian bytes
fn decode_utf16_le(bytes: &[u8]) -> (String, bool) {
    let mut lossy = false;
    let u16_iter = bytes.chunks_exact(2).map(|chunk| {
        u16::from_le_bytes([chunk[0], chunk[1]])
    });
    
    let result: String = char::decode_utf16(u16_iter)
        .map(|r| {
            r.unwrap_or_else(|_| {
                lossy = true;
                '\u{FFFD}'
            })
        })
        .collect();
    
    (result, lossy)
}

/// Decode UTF-16 Big Endian bytes
fn decode_utf16_be(bytes: &[u8]) -> (String, bool) {
    let mut lossy = false;
    let u16_iter = bytes.chunks_exact(2).map(|chunk| {
        u16::from_be_bytes([chunk[0], chunk[1]])
    });
    
    let result: String = char::decode_utf16(u16_iter)
        .map(|r| {
            r.unwrap_or_else(|_| {
                lossy = true;
                '\u{FFFD}'
            })
        })
        .collect();
    
    (result, lossy)
}

/// Read a file with encoding detection
pub async fn read_file(path: impl AsRef<Path>) -> FileResult<FileReadResult> {
    let path = path.as_ref();
    let path_buf = path.to_path_buf();
    
    // Check file exists
    if !path.exists() {
        return Err(FileError::NotFound(path_buf));
    }
    
    // Check file size
    let metadata = tokio::fs::metadata(path).await.map_err(|e| FileError::ReadError {
        path: path_buf.clone(),
        source: e,
    })?;
    
    let size_bytes = metadata.len();
    if size_bytes > MAX_FILE_SIZE {
        return Err(FileError::FileTooLarge {
            path: path_buf,
            size: size_bytes,
            max_size: MAX_FILE_SIZE,
        });
    }
    
    // Read raw bytes
    let bytes = tokio::fs::read(path).await.map_err(|e| FileError::ReadError {
        path: path_buf.clone(),
        source: e,
    })?;
    
    // Detect encoding and decode
    let encoding = detect_encoding(&bytes);
    let (content, lossy) = decode_content(&bytes, encoding);
    
    Ok(FileReadResult {
        content,
        encoding,
        size_bytes,
        lossy,
    })
}

/// Read a file synchronously with encoding detection
pub fn read_file_sync(path: impl AsRef<Path>) -> FileResult<FileReadResult> {
    let path = path.as_ref();
    let path_buf = path.to_path_buf();
    
    // Check file exists
    if !path.exists() {
        return Err(FileError::NotFound(path_buf));
    }
    
    // Check file size
    let metadata = std::fs::metadata(path).map_err(|e| FileError::ReadError {
        path: path_buf.clone(),
        source: e,
    })?;
    
    let size_bytes = metadata.len();
    if size_bytes > MAX_FILE_SIZE {
        return Err(FileError::FileTooLarge {
            path: path_buf,
            size: size_bytes,
            max_size: MAX_FILE_SIZE,
        });
    }
    
    // Read raw bytes
    let bytes = std::fs::read(path).map_err(|e| FileError::ReadError {
        path: path_buf.clone(),
        source: e,
    })?;
    
    // Detect encoding and decode
    let encoding = detect_encoding(&bytes);
    let (content, lossy) = decode_content(&bytes, encoding);
    
    Ok(FileReadResult {
        content,
        encoding,
        size_bytes,
        lossy,
    })
}

/// Write content to a file using atomic write
/// 
/// This ensures the file is either fully written or unchanged,
/// preventing data loss from interrupted writes.
pub async fn write_file_atomic(path: impl AsRef<Path>, content: &str) -> FileResult<()> {
    let path = path.as_ref();
    let path_buf = path.to_path_buf();
    
    // Generate temp filename in same directory
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    
    let temp_filename = format!(".{}.{}.tmp", filename, timestamp);
    let temp_path = parent.join(&temp_filename);
    
    // Write to temp file
    let write_result = async {
        let mut file = tokio::fs::File::create(&temp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, content.as_bytes()).await?;
        tokio::io::AsyncWriteExt::flush(&mut file).await?;
        file.sync_all().await?;
        Ok::<(), std::io::Error>(())
    }.await;
    
    if let Err(e) = write_result {
        // Clean up temp file on failure
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(FileError::WriteError {
            path: path_buf,
            source: e,
        });
    }
    
    // Atomic rename
    if let Err(e) = tokio::fs::rename(&temp_path, path).await {
        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(FileError::WriteError {
            path: path_buf,
            source: e,
        });
    }
    
    Ok(())
}

/// Write content to a file synchronously using atomic write
pub fn write_file_atomic_sync(path: impl AsRef<Path>, content: &str) -> FileResult<()> {
    let path = path.as_ref();
    let path_buf = path.to_path_buf();
    
    // Generate temp filename in same directory
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    
    let temp_filename = format!(".{}.{}.tmp", filename, timestamp);
    let temp_path = parent.join(&temp_filename);
    
    // Write to temp file
    let write_result = (|| {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    
    if let Err(e) = write_result {
        // Clean up temp file on failure
        let _ = std::fs::remove_file(&temp_path);
        return Err(FileError::WriteError {
            path: path_buf,
            source: e,
        });
    }
    
    // Atomic rename
    if let Err(e) = std::fs::rename(&temp_path, path) {
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        return Err(FileError::WriteError {
            path: path_buf,
            source: e,
        });
    }
    
    Ok(())
}

/// Simple write without atomic safety (for non-critical writes)
pub async fn write_file(path: impl AsRef<Path>, content: &str) -> FileResult<()> {
    let path = path.as_ref();
    let path_buf = path.to_path_buf();
    
    tokio::fs::write(path, content).await.map_err(|e| FileError::WriteError {
        path: path_buf,
        source: e,
    })
}

/// Check if a file exists and is a regular file
pub fn file_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_file()
}

/// Check if a directory exists
pub fn dir_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists() && path.is_dir()
}

/// Get file size in bytes
pub fn file_size(path: impl AsRef<Path>) -> Option<u64> {
    std::fs::metadata(path.as_ref()).ok().map(|m| m.len())
}

/// Ensure parent directory exists
pub async fn ensure_parent_dir(path: impl AsRef<Path>) -> FileResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| FileError::DirectoryError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_encoding_utf8() {
        let bytes = "Hello, world!".as_bytes();
        assert_eq!(detect_encoding(bytes), FileEncoding::Utf8);
    }
    
    #[test]
    fn test_detect_encoding_utf8_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'H', b'i'];
        assert_eq!(detect_encoding(&bytes), FileEncoding::Utf8Bom);
    }
    
    #[test]
    fn test_detect_encoding_utf16_le() {
        let bytes = [0xFF, 0xFE, b'H', 0, b'i', 0];
        assert_eq!(detect_encoding(&bytes), FileEncoding::Utf16Le);
    }
    
    #[test]
    fn test_detect_encoding_utf16_be() {
        let bytes = [0xFE, 0xFF, 0, b'H', 0, b'i'];
        assert_eq!(detect_encoding(&bytes), FileEncoding::Utf16Be);
    }
    
    #[test]
    fn test_file_info_modified_ago() {
        let info = FileInfo {
            path: PathBuf::from("/test"),
            size_bytes: 100,
            modified: Some(SystemTime::now()),
            created: None,
            is_readonly: false,
            exists: true,
        };
        assert_eq!(info.modified_ago(), "Just now");
    }
}
