//! Image Handling for Markdown Editor
//!
//! This module provides functionality for handling images in the markdown editor:
//! - Drag & drop image support
//! - Clipboard image paste
//! - Saving images to assets folder
//! - Generating markdown image links

use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use chrono::Utc;
use thiserror::Error;

/// Errors that can occur during image handling
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid image format")]
    InvalidFormat,
    
    #[error("No document path set (save file first)")]
    NoDocumentPath,
    
    #[error("Failed to create assets directory: {0}")]
    AssetsDirError(String),
    
    #[error("Image data is empty")]
    EmptyData,
    
    #[error("Clipboard error: {0}")]
    Clipboard(String),
}

/// Result type for image operations
pub type ImageResult<T> = Result<T, ImageError>;

/// Configuration for image handling
#[derive(Debug, Clone)]
pub struct ImageConfig {
    /// Name of the assets directory (relative to document)
    pub assets_dir_name: String,
    /// Prefix for generated image filenames
    pub filename_prefix: String,
    /// Whether to copy images (true) or link to original (false)
    pub copy_images: bool,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            assets_dir_name: "assets".to_string(),
            filename_prefix: "image".to_string(),
            copy_images: true,
        }
    }
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Svg,
}

impl ImageFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Webp => "webp",
            ImageFormat::Svg => "svg",
        }
    }
    
    /// Detect format from magic bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some(ImageFormat::Png);
        }
        
        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(ImageFormat::Jpeg);
        }
        
        // GIF: GIF87a or GIF89a
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Some(ImageFormat::Gif);
        }
        
        // WebP: RIFF....WEBP
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
            return Some(ImageFormat::Webp);
        }
        
        // SVG: Look for XML/SVG declaration
        let start = std::str::from_utf8(&data[..data.len().min(256)]).ok()?;
        if start.contains("<svg") || start.contains("<?xml") {
            return Some(ImageFormat::Svg);
        }
        
        None
    }
    
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::Webp),
            "svg" => Some(ImageFormat::Svg),
            _ => None,
        }
    }
}

/// Image handler for managing images in the editor
pub struct ImageHandler {
    /// Configuration
    config: ImageConfig,
}

impl ImageHandler {
    /// Create a new image handler with default config
    pub fn new() -> Self {
        Self {
            config: ImageConfig::default(),
        }
    }
    
    /// Create a new image handler with custom config
    pub fn with_config(config: ImageConfig) -> Self {
        Self { config }
    }
    
    /// Get the assets directory path for a document
    pub fn get_assets_dir(&self, document_path: &Path) -> PathBuf {
        document_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(&self.config.assets_dir_name)
    }
    
    /// Ensure the assets directory exists
    pub fn ensure_assets_dir(&self, document_path: &Path) -> ImageResult<PathBuf> {
        let assets_dir = self.get_assets_dir(document_path);
        if !assets_dir.exists() {
            fs::create_dir_all(&assets_dir)
                .map_err(|e| ImageError::AssetsDirError(e.to_string()))?;
        }
        Ok(assets_dir)
    }
    
    /// Generate a unique filename for an image
    pub fn generate_filename(&self, format: ImageFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S-%3f").to_string();
        format!("{}-{}.{}", self.config.filename_prefix, timestamp, format.extension())
    }
    
    /// Generate markdown image link
    pub fn generate_markdown_link(&self, filename: &str, alt_text: &str) -> String {
        let relative_path = format!("{}/{}", self.config.assets_dir_name, filename);
        format!("![{}]({})", alt_text, relative_path)
    }
    
