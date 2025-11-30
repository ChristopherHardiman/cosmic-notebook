//! Markdown Preview Rendering
//!
//! This module provides functionality to render Markdown content as a rich preview
//! using pulldown-cmark for parsing and cosmic/iced widgets for display.

use std::path::{Path, PathBuf};
use pulldown_cmark::{Parser, Event, Tag, Options, CodeBlockKind, HeadingLevel, CowStr};

// Note: ViewMode is defined in crate::config and re-exported from there
// We extend it here with helper methods via an extension trait

/// Extension trait for ViewMode
pub trait ViewModeExt {
    /// Toggle between Edit and Preview modes
    fn toggle_preview(&self) -> crate::config::ViewMode;
    /// Toggle split view
    fn toggle_split(&self) -> crate::config::ViewMode;
    /// Check if preview is visible
    fn shows_preview(&self) -> bool;
    /// Check if editor is visible
    fn shows_editor(&self) -> bool;
}

impl ViewModeExt for crate::config::ViewMode {
    fn toggle_preview(&self) -> crate::config::ViewMode {
        match self {
            crate::config::ViewMode::Edit => crate::config::ViewMode::Preview,
            crate::config::ViewMode::Preview => crate::config::ViewMode::Edit,
            crate::config::ViewMode::Split => crate::config::ViewMode::Edit,
        }
    }
    
    fn toggle_split(&self) -> crate::config::ViewMode {
        match self {
            crate::config::ViewMode::Split => crate::config::ViewMode::Edit,
            _ => crate::config::ViewMode::Split,
        }
    }
    
    fn shows_preview(&self) -> bool {
        matches!(self, crate::config::ViewMode::Preview | crate::config::ViewMode::Split)
    }
    
    fn shows_editor(&self) -> bool {
        matches!(self, crate::config::ViewMode::Edit | crate::config::ViewMode::Split)
    }
}

/// A rendered element that can be displayed in the preview pane
#[derive(Debug, Clone)]
pub enum PreviewElement {
    /// A paragraph of text with optional styling
    Paragraph(Vec<StyledText>),
    /// A heading with level and content
    Heading {
        level: u8,
        content: Vec<StyledText>,
    },
    /// A code block with optional language
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    /// An inline code span (kept separate for convenience)
    InlineCode(String),
    /// A blockquote
    Blockquote(Vec<PreviewElement>),
    /// An unordered list
    UnorderedList(Vec<ListItem>),
    /// An ordered list with starting number
    OrderedList {
        start: u64,
        items: Vec<ListItem>,
    },
    /// A task list (GFM)
    TaskList(Vec<TaskItem>),
    /// A table
    Table {
        headers: Vec<Vec<StyledText>>,
        rows: Vec<Vec<Vec<StyledText>>>,
        alignments: Vec<TableAlignment>,
    },
    /// A horizontal rule
    HorizontalRule,
    /// An image
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },
    /// A link (for block-level links)
    Link {
        text: Vec<StyledText>,
        url: String,
        title: Option<String>,
    },
    /// Raw HTML (display as code or render carefully)
    Html(String),
    /// Footnote definition
    FootnoteDefinition {
        label: String,
        content: Vec<PreviewElement>,
    },
    /// A thematic break / soft break
    SoftBreak,
    /// Hard line break
    HardBreak,
}

/// A list item
#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Vec<PreviewElement>,
}

/// A task list item
#[derive(Debug, Clone)]
pub struct TaskItem {
    pub checked: bool,
    pub content: Vec<PreviewElement>,
}

/// Table column alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

/// Styled text with formatting
#[derive(Debug, Clone)]
pub struct StyledText {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl StyledText {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italic: false,
            strikethrough: false,
            code: false,
            link: None,
        }
    }
    
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }
    
    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }
    
    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }
    
    pub fn with_code(mut self) -> Self {
        self.code = true;
        self
    }
    
    pub fn with_link(mut self, url: String) -> Self {
        self.link = Some(url);
        self
    }
}

/// State machine for parsing context
#[derive(Debug, Clone)]
struct ParseContext {
    /// Current text style
    bold: bool,
    italic: bool,
    strikethrough: bool,
    code: bool,
    /// Current link URL if inside a link
    link_url: Option<String>,
    /// Accumulated styled text
    text_buffer: Vec<StyledText>,
}

