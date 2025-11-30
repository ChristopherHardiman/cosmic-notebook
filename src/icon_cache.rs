// SPDX-License-Identifier: GPL-3.0-only
//! Icon cache for bundled application icons.
//!
//! This module provides efficient caching of SVG icons that are bundled directly
//! into the application binary. Icons follow the COSMIC/freedesktop naming conventions
//! and use `currentColor` for theme adaptation.

use cosmic::widget::icon;
use std::collections::HashMap;

/// Key for icon cache lookup
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    pub name: &'static str,
    pub size: u16,
}

/// Icon cache that stores pre-loaded icon handles
pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    /// Create a new icon cache with all bundled icons
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        // Macro to bundle icons from the res/icons directory
        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../assets/icons/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: $name,
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        // Bundle all symbolic icons at 16px (standard COSMIC size)
        bundle!("file-markdown-symbolic", 16);
        bundle!("folder-open-symbolic", 16);
        bundle!("folder-closed-symbolic", 16);
        bundle!("tab-close-symbolic", 16);
        bundle!("tab-unsaved-symbolic", 16);
        bundle!("search-symbolic", 16);
        bundle!("settings-symbolic", 16);
        bundle!("preview-symbolic", 16);
        bundle!("split-view-symbolic", 16);
        bundle!("distraction-free-symbolic", 16);

        // Also bundle at 24px for larger UI elements
        bundle!("file-markdown-symbolic", 24);
        bundle!("folder-open-symbolic", 24);
        bundle!("folder-closed-symbolic", 24);
        bundle!("preview-symbolic", 24);
        bundle!("split-view-symbolic", 24);
        bundle!("distraction-free-symbolic", 24);

        Self { cache }
    }

    /// Get an icon from the cache by name and size
    pub fn get(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let key = IconCacheKey { name, size };
        
        // Return cached icon if available
        if let Some(handle) = self.cache.get(&key) {
            return icon::icon(handle.clone()).size(size);
        }

        // Fall back to loading by name from system icon theme
        icon::from_name(name).size(size).icon()
    }

    /// Get an icon handle from the cache
    pub fn get_handle(&self, name: &'static str, size: u16) -> Option<icon::Handle> {
        let key = IconCacheKey { name, size };
        self.cache.get(&key).cloned()
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Icon names used throughout the application
pub mod names {
    /// Markdown file icon
    pub const FILE_MARKDOWN: &str = "file-markdown-symbolic";
    
    /// Open folder icon
    pub const FOLDER_OPEN: &str = "folder-open-symbolic";
    
    /// Closed folder icon
    pub const FOLDER_CLOSED: &str = "folder-closed-symbolic";
    
    /// Tab close button icon
    pub const TAB_CLOSE: &str = "tab-close-symbolic";
    
    /// Unsaved tab indicator icon
    pub const TAB_UNSAVED: &str = "tab-unsaved-symbolic";
    
    /// Search icon
    pub const SEARCH: &str = "search-symbolic";
    
    /// Settings gear icon
    pub const SETTINGS: &str = "settings-symbolic";
    
    /// Preview/eye icon
    pub const PREVIEW: &str = "preview-symbolic";
    
    /// Split view icon
    pub const SPLIT_VIEW: &str = "split-view-symbolic";
    
    /// Distraction-free mode icon
    pub const DISTRACTION_FREE: &str = "distraction-free-symbolic";
    
    // Common system icons (loaded from theme)
    pub const DOCUMENT_NEW: &str = "document-new-symbolic";
    pub const DOCUMENT_OPEN: &str = "document-open-symbolic";
    pub const DOCUMENT_SAVE: &str = "document-save-symbolic";
    pub const EDIT_UNDO: &str = "edit-undo-symbolic";
    pub const EDIT_REDO: &str = "edit-redo-symbolic";
    pub const EDIT_CUT: &str = "edit-cut-symbolic";
    pub const EDIT_COPY: &str = "edit-copy-symbolic";
    pub const EDIT_PASTE: &str = "edit-paste-symbolic";
    pub const EDIT_FIND: &str = "edit-find-symbolic";
    pub const WINDOW_CLOSE: &str = "window-close-symbolic";
    pub const GO_NEXT: &str = "go-next-symbolic";
    pub const GO_DOWN: &str = "go-down-symbolic";
    pub const LIST_ADD: &str = "list-add-symbolic";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_cache_creation() {
        let cache = IconCache::new();
        // Verify icons are loaded
        assert!(cache
            .get_handle(names::FILE_MARKDOWN, 16)
            .is_some());
        assert!(cache
            .get_handle(names::FOLDER_OPEN, 16)
            .is_some());
    }
}