    /// Handle a dropped image file
    pub fn handle_dropped_file(
        &self,
        source_path: &Path,
        document_path: &Path,
    ) -> ImageResult<String> {
        // Determine the format
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let format = ImageFormat::from_extension(extension)
            .ok_or(ImageError::InvalidFormat)?;
        
        if self.config.copy_images {
            // Copy to assets directory
            let assets_dir = self.ensure_assets_dir(document_path)?;
            let filename = source_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
                .unwrap_or_else(|| self.generate_filename(format));
            
            let dest_path = assets_dir.join(&filename);
            fs::copy(source_path, &dest_path)?;
            
            // Generate alt text from filename (without extension)
            let alt_text = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image")
                .replace('-', " ")
                .replace('_', " ");
            
            Ok(self.generate_markdown_link(&filename, &alt_text))
        } else {
            // Link to original location (convert to relative path if possible)
            let relative_path = pathdiff::diff_paths(
                source_path,
                document_path.parent().unwrap_or(Path::new(".")),
            )
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| source_path.to_string_lossy().to_string());
            
            let alt_text = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            
            Ok(format!("![{}]({})", alt_text, relative_path))
        }
    }
    
    /// Handle pasted image data from clipboard
    pub fn handle_pasted_image(
        &self,
        image_data: &[u8],
        document_path: &Path,
    ) -> ImageResult<String> {
        if image_data.is_empty() {
            return Err(ImageError::EmptyData);
        }
        
        // Detect format from magic bytes
        let format = ImageFormat::from_bytes(image_data)
            .ok_or(ImageError::InvalidFormat)?;
        
        // Ensure assets directory exists
        let assets_dir = self.ensure_assets_dir(document_path)?;
        
        // Generate filename
        let filename = self.generate_filename(format);
        let dest_path = assets_dir.join(&filename);
        
        // Write the image data
        let mut file = fs::File::create(&dest_path)?;
        file.write_all(image_data)?;
        
        Ok(self.generate_markdown_link(&filename, "image"))
    }
    
    /// Handle multiple dropped files
    pub fn handle_dropped_files(
        &self,
        source_paths: &[PathBuf],
        document_path: &Path,
    ) -> Vec<ImageResult<String>> {
        source_paths
            .iter()
            .map(|path| self.handle_dropped_file(path, document_path))
            .collect()
    }
    
    /// Check if a file is a supported image format
    pub fn is_supported_image(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| ImageFormat::from_extension(e).is_some())
            .unwrap_or(false)
    }
    
    /// Get all images in the assets directory
    pub fn list_assets(&self, document_path: &Path) -> ImageResult<Vec<PathBuf>> {
        let assets_dir = self.get_assets_dir(document_path);
        
        if !assets_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut images = Vec::new();
        for entry in fs::read_dir(&assets_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && Self::is_supported_image(&path) {
                images.push(path);
            }
        }
        
        images.sort();
        Ok(images)
    }
    
    /// Delete an image from assets
    pub fn delete_asset(&self, document_path: &Path, filename: &str) -> ImageResult<()> {
        let assets_dir = self.get_assets_dir(document_path);
        let image_path = assets_dir.join(filename);
        
        if image_path.exists() {
            fs::remove_file(image_path)?;
        }
        
        Ok(())
    }
    
    /// Clean up unused images (images not referenced in document)
    pub fn cleanup_unused(
        &self,
        document_path: &Path,
        document_content: &str,
    ) -> ImageResult<Vec<String>> {
        let assets = self.list_assets(document_path)?;
        let mut removed = Vec::new();
        
        for asset_path in assets {
            let filename = asset_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Check if this image is referenced in the document
            let search_pattern = format!("{})", filename);
            if !document_content.contains(&search_pattern) {
                fs::remove_file(&asset_path)?;
                removed.push(filename.to_string());
            }
        }
        
        Ok(removed)
    }
}

impl Default for ImageHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Small utility module for path difference calculation
/// (If pathdiff crate is not available)
mod pathdiff {
    use std::path::{Path, PathBuf, Component};
    
    pub fn diff_paths(path: &Path, base: &Path) -> Option<PathBuf> {
        let path = path.canonicalize().ok()?;
        let base = base.canonicalize().ok()?;
        
        let mut path_components = path.components().peekable();
        let mut base_components = base.components().peekable();
        
        // Skip common prefix
        while let (Some(p), Some(b)) = (path_components.peek(), base_components.peek()) {
            if p == b {
                path_components.next();
                base_components.next();
            } else {
                break;
            }
        }
        
        // Build relative path
        let mut result = PathBuf::new();
        
        // Add ".." for each remaining base component
        for _ in base_components {
            result.push("..");
        }
        
        // Add remaining path components
        for component in path_components {
            result.push(component.as_os_str());
        }
        
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_format_detection() {
        // PNG signature
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(ImageFormat::from_bytes(&png_data), Some(ImageFormat::Png));
        
        // JPEG signature
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(ImageFormat::from_bytes(&jpeg_data), Some(ImageFormat::Jpeg));
        
        // GIF signature
        let gif_data = b"GIF89a\x00\x00";
        assert_eq!(ImageFormat::from_bytes(gif_data), Some(ImageFormat::Gif));
    }
    
    #[test]
    fn test_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("PNG"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("txt"), None);
    }
    
    #[test]
    fn test_generate_filename() {
        let handler = ImageHandler::new();
        let filename = handler.generate_filename(ImageFormat::Png);
        assert!(filename.starts_with("image-"));
        assert!(filename.ends_with(".png"));
    }
    
    #[test]
    fn test_generate_markdown_link() {
        let handler = ImageHandler::new();
        let link = handler.generate_markdown_link("test.png", "Test Image");
        assert_eq!(link, "![Test Image](assets/test.png)");
    }
    
    #[test]
    fn test_is_supported_image() {
        assert!(ImageHandler::is_supported_image(Path::new("test.png")));
        assert!(ImageHandler::is_supported_image(Path::new("test.jpg")));
        assert!(ImageHandler::is_supported_image(Path::new("test.gif")));
        assert!(!ImageHandler::is_supported_image(Path::new("test.txt")));
        assert!(!ImageHandler::is_supported_image(Path::new("test.rs")));
    }
}