impl Default for ParseContext {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            strikethrough: false,
            code: false,
            link_url: None,
            text_buffer: Vec::new(),
        }
    }
}

impl ParseContext {
    fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        
        let mut styled = StyledText::plain(text);
        if self.bold {
            styled = styled.with_bold();
        }
        if self.italic {
            styled = styled.with_italic();
        }
        if self.strikethrough {
            styled = styled.with_strikethrough();
        }
        if self.code {
            styled = styled.with_code();
        }
        if let Some(ref url) = self.link_url {
            styled = styled.with_link(url.clone());
        }
        self.text_buffer.push(styled);
    }
    
    fn take_buffer(&mut self) -> Vec<StyledText> {
        std::mem::take(&mut self.text_buffer)
    }
}

/// Markdown preview renderer
pub struct PreviewRenderer {
    /// Base path for resolving relative URLs
    base_path: Option<PathBuf>,
    /// Parser options
    options: Options,
}

impl PreviewRenderer {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        Self {
            base_path: None,
            options,
        }
    }
    
    /// Set the base path for resolving relative URLs
    pub fn with_base_path(mut self, path: impl AsRef<Path>) -> Self {
        self.base_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Parse and render Markdown content to preview elements
    pub fn render(&self, markdown: &str) -> Vec<PreviewElement> {
        let parser = Parser::new_ext(markdown, self.options);
        let mut elements = Vec::new();
        let mut context = ParseContext::default();
        let mut element_stack: Vec<ElementBuilder> = Vec::new();
        
        for event in parser {
            match event {
                Event::Start(tag) => {
                    self.handle_start_tag(tag, &mut context, &mut element_stack);
                }
                Event::End(tag) => {
                    if let Some(element) = self.handle_end_tag(tag, &mut context, &mut element_stack) {
                        if element_stack.is_empty() {
                            elements.push(element);
                        } else if let Some(parent) = element_stack.last_mut() {
                            parent.add_child(element);
                        }
                    }
                }
                Event::Text(text) => {
                    context.push_text(&text);
                }
                Event::Code(code) => {
                    let mut styled = StyledText::plain(code.to_string()).with_code();
                    if context.bold {
                        styled = styled.with_bold();
                    }
                    if context.italic {
                        styled = styled.with_italic();
                    }
                    context.text_buffer.push(styled);
                }
                Event::SoftBreak => {
                    context.push_text(" ");
                }
                Event::HardBreak => {
                    // Finalize current text and add break
                    let buffer = context.take_buffer();
                    if !buffer.is_empty() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.add_text(buffer);
                        }
                    }
                    if element_stack.is_empty() {
                        elements.push(PreviewElement::HardBreak);
                    } else if let Some(parent) = element_stack.last_mut() {
                        parent.add_child(PreviewElement::HardBreak);
                    }
                }
                Event::Rule => {
                    if element_stack.is_empty() {
                        elements.push(PreviewElement::HorizontalRule);
                    } else if let Some(parent) = element_stack.last_mut() {
                        parent.add_child(PreviewElement::HorizontalRule);
                    }
                }
                Event::Html(html) => {
                    let element = PreviewElement::Html(html.to_string());
                    if element_stack.is_empty() {
                        elements.push(element);
                    } else if let Some(parent) = element_stack.last_mut() {
                        parent.add_child(element);
                    }
                }
                Event::FootnoteReference(label) => {
                    context.push_text(&format!("[^{}]", label));
                }
                Event::TaskListMarker(checked) => {
                    if let Some(parent) = element_stack.last_mut() {
                        parent.set_task_checked(checked);
                    }
                }
            }
        }
        
        elements
    }
    
    fn handle_start_tag(&self, tag: Tag<'_>, context: &mut ParseContext, stack: &mut Vec<ElementBuilder>) {
        match tag {
            Tag::Paragraph => {
                stack.push(ElementBuilder::Paragraph(Vec::new()));
            }
            Tag::Heading(level, _id, _classes) => {
                let level_num = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                stack.push(ElementBuilder::Heading { level: level_num, content: Vec::new() });
            }
            Tag::BlockQuote => {
                stack.push(ElementBuilder::Blockquote(Vec::new()));
            }
            Tag::CodeBlock(kind) => {
                let language = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                    _ => None,
                };
                stack.push(ElementBuilder::CodeBlock { language, code: String::new() });
            }
            Tag::List(start) => {
                match start {
                    Some(n) => stack.push(ElementBuilder::OrderedList { start: n, items: Vec::new() }),
                    None => stack.push(ElementBuilder::UnorderedList(Vec::new())),
                }
            }
            Tag::Item => {
                stack.push(ElementBuilder::ListItem { content: Vec::new(), is_task: false, checked: false });
            }
            Tag::Table(alignments) => {
                let aligns: Vec<TableAlignment> = alignments.iter().map(|a| {
                    match a {
                        pulldown_cmark::Alignment::None => TableAlignment::None,
                        pulldown_cmark::Alignment::Left => TableAlignment::Left,
                        pulldown_cmark::Alignment::Center => TableAlignment::Center,
                        pulldown_cmark::Alignment::Right => TableAlignment::Right,
                    }
                }).collect();
                stack.push(ElementBuilder::Table {
                    headers: Vec::new(),
                    rows: Vec::new(),
                    alignments: aligns,
                    in_header: false,
                });
            }
            Tag::TableHead => {
                if let Some(ElementBuilder::Table { in_header, .. }) = stack.last_mut() {
                    *in_header = true;
                }
            }
            Tag::TableRow => {
                stack.push(ElementBuilder::TableRow(Vec::new()));
            }
            Tag::TableCell => {
                stack.push(ElementBuilder::TableCell(Vec::new()));
            }
            Tag::Emphasis => {
                context.italic = true;
            }
            Tag::Strong => {
                context.bold = true;
            }
            Tag::Strikethrough => {
                context.strikethrough = true;
            }
            Tag::Link(_link_type, dest_url, _title) => {
                context.link_url = Some(dest_url.to_string());
            }
            Tag::Image(_link_type, dest_url, title) => {
                // Image handled at end tag
                stack.push(ElementBuilder::Image {
                    url: dest_url.to_string(),
                    title: if title.is_empty() { None } else { Some(title.to_string()) },
                    alt: String::new(),
                });
            }
            Tag::FootnoteDefinition(label) => {
                stack.push(ElementBuilder::FootnoteDefinition {
                    label: label.to_string(),
                    content: Vec::new(),
                });
            }
        }
    }
    
    fn handle_end_tag(&self, tag: Tag<'_>, context: &mut ParseContext, stack: &mut Vec<ElementBuilder>) -> Option<PreviewElement> {
        match tag {
            Tag::Paragraph => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::Paragraph(mut content)) = stack.pop() {
                    content.extend(buffer);
                    return Some(PreviewElement::Paragraph(content));
                }
            }
            Tag::Heading(_, _, _) => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::Heading { level, mut content }) = stack.pop() {
                    content.extend(buffer);
                    return Some(PreviewElement::Heading { level, content });
                }
            }
            Tag::BlockQuote => {
                if let Some(ElementBuilder::Blockquote(content)) = stack.pop() {
                    return Some(PreviewElement::Blockquote(content));
                }
            }
            Tag::CodeBlock(_) => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::CodeBlock { language, mut code }) = stack.pop() {
                    // Add any remaining text
                    for styled in buffer {
                        code.push_str(&styled.text);
                    }
                    return Some(PreviewElement::CodeBlock { language, code });
                }
            }
            Tag::List(ordered) => {
                let is_ordered = ordered.is_some();
                match stack.pop() {
                    Some(ElementBuilder::OrderedList { start, items }) if is_ordered => {
                        return Some(PreviewElement::OrderedList { start, items });
                    }
                    Some(ElementBuilder::UnorderedList(items)) if !is_ordered => {
                        return Some(PreviewElement::UnorderedList(items));
                    }
                    _ => {}
                }
            }
            Tag::Item => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::ListItem { mut content, is_task: _, checked: _ }) = stack.pop() {
                    if !buffer.is_empty() {
                        content.push(PreviewElement::Paragraph(buffer));
                    }
                    
                    let item = ListItem { content };
                    
                    // Add to parent list
                    if let Some(parent) = stack.last_mut() {
                        match parent {
                            ElementBuilder::UnorderedList(items) => items.push(item),
                            ElementBuilder::OrderedList { items, .. } => items.push(item),
                            _ => {}
                        }
                    }
                }
            }
            Tag::Table(_) => {
                if let Some(ElementBuilder::Table { headers, rows, alignments, .. }) = stack.pop() {
                    return Some(PreviewElement::Table { headers, rows, alignments });
                }
            }
            Tag::TableHead => {
                if let Some(ElementBuilder::Table { in_header, .. }) = stack.last_mut() {
                    *in_header = false;
                }
            }
            Tag::TableRow => {
                if let Some(ElementBuilder::TableRow(cells)) = stack.pop() {
                    if let Some(ElementBuilder::Table { headers, rows, in_header, .. }) = stack.last_mut() {
                        if *in_header {
                            *headers = cells;
                        } else {
                            rows.push(cells);
                        }
                    }
                }
            }
            Tag::TableCell => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::TableCell(mut content)) = stack.pop() {
                    content.extend(buffer);
                    if let Some(ElementBuilder::TableRow(cells)) = stack.last_mut() {
                        cells.push(content);
                    }
                }
            }
            Tag::Emphasis => {
                context.italic = false;
            }
            Tag::Strong => {
                context.bold = false;
            }
            Tag::Strikethrough => {
                context.strikethrough = false;
            }
            Tag::Link(_, _, _) => {
                context.link_url = None;
            }
            Tag::Image(_, _, _) => {
                let buffer = context.take_buffer();
                if let Some(ElementBuilder::Image { url, title, mut alt }) = stack.pop() {
                    for styled in buffer {
                        alt.push_str(&styled.text);
                    }
                    return Some(PreviewElement::Image { alt, url, title });
                }
            }
            Tag::FootnoteDefinition(_) => {
                if let Some(ElementBuilder::FootnoteDefinition { label, content }) = stack.pop() {
                    return Some(PreviewElement::FootnoteDefinition { label, content });
                }
            }
        }
        None
    }
    
    /// Resolve a URL relative to the base path
    pub fn resolve_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
            return url.to_string();
        }
        
        if let Some(ref base) = self.base_path {
            let path = base.join(url);
            if path.exists() {
                return format!("file://{}", path.display());
            }
        }
        
        url.to_string()
    }
}

