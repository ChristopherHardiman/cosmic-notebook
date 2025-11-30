//! Syntax Highlighting for Markdown
//!
//! This module provides tokenization and syntax highlighting for Markdown text,
//! supporting both standard Markdown and GitHub Flavored Markdown (GFM) extensions.

use std::collections::HashMap;
use cosmic::iced_core::Color;

/// Types of Markdown tokens recognized by the tokenizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Standard Markdown
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Bold,
    Italic,
    BoldItalic,
    InlineCode,
    CodeBlockDelimiter,
    CodeBlockContent,
    CodeBlockLanguage,
    Blockquote,
    UnorderedListMarker,
    OrderedListMarker,
    LinkText,
    LinkUrl,
    ImageAlt,
    ImageUrl,
    HorizontalRule,
    
    // GitHub Flavored Markdown
    Strikethrough,
    TaskListUnchecked,
    TaskListChecked,
    TableDelimiter,
    TableCell,
    Autolink,
    Footnote,
    FootnoteReference,
    
    // Special
    Frontmatter,
    PlainText,
    Escape,
}

/// A single token in a line of Markdown text
#[derive(Debug, Clone)]
pub struct Token {
    /// Type of this token
    pub token_type: TokenType,
    /// Start byte offset in the line
    pub start: usize,
    /// End byte offset in the line (exclusive)
    pub end: usize,
    /// Nested style (e.g., bold inside heading)
    pub nested_style: Option<Box<Token>>,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, end: usize) -> Self {
        Self {
            token_type,
            start,
            end,
            nested_style: None,
        }
    }
    
    pub fn with_nested(mut self, nested: Token) -> Self {
        self.nested_style = Some(Box::new(nested));
        self
    }
    
    /// Length of this token in bytes
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Style for rendering a token
#[derive(Debug, Clone)]
pub struct TokenStyle {
    pub foreground: Color,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Default for TokenStyle {
    fn default() -> Self {
        Self {
            foreground: Color::BLACK,
            background: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

/// Color scheme for syntax highlighting
#[derive(Debug, Clone)]
pub struct SyntaxColorScheme {
    pub name: String,
    pub is_dark: bool,
    pub styles: HashMap<TokenType, TokenStyle>,
}

impl SyntaxColorScheme {
    /// Create the default light color scheme
    pub fn light() -> Self {
        let mut styles = HashMap::new();
        
        // Headings - dark blue
        let heading_color = Color::from_rgb(0.0, 0.0, 0.55);
        for (token, size_boost) in [
            (TokenType::Heading1, true),
            (TokenType::Heading2, true),
            (TokenType::Heading3, true),
            (TokenType::Heading4, false),
            (TokenType::Heading5, false),
            (TokenType::Heading6, false),
        ] {
            styles.insert(token, TokenStyle {
                foreground: heading_color,
                bold: size_boost,
                ..Default::default()
            });
        }
        
        // Emphasis
        styles.insert(TokenType::Italic, TokenStyle {
            foreground: Color::from_rgb(0.3, 0.3, 0.3),
            italic: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::Bold, TokenStyle {
            foreground: Color::from_rgb(0.2, 0.2, 0.2),
            bold: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::BoldItalic, TokenStyle {
            foreground: Color::from_rgb(0.2, 0.2, 0.2),
            bold: true,
            italic: true,
            ..Default::default()
        });
        
        // Code
        let code_bg = Color::from_rgba(0.9, 0.9, 0.9, 1.0);
        styles.insert(TokenType::InlineCode, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.2, 0.2),
            background: Some(code_bg),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockDelimiter, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockLanguage, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.0, 0.6),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockContent, TokenStyle {
            foreground: Color::from_rgb(0.3, 0.3, 0.3),
            background: Some(code_bg),
            ..Default::default()
        });
        
        // Links
        styles.insert(TokenType::LinkText, TokenStyle {
            foreground: Color::from_rgb(0.0, 0.4, 0.8),
            underline: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::LinkUrl, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Images
        styles.insert(TokenType::ImageAlt, TokenStyle {
            foreground: Color::from_rgb(0.0, 0.5, 0.0),
            ..Default::default()
        });
        
        styles.insert(TokenType::ImageUrl, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Lists
        let accent = Color::from_rgb(0.8, 0.4, 0.0);
        styles.insert(TokenType::UnorderedListMarker, TokenStyle {
            foreground: accent,
            bold: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::OrderedListMarker, TokenStyle {
            foreground: accent,
            bold: true,
            ..Default::default()
        });
        
        // Blockquote
        styles.insert(TokenType::Blockquote, TokenStyle {
            foreground: Color::from_rgb(0.4, 0.4, 0.4),
            italic: true,
            ..Default::default()
        });
        
        // Horizontal rule
        styles.insert(TokenType::HorizontalRule, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.6, 0.6),
            ..Default::default()
        });
        
        // GFM - Strikethrough
        styles.insert(TokenType::Strikethrough, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            strikethrough: true,
            ..Default::default()
        });
        
        // Task lists
        styles.insert(TokenType::TaskListUnchecked, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.6, 0.6),
            ..Default::default()
        });
        
        styles.insert(TokenType::TaskListChecked, TokenStyle {
            foreground: Color::from_rgb(0.0, 0.6, 0.0),
            ..Default::default()
        });
        
        // Tables
        styles.insert(TokenType::TableDelimiter, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Autolinks
        styles.insert(TokenType::Autolink, TokenStyle {
            foreground: Color::from_rgb(0.0, 0.4, 0.8),
            underline: true,
            ..Default::default()
        });
        
        // Footnotes
        styles.insert(TokenType::Footnote, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.0, 0.6),
            ..Default::default()
        });
        
        styles.insert(TokenType::FootnoteReference, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.0, 0.6),
            ..Default::default()
        });
        
        // Frontmatter
        styles.insert(TokenType::Frontmatter, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.0, 0.6),
            ..Default::default()
        });
        
        // Plain text
        styles.insert(TokenType::PlainText, TokenStyle::default());
        
        // Escape sequences
        styles.insert(TokenType::Escape, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.3, 0.0),
            ..Default::default()
        });
        
        Self {
            name: "Light".to_string(),
            is_dark: false,
            styles,
        }
    }
    
    /// Create the default dark color scheme
    pub fn dark() -> Self {
        let mut styles = HashMap::new();
        
        // Headings - light blue
        let heading_color = Color::from_rgb(0.4, 0.7, 1.0);
        for (token, size_boost) in [
            (TokenType::Heading1, true),
            (TokenType::Heading2, true),
            (TokenType::Heading3, true),
            (TokenType::Heading4, false),
            (TokenType::Heading5, false),
            (TokenType::Heading6, false),
        ] {
            styles.insert(token, TokenStyle {
                foreground: heading_color,
                bold: size_boost,
                ..Default::default()
            });
        }
        
        // Emphasis
        styles.insert(TokenType::Italic, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.8, 0.8),
            italic: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::Bold, TokenStyle {
            foreground: Color::from_rgb(0.95, 0.95, 0.95),
            bold: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::BoldItalic, TokenStyle {
            foreground: Color::from_rgb(0.95, 0.95, 0.95),
            bold: true,
            italic: true,
            ..Default::default()
        });
        
        // Code
        let code_bg = Color::from_rgba(0.2, 0.2, 0.2, 1.0);
        styles.insert(TokenType::InlineCode, TokenStyle {
            foreground: Color::from_rgb(1.0, 0.5, 0.5),
            background: Some(code_bg),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockDelimiter, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.6, 0.6),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockLanguage, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.4, 0.8),
            ..Default::default()
        });
        
        styles.insert(TokenType::CodeBlockContent, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.8, 0.8),
            background: Some(code_bg),
            ..Default::default()
        });
        
        // Links
        styles.insert(TokenType::LinkText, TokenStyle {
            foreground: Color::from_rgb(0.4, 0.8, 1.0),
            underline: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::LinkUrl, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Images
        styles.insert(TokenType::ImageAlt, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.9, 0.5),
            ..Default::default()
        });
        
        styles.insert(TokenType::ImageUrl, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Lists
        let accent = Color::from_rgb(1.0, 0.6, 0.2);
        styles.insert(TokenType::UnorderedListMarker, TokenStyle {
            foreground: accent,
            bold: true,
            ..Default::default()
        });
        
        styles.insert(TokenType::OrderedListMarker, TokenStyle {
            foreground: accent,
            bold: true,
            ..Default::default()
        });
        
        // Blockquote
        styles.insert(TokenType::Blockquote, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.6, 0.6),
            italic: true,
            ..Default::default()
        });
        
        // Horizontal rule
        styles.insert(TokenType::HorizontalRule, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // GFM - Strikethrough
        styles.insert(TokenType::Strikethrough, TokenStyle {
            foreground: Color::from_rgb(0.6, 0.6, 0.6),
            strikethrough: true,
            ..Default::default()
        });
        
        // Task lists
        styles.insert(TokenType::TaskListUnchecked, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        styles.insert(TokenType::TaskListChecked, TokenStyle {
            foreground: Color::from_rgb(0.4, 0.9, 0.4),
            ..Default::default()
        });
        
        // Tables
        styles.insert(TokenType::TableDelimiter, TokenStyle {
            foreground: Color::from_rgb(0.5, 0.5, 0.5),
            ..Default::default()
        });
        
        // Autolinks
        styles.insert(TokenType::Autolink, TokenStyle {
            foreground: Color::from_rgb(0.4, 0.8, 1.0),
            underline: true,
            ..Default::default()
        });
        
        // Footnotes
        styles.insert(TokenType::Footnote, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.4, 0.8),
            ..Default::default()
        });
        
        styles.insert(TokenType::FootnoteReference, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.4, 0.8),
            ..Default::default()
        });
        
        // Frontmatter
        styles.insert(TokenType::Frontmatter, TokenStyle {
            foreground: Color::from_rgb(0.8, 0.4, 0.8),
            ..Default::default()
        });
        
        // Plain text
        styles.insert(TokenType::PlainText, TokenStyle {
            foreground: Color::from_rgb(0.9, 0.9, 0.9),
            ..Default::default()
        });
        
        // Escape sequences
        styles.insert(TokenType::Escape, TokenStyle {
            foreground: Color::from_rgb(0.9, 0.6, 0.3),
            ..Default::default()
        });
        
        Self {
            name: "Dark".to_string(),
            is_dark: true,
            styles,
        }
    }
    
    /// Get the style for a token type
    pub fn get_style(&self, token_type: TokenType) -> TokenStyle {
        self.styles.get(&token_type).cloned().unwrap_or_default()
    }
}

