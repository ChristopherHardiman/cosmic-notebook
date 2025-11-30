//! Markdown module for Cosmic Notebook
//!
//! Handles Markdown parsing and rendering including:
//! - Markdown tokenization
//! - Syntax highlighting
//! - Preview rendering
//! - Image handling
//! - Export functionality (HTML, PDF)

pub mod syntax;
pub mod preview;
pub mod image;
pub mod export;

pub use syntax::{
    MarkdownTokenizer, Token, TokenType, TokenStyle,
    SyntaxColorScheme, LineState, LineTokens,
};
pub use preview::{
    ViewModeExt, PreviewRenderer, PreviewElement, StyledText,
    ListItem, TaskItem, TableAlignment, HtmlExporter,
};
pub use image::{
    ImageHandler, ImageConfig, ImageFormat, ImageError, ImageResult,
};
pub use export::{
    MarkdownExporter, ExportFormat, HtmlExportOptions, ExportError, ExportResult,
};

/// Main Markdown renderer combining tokenization and preview
pub struct MarkdownRenderer {
    tokenizer: MarkdownTokenizer,
    preview_renderer: PreviewRenderer,
    color_scheme: SyntaxColorScheme,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {
            tokenizer: MarkdownTokenizer::new(),
            preview_renderer: PreviewRenderer::new(),
            color_scheme: SyntaxColorScheme::light(),
        }
    }
    
    /// Set the color scheme for syntax highlighting
    pub fn set_color_scheme(&mut self, dark_mode: bool) {
        self.color_scheme = if dark_mode {
            SyntaxColorScheme::dark()
        } else {
            SyntaxColorScheme::light()
        };
    }
    
    /// Get the current color scheme
    pub fn color_scheme(&self) -> &SyntaxColorScheme {
        &self.color_scheme
    }
    
    /// Get the tokenizer
    pub fn tokenizer(&mut self) -> &mut MarkdownTokenizer {
        &mut self.tokenizer
    }
    
    /// Get the preview renderer
    pub fn preview_renderer(&self) -> &PreviewRenderer {
        &self.preview_renderer
    }
    
    /// Tokenize a document for syntax highlighting
    pub fn tokenize_document(&mut self, lines: &[&str]) -> Vec<LineTokens> {
        self.tokenizer.tokenize_document(lines)
    }
    
    /// Render markdown to preview elements
    pub fn render_preview(&self, markdown: &str) -> Vec<PreviewElement> {
        self.preview_renderer.render(markdown)
    }

    /// Render markdown to HTML
    pub fn render_html(&self, markdown: &str) -> String {
        let exporter = HtmlExporter::new();
        exporter.export(markdown, None)
    }
    
    /// Invalidate cache from a specific line
    pub fn invalidate_from_line(&mut self, line_num: usize) {
        self.tokenizer.invalidate_from_line(line_num);
    }
    
    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.tokenizer.clear_cache();
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