impl Default for PreviewRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing preview elements during parsing
#[derive(Debug)]
enum ElementBuilder {
    Paragraph(Vec<StyledText>),
    Heading { level: u8, content: Vec<StyledText> },
    Blockquote(Vec<PreviewElement>),
    CodeBlock { language: Option<String>, code: String },
    UnorderedList(Vec<ListItem>),
    OrderedList { start: u64, items: Vec<ListItem> },
    ListItem { content: Vec<PreviewElement>, is_task: bool, checked: bool },
    Table {
        headers: Vec<Vec<StyledText>>,
        rows: Vec<Vec<Vec<StyledText>>>,
        alignments: Vec<TableAlignment>,
        in_header: bool,
    },
    TableRow(Vec<Vec<StyledText>>),
    TableCell(Vec<StyledText>),
    Image { url: String, title: Option<String>, alt: String },
    FootnoteDefinition { label: String, content: Vec<PreviewElement> },
}

impl ElementBuilder {
    fn add_child(&mut self, element: PreviewElement) {
        match self {
            ElementBuilder::Blockquote(children) => children.push(element),
            ElementBuilder::ListItem { content, .. } => content.push(element),
            ElementBuilder::FootnoteDefinition { content, .. } => content.push(element),
            _ => {}
        }
    }
    
