//! Export functionality for Markdown documents
//!
//! This module provides export capabilities for markdown documents:
//! - HTML export with embedded styles
//! - Future: PDF export

use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use thiserror::Error;
use pulldown_cmark::{Parser, Options};

/// Errors that can occur during export
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid export path")]
    InvalidPath,
    
    #[error("Export format not supported: {0}")]
    UnsupportedFormat(String),
}

/// Result type for export operations
pub type ExportResult<T> = Result<T, ExportError>;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Html,
    // Future: Pdf,
}

impl ExportFormat {
    /// Get the file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Html => "html",
        }
    }
    
    /// Get display name for the format
    pub fn display_name(&self) -> &'static str {
        match self {
            ExportFormat::Html => "HTML",
        }
    }
}

/// Options for HTML export
#[derive(Debug, Clone)]
pub struct HtmlExportOptions {
    /// Include CSS styles inline
    pub include_styles: bool,
    /// Embed images as base64 data URIs
    pub embed_images: bool,
    /// Document title
    pub title: Option<String>,
    /// Use dark mode styles
    pub dark_mode: bool,
    /// Custom CSS to include
    pub custom_css: Option<String>,
    /// Include table of contents
    pub include_toc: bool,
}

impl Default for HtmlExportOptions {
    fn default() -> Self {
        Self {
            include_styles: true,
            embed_images: false,
            title: None,
            dark_mode: false,
            custom_css: None,
            include_toc: false,
        }
    }
}

/// Main exporter for markdown documents
pub struct MarkdownExporter {
    options: Options,
}