/// Line-level tokenization state for multi-line constructs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineState {
    /// Normal state
    Normal,
    /// Inside a fenced code block
    InCodeBlock { fence_char: char, fence_count: usize },
    /// Inside a frontmatter block
    InFrontmatter,
}

/// Cached tokens for a single line
#[derive(Debug, Clone)]
pub struct LineTokens {
    /// The tokens for this line
    pub tokens: Vec<Token>,
    /// State at the end of this line (for continuation)
    pub end_state: LineState,
    /// Hash of the line content for cache invalidation
    content_hash: u64,
}

impl LineTokens {
    pub fn new(tokens: Vec<Token>, end_state: LineState, content_hash: u64) -> Self {
        Self {
            tokens,
            end_state,
            content_hash,
        }
    }
}

/// The main syntax tokenizer for Markdown
pub struct MarkdownTokenizer {
    /// Cached tokens per line
    line_cache: HashMap<usize, LineTokens>,
}

impl MarkdownTokenizer {
    pub fn new() -> Self {
        Self {
            line_cache: HashMap::new(),
        }
    }
    
    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.line_cache.clear();
    }
    
    /// Invalidate cache for a specific line and all following lines
    pub fn invalidate_from_line(&mut self, line_num: usize) {
        self.line_cache.retain(|&k, _| k < line_num);
    }
    
    /// Calculate a simple hash for content
    fn hash_content(content: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get the starting state for a line
    fn get_start_state(&self, line_num: usize) -> LineState {
        if line_num == 0 {
            LineState::Normal
        } else if let Some(prev) = self.line_cache.get(&(line_num - 1)) {
            prev.end_state
        } else {
            LineState::Normal
        }
    }
    
    /// Tokenize a single line
    pub fn tokenize_line(&mut self, line_num: usize, content: &str, start_state: LineState) -> &LineTokens {
        let content_hash = Self::hash_content(content);
        
        // Check cache
        if let Some(cached) = self.line_cache.get(&line_num) {
            if cached.content_hash == content_hash {
                return self.line_cache.get(&line_num).unwrap();
            }
        }
        
        let (tokens, end_state) = self.do_tokenize(content, start_state);
        let line_tokens = LineTokens::new(tokens, end_state, content_hash);
        self.line_cache.insert(line_num, line_tokens);
        self.line_cache.get(&line_num).unwrap()
    }
    
    /// Tokenize all lines in a document
    pub fn tokenize_document(&mut self, lines: &[&str]) -> Vec<LineTokens> {
        let mut result = Vec::with_capacity(lines.len());
        let mut state = LineState::Normal;
        
        for (i, line) in lines.iter().enumerate() {
            let line_tokens = self.tokenize_line(i, line, state);
            state = line_tokens.end_state;
            result.push(line_tokens.clone());
        }
        
        result
    }
    
    /// Perform the actual tokenization
    fn do_tokenize(&self, line: &str, state: LineState) -> (Vec<Token>, LineState) {
        let mut tokens = Vec::new();
        let trimmed = line.trim_start();
        let leading_spaces = line.len() - trimmed.len();
        
        // Handle state-based continuation
        match state {
            LineState::InCodeBlock { fence_char, fence_count } => {
                // Check if this line ends the code block
                let fence = fence_char.to_string().repeat(fence_count);
                if trimmed.starts_with(&fence) && trimmed.trim() == fence.trim() {
                    tokens.push(Token::new(TokenType::CodeBlockDelimiter, 0, line.len()));
                    return (tokens, LineState::Normal);
                }
                tokens.push(Token::new(TokenType::CodeBlockContent, 0, line.len()));
                return (tokens, state);
            }
            LineState::InFrontmatter => {
                if trimmed == "---" {
                    tokens.push(Token::new(TokenType::Frontmatter, 0, line.len()));
                    return (tokens, LineState::Normal);
                }
                tokens.push(Token::new(TokenType::Frontmatter, 0, line.len()));
                return (tokens, state);
            }
            LineState::Normal => {}
        }
        
        // Check for frontmatter start (only at beginning of document)
        if line == "---" {
            tokens.push(Token::new(TokenType::Frontmatter, 0, line.len()));
            return (tokens, LineState::InFrontmatter);
        }
        
        // Check for code block start
        if let Some(fence_info) = self.parse_code_fence(trimmed) {
            tokens.push(Token::new(TokenType::CodeBlockDelimiter, 0, leading_spaces + 3));
            if !fence_info.language.is_empty() {
                let lang_start = line.find(&fence_info.language).unwrap_or(leading_spaces + 3);
                tokens.push(Token::new(
                    TokenType::CodeBlockLanguage,
                    lang_start,
                    lang_start + fence_info.language.len(),
                ));
            }
            return (tokens, LineState::InCodeBlock {
                fence_char: fence_info.char,
                fence_count: fence_info.count,
            });
        }
        
        // Check for heading
        if let Some((level, content_start)) = self.parse_heading(trimmed) {
            let token_type = match level {
                1 => TokenType::Heading1,
                2 => TokenType::Heading2,
                3 => TokenType::Heading3,
                4 => TokenType::Heading4,
                5 => TokenType::Heading5,
                _ => TokenType::Heading6,
            };
            tokens.push(Token::new(token_type, 0, line.len()));
            // Also tokenize inline elements within heading
            let inline_tokens = self.tokenize_inline(&trimmed[content_start..], leading_spaces + content_start);
            tokens.extend(inline_tokens);
            return (tokens, LineState::Normal);
        }
        
        // Check for horizontal rule
        if self.is_horizontal_rule(trimmed) {
            tokens.push(Token::new(TokenType::HorizontalRule, 0, line.len()));
            return (tokens, LineState::Normal);
        }
        
        // Check for blockquote
        if trimmed.starts_with('>') {
            tokens.push(Token::new(TokenType::Blockquote, leading_spaces, leading_spaces + 1));
            let content_start = if trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') { 2 } else { 1 };
            let inline_tokens = self.tokenize_inline(&trimmed[content_start..], leading_spaces + content_start);
            tokens.extend(inline_tokens);
            return (tokens, LineState::Normal);
        }
        
        // Check for unordered list
        if let Some((marker_end, is_task, is_checked)) = self.parse_unordered_list(trimmed) {
            if is_task {
                let token_type = if is_checked {
                    TokenType::TaskListChecked
                } else {
                    TokenType::TaskListUnchecked
                };
                tokens.push(Token::new(token_type, leading_spaces, leading_spaces + marker_end));
            } else {
                tokens.push(Token::new(TokenType::UnorderedListMarker, leading_spaces, leading_spaces + marker_end));
            }
            let inline_tokens = self.tokenize_inline(&trimmed[marker_end..], leading_spaces + marker_end);
            tokens.extend(inline_tokens);
            return (tokens, LineState::Normal);
        }
        
        // Check for ordered list
        if let Some(marker_end) = self.parse_ordered_list(trimmed) {
            tokens.push(Token::new(TokenType::OrderedListMarker, leading_spaces, leading_spaces + marker_end));
            let inline_tokens = self.tokenize_inline(&trimmed[marker_end..], leading_spaces + marker_end);
            tokens.extend(inline_tokens);
            return (tokens, LineState::Normal);
        }
        
        // Check for table row
        if trimmed.contains('|') && self.is_table_row(trimmed) {
            tokens.push(Token::new(TokenType::TableDelimiter, 0, line.len()));
            return (tokens, LineState::Normal);
        }
        
        // Regular line - tokenize inline elements
        let inline_tokens = self.tokenize_inline(line, 0);
        if inline_tokens.is_empty() && !line.is_empty() {
            tokens.push(Token::new(TokenType::PlainText, 0, line.len()));
        } else {
            tokens.extend(inline_tokens);
        }
        
        (tokens, LineState::Normal)
    }
    
    /// Parse a code fence (``` or ~~~)
    fn parse_code_fence(&self, line: &str) -> Option<CodeFenceInfo> {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 3 {
            return None;
        }
        
        let fence_char = chars[0];
        if fence_char != '`' && fence_char != '~' {
            return None;
        }
        
        let mut count = 0;
        for c in &chars {
            if *c == fence_char {
                count += 1;
            } else {
                break;
            }
        }
        
        if count < 3 {
            return None;
        }
        
        let language = line[count..].trim().to_string();
        
        Some(CodeFenceInfo {
            char: fence_char,
            count,
            language,
        })
    }
    
    /// Parse a heading
    fn parse_heading(&self, line: &str) -> Option<(usize, usize)> {
        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() || chars[0] != '#' {
            return None;
        }
        
        let mut level = 0;
        for c in &chars {
            if *c == '#' {
                level += 1;
            } else {
                break;
            }
        }
        
        if level > 6 {
            return None;
        }
        
        // Must have space after #s or be empty
        if level < chars.len() && chars[level] != ' ' {
            return None;
        }
        
        let content_start = if level < chars.len() { level + 1 } else { level };
        Some((level, content_start))
    }
    
    /// Check if line is a horizontal rule
    fn is_horizontal_rule(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 3 {
            return false;
        }
        
        let chars: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.len() < 3 {
            return false;
        }
        
        let first = chars[0];
        if first != '-' && first != '*' && first != '_' {
            return false;
        }
        
        chars.iter().all(|c| *c == first)
    }
    
    /// Parse unordered list item, returns (marker_end, is_task, is_checked)
    fn parse_unordered_list(&self, line: &str) -> Option<(usize, bool, bool)> {
        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return None;
        }
        
        let marker = chars[0];
        if marker != '-' && marker != '*' && marker != '+' {
            return None;
        }
        
        if chars.len() < 2 || chars[1] != ' ' {
            return None;
        }
        
        // Check for task list
        if chars.len() >= 5 && chars[2] == '[' {
            if chars[3] == ' ' && chars[4] == ']' {
                let marker_end = if chars.len() > 5 && chars[5] == ' ' { 6 } else { 5 };
                return Some((marker_end, true, false));
            }
            if (chars[3] == 'x' || chars[3] == 'X') && chars[4] == ']' {
                let marker_end = if chars.len() > 5 && chars[5] == ' ' { 6 } else { 5 };
                return Some((marker_end, true, true));
            }
        }
        
        Some((2, false, false))
    }
    
    /// Parse ordered list item
    fn parse_ordered_list(&self, line: &str) -> Option<usize> {
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();
        
        // Parse digits
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        
        if i == 0 || i >= chars.len() {
            return None;
        }
        
        // Check for . or ) followed by space
        if (chars[i] == '.' || chars[i] == ')') && i + 1 < chars.len() && chars[i + 1] == ' ' {
            return Some(i + 2);
        }
        
        None
    }
    
    /// Check if line is a table row
    fn is_table_row(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // Simple heuristic: contains | and doesn't look like other constructs
        trimmed.starts_with('|') || trimmed.ends_with('|') || trimmed.contains(" | ")
    }
    
    /// Tokenize inline elements
    fn tokenize_inline(&self, text: &str, offset: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = text.chars().collect();
        
        while pos < chars.len() {
            // Check for escape
            if chars[pos] == '\\' && pos + 1 < chars.len() {
                tokens.push(Token::new(TokenType::Escape, offset + pos, offset + pos + 2));
                pos += 2;
                continue;
            }
            
            // Check for inline code
            if chars[pos] == '`' {
                if let Some((end, _)) = self.find_inline_code(&chars, pos) {
                    tokens.push(Token::new(TokenType::InlineCode, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for bold italic (***)
            if pos + 2 < chars.len() && chars[pos] == '*' && chars[pos + 1] == '*' && chars[pos + 2] == '*' {
                if let Some(end) = self.find_closing(&chars, pos + 3, "***") {
                    tokens.push(Token::new(TokenType::BoldItalic, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for bold (**)
            if pos + 1 < chars.len() && chars[pos] == '*' && chars[pos + 1] == '*' {
                if let Some(end) = self.find_closing(&chars, pos + 2, "**") {
                    tokens.push(Token::new(TokenType::Bold, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for bold (__)
            if pos + 1 < chars.len() && chars[pos] == '_' && chars[pos + 1] == '_' {
                if let Some(end) = self.find_closing(&chars, pos + 2, "__") {
                    tokens.push(Token::new(TokenType::Bold, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for strikethrough (~~)
            if pos + 1 < chars.len() && chars[pos] == '~' && chars[pos + 1] == '~' {
                if let Some(end) = self.find_closing(&chars, pos + 2, "~~") {
                    tokens.push(Token::new(TokenType::Strikethrough, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for italic (*)
            if chars[pos] == '*' {
                if let Some(end) = self.find_closing(&chars, pos + 1, "*") {
                    tokens.push(Token::new(TokenType::Italic, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for italic (_)
            if chars[pos] == '_' {
                if let Some(end) = self.find_closing(&chars, pos + 1, "_") {
                    tokens.push(Token::new(TokenType::Italic, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for image ![]()
            if chars[pos] == '!' && pos + 1 < chars.len() && chars[pos + 1] == '[' {
                if let Some((alt_end, url_end)) = self.find_link(&chars, pos + 1) {
                    tokens.push(Token::new(TokenType::ImageAlt, offset + pos, offset + alt_end));
                    tokens.push(Token::new(TokenType::ImageUrl, offset + alt_end, offset + url_end));
                    pos = url_end;
                    continue;
                }
            }
            
            // Check for link []()
            if chars[pos] == '[' {
                if let Some((text_end, url_end)) = self.find_link(&chars, pos) {
                    tokens.push(Token::new(TokenType::LinkText, offset + pos, offset + text_end));
                    tokens.push(Token::new(TokenType::LinkUrl, offset + text_end, offset + url_end));
                    pos = url_end;
                    continue;
                }
            }
            
            // Check for footnote reference [^id]
            if chars[pos] == '[' && pos + 1 < chars.len() && chars[pos + 1] == '^' {
                if let Some(end) = self.find_footnote_ref(&chars, pos) {
                    tokens.push(Token::new(TokenType::FootnoteReference, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            // Check for autolink
            if self.is_autolink_start(&chars, pos) {
                if let Some(end) = self.find_autolink_end(&chars, pos) {
                    tokens.push(Token::new(TokenType::Autolink, offset + pos, offset + end));
                    pos = end;
                    continue;
                }
            }
            
            pos += 1;
        }
        
        tokens
    }
    
    /// Find inline code end
    fn find_inline_code(&self, chars: &[char], start: usize) -> Option<(usize, usize)> {
        let mut backticks = 0;
        let mut pos = start;
        
        while pos < chars.len() && chars[pos] == '`' {
            backticks += 1;
            pos += 1;
        }
        
        // Find closing backticks
        let mut count = 0;
        while pos < chars.len() {
            if chars[pos] == '`' {
                count += 1;
                if count == backticks {
                    return Some((pos + 1, backticks));
                }
            } else {
                count = 0;
            }
            pos += 1;
        }
        
        None
    }
    
    /// Find closing marker
    fn find_closing(&self, chars: &[char], start: usize, marker: &str) -> Option<usize> {
        let marker_chars: Vec<char> = marker.chars().collect();
        let marker_len = marker_chars.len();
        
        let mut pos = start;
        while pos + marker_len <= chars.len() {
            let mut matches = true;
            for (i, mc) in marker_chars.iter().enumerate() {
                if chars[pos + i] != *mc {
                    matches = false;
                    break;
                }
            }
            if matches {
                return Some(pos + marker_len);
            }
            pos += 1;
        }
        
        None
    }
    
    /// Find link end [text](url)
    fn find_link(&self, chars: &[char], start: usize) -> Option<(usize, usize)> {
        if chars[start] != '[' {
            return None;
        }
        
        let mut pos = start + 1;
        let mut bracket_depth = 1;
        
        // Find ]
        while pos < chars.len() && bracket_depth > 0 {
            match chars[pos] {
                '[' => bracket_depth += 1,
                ']' => bracket_depth -= 1,
                '\\' => pos += 1, // Skip escaped char
                _ => {}
            }
            pos += 1;
        }
        
        if bracket_depth != 0 {
            return None;
        }
        
        let text_end = pos;
        
        // Check for (
        if pos >= chars.len() || chars[pos] != '(' {
            return None;
        }
        pos += 1;
        
        // Find )
        let mut paren_depth = 1;
        while pos < chars.len() && paren_depth > 0 {
            match chars[pos] {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                '\\' => pos += 1, // Skip escaped char
                _ => {}
            }
            pos += 1;
        }
        
        if paren_depth != 0 {
            return None;
        }
        
        Some((text_end, pos))
    }
    
    /// Find footnote reference end
    fn find_footnote_ref(&self, chars: &[char], start: usize) -> Option<usize> {
        if start + 2 >= chars.len() || chars[start] != '[' || chars[start + 1] != '^' {
            return None;
        }
        
        let mut pos = start + 2;
        while pos < chars.len() {
            if chars[pos] == ']' {
                return Some(pos + 1);
            }
            if !chars[pos].is_alphanumeric() && chars[pos] != '-' && chars[pos] != '_' {
                return None;
            }
            pos += 1;
        }
        
        None
    }
    
    /// Check if position could be start of autolink
    fn is_autolink_start(&self, chars: &[char], pos: usize) -> bool {
        // Check for http:// or https://
        let remaining: String = chars[pos..].iter().collect();
        remaining.starts_with("http://") || remaining.starts_with("https://")
    }
    
    /// Find end of autolink
    fn find_autolink_end(&self, chars: &[char], start: usize) -> Option<usize> {
        let mut pos = start;
        
        while pos < chars.len() {
            let c = chars[pos];
            // URLs end at whitespace or certain punctuation
            if c.is_whitespace() || c == '<' || c == '>' || c == '"' || c == '\'' {
                break;
            }
            pos += 1;
        }
        
        // Remove trailing punctuation that's likely not part of URL
        while pos > start {
            let c = chars[pos - 1];
            if c == '.' || c == ',' || c == ':' || c == ';' || c == '!' || c == '?' || c == ')' {
                pos -= 1;
            } else {
                break;
            }
        }
        
        if pos > start + 7 {
            Some(pos)
        } else {
            None
        }
    }
}

impl Default for MarkdownTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a code fence
struct CodeFenceInfo {
    char: char,
    count: usize,
    language: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_heading_tokenization() {
        let mut tokenizer = MarkdownTokenizer::new();
        let line_tokens = tokenizer.tokenize_line(0, "# Heading 1", LineState::Normal);
        assert!(!line_tokens.tokens.is_empty());
        assert_eq!(line_tokens.tokens[0].token_type, TokenType::Heading1);
    }
    
    #[test]
    fn test_code_block() {
        let mut tokenizer = MarkdownTokenizer::new();
        
        let line1 = tokenizer.tokenize_line(0, "```rust", LineState::Normal);
        assert_eq!(line1.end_state, LineState::InCodeBlock { fence_char: '`', fence_count: 3 });
        
        let line2 = tokenizer.tokenize_line(1, "let x = 42;", line1.end_state);
        assert_eq!(line2.tokens[0].token_type, TokenType::CodeBlockContent);
        
        let line3 = tokenizer.tokenize_line(2, "```", line2.end_state);
        assert_eq!(line3.end_state, LineState::Normal);
    }
    
    #[test]
    fn test_inline_elements() {
        let mut tokenizer = MarkdownTokenizer::new();
        
        let line = tokenizer.tokenize_line(0, "This is **bold** and *italic*", LineState::Normal);
        let bold_count = line.tokens.iter().filter(|t| t.token_type == TokenType::Bold).count();
        let italic_count = line.tokens.iter().filter(|t| t.token_type == TokenType::Italic).count();
        
        assert_eq!(bold_count, 1);
        assert_eq!(italic_count, 1);
    }
    
    #[test]
    fn test_task_list() {
        let mut tokenizer = MarkdownTokenizer::new();
        
        let unchecked = tokenizer.tokenize_line(0, "- [ ] Todo item", LineState::Normal);
        assert!(unchecked.tokens.iter().any(|t| t.token_type == TokenType::TaskListUnchecked));
        
        let checked = tokenizer.tokenize_line(1, "- [x] Done item", LineState::Normal);
        assert!(checked.tokens.iter().any(|t| t.token_type == TokenType::TaskListChecked));
    }
}