    fn add_text(&mut self, text: Vec<StyledText>) {
        match self {
            ElementBuilder::Paragraph(content) => content.extend(text),
            ElementBuilder::Heading { content, .. } => content.extend(text),
            ElementBuilder::CodeBlock { code, .. } => {
                for t in text {
                    code.push_str(&t.text);
                }
            }
            ElementBuilder::TableCell(content) => content.extend(text),
            ElementBuilder::Image { alt, .. } => {
                for t in text {
                    alt.push_str(&t.text);
                }
            }
            _ => {}
        }
    }
    
    fn set_task_checked(&mut self, checked: bool) {
        if let ElementBuilder::ListItem { is_task, checked: c, .. } = self {
            *is_task = true;
            *c = checked;
        }
    }
}

/// Convert preview elements to HTML
pub struct HtmlExporter {
    /// Include embedded styles
    include_styles: bool,
    /// Embed images as base64
    #[allow(dead_code)]
    embed_images: bool,
}

impl HtmlExporter {
    pub fn new() -> Self {
        Self {
            include_styles: true,
            embed_images: false,
        }
    }
    
    pub fn with_embedded_styles(mut self, include: bool) -> Self {
        self.include_styles = include;
        self
    }
    
    pub fn with_embedded_images(mut self, embed: bool) -> Self {
        self.embed_images = embed;
        self
    }
    