impl MarkdownExporter {
    /// Create a new exporter
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        Self { options }
    }
    
    /// Export markdown to HTML string
    pub fn export_html(&self, markdown: &str, options: &HtmlExportOptions) -> String {
        let parser = Parser::new_ext(markdown, self.options);
        let mut html_content = String::new();
        pulldown_cmark::html::push_html(&mut html_content, parser);
        
        let title = options.title.as_deref().unwrap_or("Document");
        let styles = if options.include_styles {
            Self::get_styles(options.dark_mode, options.custom_css.as_deref())
        } else {
            String::new()
        };
        
        let toc = if options.include_toc {
            Self::generate_toc(markdown)
        } else {
            String::new()
        };
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="generator" content="Cosmic Notebook">
    <title>{}</title>
    {}
</head>
<body>
    <article class="markdown-body">
        {}
        {}
    </article>
</body>
</html>"#,
            Self::escape_html(title),
            styles,
            toc,
            html_content
        )
    }
    
    /// Export markdown to HTML file
    pub fn export_html_file(
        &self,
        markdown: &str,
        output_path: &Path,
        options: &HtmlExportOptions,
    ) -> ExportResult<()> {
        let html = self.export_html(markdown, options);
        let mut file = fs::File::create(output_path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
    
    /// Export with automatic format detection from path
    pub fn export_to_file(
        &self,
        markdown: &str,
        output_path: &Path,
    ) -> ExportResult<()> {
        let extension = output_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "html" | "htm" => {
                let options = HtmlExportOptions {
                    title: output_path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(String::from),
                    ..Default::default()
                };
                self.export_html_file(markdown, output_path, &options)
            }
            other => Err(ExportError::UnsupportedFormat(other.to_string())),
        }
    }
    
    /// Generate suggested output path from input path
    pub fn suggest_output_path(input_path: &Path, format: ExportFormat) -> PathBuf {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        
        let mut output = input_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        
        output.push(format!("{}.{}", stem, format.extension()));
        output
    }
    
    /// Generate a table of contents from markdown
    fn generate_toc(markdown: &str) -> String {
        let mut toc = String::from("<nav class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");
        let mut current_level = 0;
        
        for line in markdown.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('#') {
                continue;
            }
            
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level == 0 || level > 6 {
                continue;
            }
            
            let title = trimmed[level..].trim_start_matches(' ').trim();
            if title.is_empty() {
                continue;
            }
            
            // Generate anchor ID
            let anchor = Self::generate_anchor(title);
            
            // Handle nesting
            while current_level < level {
                toc.push_str("<ul>\n");
                current_level += 1;
            }
            while current_level > level {
                toc.push_str("</ul>\n");
                current_level -= 1;
            }
            
            toc.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>\n",
                anchor,
                Self::escape_html(title)
            ));
        }
        
        while current_level > 0 {
            toc.push_str("</ul>\n");
            current_level -= 1;
        }
        
        toc.push_str("</ul>\n</nav>\n");
        toc
    }
    
    /// Generate URL-safe anchor from heading text
    fn generate_anchor(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() {
                    '-'
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_matches(|c| c == '-' || c == '_')
            .to_string()
    }
    
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    fn get_styles(dark_mode: bool, custom_css: Option<&str>) -> String {
        let theme_styles = if dark_mode {
            r#"
            :root {
                --color-bg: #0d1117;
                --color-text: #c9d1d9;
                --color-heading: #c9d1d9;
                --color-link: #58a6ff;
                --color-code-bg: #161b22;
                --color-border: #30363d;
                --color-blockquote: #8b949e;
            }"#
        } else {
            r#"
            :root {
                --color-bg: #ffffff;
                --color-text: #24292e;
                --color-heading: #24292e;
                --color-link: #0366d6;
                --color-code-bg: #f6f8fa;
                --color-border: #e1e4e8;
                --color-blockquote: #6a737d;
            }"#
        };
        
        let custom = custom_css.unwrap_or("");
        
        format!(
            r#"<style>
        {}
        
        * {{
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            font-size: 16px;
            line-height: 1.6;
            color: var(--color-text);
            background-color: var(--color-bg);
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .markdown-body h1, .markdown-body h2, .markdown-body h3,
        .markdown-body h4, .markdown-body h5, .markdown-body h6 {{
            color: var(--color-heading);
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }}
        
        .markdown-body h1 {{ font-size: 2em; border-bottom: 1px solid var(--color-border); padding-bottom: .3em; }}
        .markdown-body h2 {{ font-size: 1.5em; border-bottom: 1px solid var(--color-border); padding-bottom: .3em; }}
        .markdown-body h3 {{ font-size: 1.25em; }}
        .markdown-body h4 {{ font-size: 1em; }}
        .markdown-body h5 {{ font-size: .875em; }}
        .markdown-body h6 {{ font-size: .85em; color: var(--color-blockquote); }}
        
        .markdown-body p {{
            margin: 16px 0;
        }}
        
        .markdown-body a {{
            color: var(--color-link);
            text-decoration: none;
        }}
        
        .markdown-body a:hover {{
            text-decoration: underline;
        }}
        
        .markdown-body code {{
            background-color: var(--color-code-bg);
            padding: .2em .4em;
            border-radius: 6px;
            font-size: 85%;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
        }}
        
        .markdown-body pre {{
            background-color: var(--color-code-bg);
            padding: 16px;
            overflow: auto;
            border-radius: 6px;
            line-height: 1.45;
        }}
        
        .markdown-body pre code {{
            background: transparent;
            padding: 0;
            border-radius: 0;
            font-size: 100%;
        }}
        
        .markdown-body blockquote {{
            margin: 16px 0;
            padding: 0 1em;
            color: var(--color-blockquote);
            border-left: .25em solid var(--color-border);
        }}
        
        .markdown-body table {{
            border-collapse: collapse;
            width: 100%;
            margin: 16px 0;
        }}
        
        .markdown-body table th,
        .markdown-body table td {{
            padding: 6px 13px;
            border: 1px solid var(--color-border);
        }}
        
        .markdown-body table th {{
            font-weight: 600;
            background-color: var(--color-code-bg);
        }}
        
        .markdown-body table tr:nth-child(even) {{
            background-color: var(--color-code-bg);
        }}
        
        .markdown-body img {{
            max-width: 100%;
            height: auto;
            border-radius: 6px;
        }}
        
        .markdown-body hr {{
            border: 0;
            border-top: 1px solid var(--color-border);
            margin: 24px 0;
        }}
        
        .markdown-body ul, .markdown-body ol {{
            padding-left: 2em;
            margin: 16px 0;
        }}
        
        .markdown-body li {{
            margin: 4px 0;
        }}
        
        .markdown-body input[type="checkbox"] {{
            margin-right: 8px;
            transform: scale(1.1);
        }}
        
        .markdown-body .footnotes {{
            margin-top: 32px;
            padding-top: 16px;
            border-top: 1px solid var(--color-border);
            font-size: 0.9em;
        }}
        
        .toc {{
            background-color: var(--color-code-bg);
            padding: 16px 24px;
            border-radius: 6px;
            margin-bottom: 24px;
        }}
        
        .toc h2 {{
            margin-top: 0;
            margin-bottom: 12px;
            font-size: 1.1em;
        }}
        
        .toc ul {{
            margin: 0;
            padding-left: 1.5em;
            list-style-type: none;
        }}
        
        .toc li {{
            margin: 4px 0;
        }}
        
        .toc a {{
            color: var(--color-link);
            text-decoration: none;
        }}
        
        .toc a:hover {{
            text-decoration: underline;
        }}
        
        @media print {{
            body {{
                max-width: none;
                padding: 1cm;
            }}
            
            .toc {{
                page-break-after: always;
            }}
            
            pre, blockquote {{
                page-break-inside: avoid;
            }}
            
            h1, h2, h3, h4, h5, h6 {{
                page-break-after: avoid;
            }}
        }}
        
        {}
    </style>"#,
            theme_styles,
            custom
        )
    }
}

impl Default for MarkdownExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_html_export() {
        let exporter = MarkdownExporter::new();
        let options = HtmlExportOptions {
            title: Some("Test".to_string()),
            ..Default::default()
        };
        
        let html = exporter.export_html("# Hello\n\nWorld", &options);
        
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }
    
    #[test]
    fn test_toc_generation() {
        let toc = MarkdownExporter::generate_toc("# One\n## Two\n### Three\n# Four");
        
        assert!(toc.contains("One"));
        assert!(toc.contains("Two"));
        assert!(toc.contains("Three"));
        assert!(toc.contains("Four"));
    }
    
    #[test]
    fn test_anchor_generation() {
        assert_eq!(MarkdownExporter::generate_anchor("Hello World"), "hello-world");
        assert_eq!(MarkdownExporter::generate_anchor("Test 123!"), "test-123_");
    }
    
    #[test]
    fn test_suggest_output_path() {
        let input = PathBuf::from("/docs/readme.md");
        let output = MarkdownExporter::suggest_output_path(&input, ExportFormat::Html);
        assert_eq!(output, PathBuf::from("/docs/readme.html"));
    }
}