    /// Export markdown to HTML
    pub fn export(&self, markdown: &str, title: Option<&str>) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        
        let styles = if self.include_styles {
            self.get_default_styles()
        } else {
            String::new()
        };
        
        let title = title.unwrap_or("Document");
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    {}
</head>
<body>
    <article class="markdown-body">
{}
    </article>
</body>
</html>"#,
            Self::escape_html(title),
            styles,
            html_output
        )
    }
    
    fn get_default_styles(&self) -> String {
        r#"<style>
        :root {
            --color-bg: #ffffff;
            --color-text: #24292e;
            --color-heading: #24292e;
            --color-link: #0366d6;
            --color-code-bg: #f6f8fa;
            --color-border: #e1e4e8;
            --color-blockquote: #6a737d;
        }
        
        @media (prefers-color-scheme: dark) {
            :root {
                --color-bg: #0d1117;
                --color-text: #c9d1d9;
                --color-heading: #c9d1d9;
                --color-link: #58a6ff;
                --color-code-bg: #161b22;
                --color-border: #30363d;
                --color-blockquote: #8b949e;
            }
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            font-size: 16px;
            line-height: 1.6;
            color: var(--color-text);
            background-color: var(--color-bg);
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
        }
        
        .markdown-body h1, .markdown-body h2, .markdown-body h3,
        .markdown-body h4, .markdown-body h5, .markdown-body h6 {
            color: var(--color-heading);
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }
        
        .markdown-body h1 { font-size: 2em; border-bottom: 1px solid var(--color-border); padding-bottom: .3em; }
        .markdown-body h2 { font-size: 1.5em; border-bottom: 1px solid var(--color-border); padding-bottom: .3em; }
        .markdown-body h3 { font-size: 1.25em; }
        .markdown-body h4 { font-size: 1em; }
        .markdown-body h5 { font-size: .875em; }
        .markdown-body h6 { font-size: .85em; color: var(--color-blockquote); }
        
        .markdown-body a {
            color: var(--color-link);
            text-decoration: none;
        }
        
        .markdown-body a:hover {
            text-decoration: underline;
        }
        
        .markdown-body code {
            background-color: var(--color-code-bg);
            padding: .2em .4em;
            border-radius: 3px;
            font-size: 85%;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
        }
        
        .markdown-body pre {
            background-color: var(--color-code-bg);
            padding: 16px;
            overflow: auto;
            border-radius: 6px;
            line-height: 1.45;
        }
        
        .markdown-body pre code {
            background: transparent;
            padding: 0;
            border-radius: 0;
        }
        
        .markdown-body blockquote {
            margin: 0;
            padding: 0 1em;
            color: var(--color-blockquote);
            border-left: .25em solid var(--color-border);
        }
        
        .markdown-body table {
            border-collapse: collapse;
            width: 100%;
            margin: 16px 0;
        }
        
        .markdown-body table th,
        .markdown-body table td {
            padding: 6px 13px;
            border: 1px solid var(--color-border);
        }
        
        .markdown-body table tr:nth-child(even) {
            background-color: var(--color-code-bg);
        }
        
        .markdown-body img {
            max-width: 100%;
            height: auto;
        }
        
        .markdown-body hr {
            border: 0;
            border-top: 1px solid var(--color-border);
            margin: 24px 0;
        }
        
        .markdown-body ul, .markdown-body ol {
            padding-left: 2em;
            margin: 16px 0;
        }
        
        .markdown-body li {
            margin: 4px 0;
        }
        
        .markdown-body input[type="checkbox"] {
            margin-right: 8px;
        }
    </style>"#.to_string()
    }
    
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

impl Default for HtmlExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preview_render() {
        let renderer = PreviewRenderer::new();
        let elements = renderer.render("# Hello\n\nThis is **bold** text.");
        
        assert!(!elements.is_empty());
        // First element should be heading
        assert!(matches!(elements[0], PreviewElement::Heading { level: 1, .. }));
    }
    
    #[test]
    fn test_html_export() {
        let exporter = HtmlExporter::new();
        let html = exporter.export("# Test\n\nParagraph.", Some("Test Doc"));
        
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("<p>Paragraph.</p>"));
        assert!(html.contains("<title>Test Doc</title>"));
    }
}
