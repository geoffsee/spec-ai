//! Markdown rendering to styled text
//!
//! Converts markdown text to styled `Text`/`Line`/`Span` structures for terminal rendering.

use super::{Color, Line, Span, Style, Text};
use unicode_width::UnicodeWidthStr;

/// Configuration for markdown rendering
#[derive(Debug, Clone)]
pub struct MarkdownConfig {
    /// Style for bold text
    pub bold_style: Style,
    /// Style for italic text
    pub italic_style: Style,
    /// Style for inline code
    pub code_style: Style,
    /// Style for code block content
    pub code_block_style: Style,
    /// Style for code block language label
    pub code_lang_style: Style,
    /// Style for headers (H1)
    pub h1_style: Style,
    /// Style for headers (H2)
    pub h2_style: Style,
    /// Style for headers (H3+)
    pub h3_style: Style,
    /// Style for links
    pub link_style: Style,
    /// Style for list bullets
    pub bullet_style: Style,
    /// Maximum width for wrapping (0 = no wrapping)
    pub max_width: usize,
    /// Prefix for continuation lines when wrapping
    pub wrap_prefix: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            bold_style: Style::new().bold(),
            italic_style: Style::new().italic(),
            code_style: Style::new().fg(Color::Yellow),
            code_block_style: Style::new().fg(Color::Yellow),
            code_lang_style: Style::new().fg(Color::DarkGrey).italic(),
            h1_style: Style::new().fg(Color::Cyan).bold(),
            h2_style: Style::new().fg(Color::Cyan).bold(),
            h3_style: Style::new().fg(Color::Cyan),
            link_style: Style::new().fg(Color::Blue).underlined(),
            bullet_style: Style::new().fg(Color::DarkGrey),
            max_width: 0,
            wrap_prefix: "  ".to_string(),
        }
    }
}

impl MarkdownConfig {
    /// Create a new config with default styles
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum width for line wrapping
    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    /// Set the prefix for wrapped continuation lines
    pub fn wrap_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.wrap_prefix = prefix.into();
        self
    }
}

/// Parse markdown text and return styled Text
pub fn parse_markdown(input: &str, config: &MarkdownConfig) -> Text {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang = String::new();

    for line in input.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                in_code_block = false;
                code_block_lang.clear();
            } else {
                // Start of code block
                in_code_block = true;
                code_block_lang = line.trim_start_matches('`').trim().to_string();
                if !code_block_lang.is_empty() {
                    lines.push(Line::from_spans([Span::styled(
                        format!("  {}", code_block_lang),
                        config.code_lang_style,
                    )]));
                }
            }
            continue;
        }

        if in_code_block {
            // Code block content - preserve as-is with code style
            lines.push(Line::from_spans([Span::styled(
                format!("  {}", line),
                config.code_block_style,
            )]));
        } else {
            // Parse the line for markdown elements
            let parsed_lines = parse_line(line, config);
            lines.extend(parsed_lines);
        }
    }

    Text::from_lines(lines)
}

/// Parse a single line of markdown
fn parse_line(line: &str, config: &MarkdownConfig) -> Vec<Line> {
    let trimmed = line.trim_start();

    // Check for headers
    if let Some(header_line) = parse_header(trimmed, config) {
        return wrap_line(header_line, config);
    }

    // Check for list items
    if let Some((bullet, rest)) = parse_list_item(trimmed) {
        let indent = line.len() - trimmed.len();
        let prefix = " ".repeat(indent);
        let mut spans = vec![Span::styled(
            format!("{}{} ", prefix, bullet),
            config.bullet_style,
        )];
        spans.extend(parse_inline(rest, config));
        return wrap_line(Line::from_spans(spans), config);
    }

    // Regular paragraph
    if trimmed.is_empty() {
        return vec![Line::empty()];
    }

    let spans = parse_inline(trimmed, config);
    wrap_line(Line::from_spans(spans), config)
}

/// Parse header syntax (# ## ###)
fn parse_header(line: &str, config: &MarkdownConfig) -> Option<Line> {
    if !line.starts_with('#') {
        return None;
    }

    let mut level = 0;
    for c in line.chars() {
        if c == '#' {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 6 {
        return None;
    }

    let content = line[level..].trim_start();
    if content.is_empty() {
        return None;
    }

    let style = match level {
        1 => config.h1_style,
        2 => config.h2_style,
        _ => config.h3_style,
    };

    let prefix = match level {
        1 => "# ",
        2 => "## ",
        _ => "### ",
    };

    Some(Line::from_spans([Span::styled(
        format!("{}{}", prefix, content),
        style,
    )]))
}

/// Parse list item (- or *)
fn parse_list_item(line: &str) -> Option<(char, &str)> {
    let mut chars = line.chars();
    let first = chars.next()?;

    if first == '-' || first == '*' {
        let second = chars.next()?;
        if second == ' ' {
            return Some((first, &line[2..]));
        }
    }

    // Numbered lists
    for (i, c) in line.char_indices() {
        if c.is_ascii_digit() {
            continue;
        }
        if c == '.' && i > 0 && line.get(i + 1..i + 2) == Some(" ") {
            return Some(('•', &line[i + 2..]));
        }
        break;
    }

    None
}

/// Parse inline markdown elements (bold, italic, code, links)
fn parse_inline(text: &str, config: &MarkdownConfig) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the next markdown element
        if let Some((before, span, after)) = find_next_element(remaining, config) {
            if !before.is_empty() {
                spans.push(Span::raw(before.to_string()));
            }
            spans.push(span);
            remaining = after;
        } else {
            // No more elements, add the rest as plain text
            spans.push(Span::raw(remaining.to_string()));
            break;
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(String::new()));
    }

    spans
}

/// Find the next markdown element in the text
fn find_next_element<'a>(
    text: &'a str,
    config: &MarkdownConfig,
) -> Option<(&'a str, Span, &'a str)> {
    let mut earliest: Option<(usize, &str, Span, &str)> = None;

    // Check for inline code (`)
    if let Some((start, content, end)) = find_delimited(text, "`", "`") {
        let span = Span::styled(format!("`{}`", content), config.code_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    // Check for bold (**text**)
    if let Some((start, content, end)) = find_delimited(text, "**", "**") {
        let span = Span::styled(content.to_string(), config.bold_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    // Check for bold (__text__)
    if let Some((start, content, end)) = find_delimited(text, "__", "__") {
        let span = Span::styled(content.to_string(), config.bold_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    // Check for italic (*text*) - but not **
    if let Some((start, content, end)) = find_single_delimited(text, '*') {
        let span = Span::styled(content.to_string(), config.italic_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    // Check for italic (_text_) - but not __
    if let Some((start, content, end)) = find_single_delimited(text, '_') {
        let span = Span::styled(content.to_string(), config.italic_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    // Check for links [text](url)
    if let Some((start, text_content, _url, end)) = find_link(text) {
        let span = Span::styled(text_content.to_string(), config.link_style);
        if earliest.is_none() || start < earliest.as_ref().unwrap().0 {
            earliest = Some((start, &text[..start], span, end));
        }
    }

    earliest.map(|(_, before, span, after)| (before, span, after))
}

/// Find text delimited by start and end markers
fn find_delimited<'a>(text: &'a str, start: &str, end: &str) -> Option<(usize, &'a str, &'a str)> {
    let start_pos = text.find(start)?;
    let content_start = start_pos + start.len();
    let remaining = &text[content_start..];
    let end_pos = remaining.find(end)?;

    if end_pos == 0 {
        return None; // Empty content
    }

    let content = &remaining[..end_pos];
    let after = &remaining[end_pos + end.len()..];

    Some((start_pos, content, after))
}

/// Find text delimited by a single character (for italic)
/// Avoids matching double delimiters (** or __)
fn find_single_delimited(text: &str, delim: char) -> Option<(usize, &str, &str)> {
    let double = format!("{}{}", delim, delim);

    let mut search_from = 0;
    loop {
        let start_pos = text[search_from..].find(delim)? + search_from;

        // Skip if this is a double delimiter
        if text[start_pos..].starts_with(&double) {
            search_from = start_pos + 2;
            continue;
        }

        // Also skip if preceded by the same char (we're in the middle of **)
        if start_pos > 0 && text[..start_pos].ends_with(delim) {
            search_from = start_pos + 1;
            continue;
        }

        let content_start = start_pos + 1;
        if content_start >= text.len() {
            return None;
        }

        let remaining = &text[content_start..];

        // Find closing delimiter
        let mut end_search = 0;
        loop {
            let end_pos = remaining[end_search..].find(delim)?;
            let abs_end = end_search + end_pos;

            // Skip if this is part of a double delimiter
            if remaining[abs_end..].starts_with(&double) {
                end_search = abs_end + 2;
                continue;
            }

            // Skip if preceded by the same char
            if abs_end > 0 && remaining[..abs_end].ends_with(delim) {
                end_search = abs_end + 1;
                continue;
            }

            if abs_end == 0 {
                return None; // Empty content
            }

            let content = &remaining[..abs_end];
            let after = &remaining[abs_end + 1..];
            return Some((start_pos, content, after));
        }
    }
}

/// Find a markdown link [text](url)
fn find_link(text: &str) -> Option<(usize, &str, &str, &str)> {
    let bracket_start = text.find('[')?;
    let remaining = &text[bracket_start + 1..];
    let bracket_end = remaining.find(']')?;
    let link_text = &remaining[..bracket_end];

    let after_bracket = &remaining[bracket_end + 1..];
    if !after_bracket.starts_with('(') {
        return None;
    }

    let paren_content = &after_bracket[1..];
    let paren_end = paren_content.find(')')?;
    let url = &paren_content[..paren_end];
    let after = &paren_content[paren_end + 1..];

    Some((bracket_start, link_text, url, after))
}

/// Wrap a line if it exceeds max_width
fn wrap_line(line: Line, config: &MarkdownConfig) -> Vec<Line> {
    if config.max_width == 0 || line.width() <= config.max_width {
        return vec![line];
    }

    let mut result = Vec::new();
    let mut current_spans: Vec<Span> = Vec::new();
    let mut current_width = 0;
    let prefix_width = UnicodeWidthStr::width(config.wrap_prefix.as_str());
    let effective_width = config.max_width;
    let mut is_first_line = true;

    for span in line.spans {
        let words: Vec<&str> = span.content.split_inclusive(' ').collect();

        for word in words {
            let word_width = UnicodeWidthStr::width(word);

            // Check if we need to wrap
            let line_limit = if is_first_line {
                effective_width
            } else {
                effective_width - prefix_width
            };

            if current_width + word_width > line_limit && current_width > 0 {
                // Wrap to new line
                result.push(Line::from_spans(current_spans.clone()));
                current_spans.clear();
                is_first_line = false;

                // Add prefix for continuation
                current_spans.push(Span::raw(config.wrap_prefix.clone()));
                current_width = prefix_width;
            }

            // Add word to current line
            if current_spans.is_empty() || current_spans.last().map(|s| s.style) != Some(span.style)
            {
                current_spans.push(Span::styled(word.to_string(), span.style));
            } else {
                // Extend the last span with same style
                if let Some(last) = current_spans.last_mut() {
                    last.content.push_str(word);
                }
            }
            current_width += word_width;
        }
    }

    if !current_spans.is_empty() {
        result.push(Line::from_spans(current_spans));
    }

    if result.is_empty() {
        result.push(Line::empty());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Modifier;

    #[test]
    fn test_plain_text() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Hello world", &config);
        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].spans[0].content, "Hello world");
    }

    #[test]
    fn test_bold() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Hello **world**", &config);
        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].spans.len(), 2);
        assert_eq!(text.lines[0].spans[0].content, "Hello ");
        assert_eq!(text.lines[0].spans[1].content, "world");
        assert!(text.lines[0].spans[1]
            .style
            .modifier
            .contains(Modifier::BOLD));
    }

    #[test]
    fn test_italic() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Hello *world*", &config);
        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].spans.len(), 2);
        assert_eq!(text.lines[0].spans[0].content, "Hello ");
        assert_eq!(text.lines[0].spans[1].content, "world");
        assert!(text.lines[0].spans[1]
            .style
            .modifier
            .contains(Modifier::ITALIC));
    }

    #[test]
    fn test_inline_code() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Use `code` here", &config);
        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].spans.len(), 3);
        assert_eq!(text.lines[0].spans[0].content, "Use ");
        assert_eq!(text.lines[0].spans[1].content, "`code`");
        assert_eq!(text.lines[0].spans[1].style.fg, Color::Yellow);
    }

    #[test]
    fn test_code_block() {
        let config = MarkdownConfig::default();
        let input = "```rust\nfn main() {}\n```";
        let text = parse_markdown(input, &config);
        assert_eq!(text.lines.len(), 2);
        assert!(text.lines[0].spans[0].content.contains("rust"));
        assert!(text.lines[1].spans[0].content.contains("fn main()"));
    }

    #[test]
    fn test_header() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("# Header", &config);
        assert_eq!(text.lines.len(), 1);
        assert!(text.lines[0].spans[0].content.contains("Header"));
        assert!(text.lines[0].spans[0]
            .style
            .modifier
            .contains(Modifier::BOLD));
    }

    #[test]
    fn test_list() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("- Item 1\n- Item 2", &config);
        assert_eq!(text.lines.len(), 2);
    }

    #[test]
    fn test_link() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Check [here](http://example.com)", &config);
        assert_eq!(text.lines.len(), 1);
        assert!(text.lines[0].spans.iter().any(|s| s.content == "here"));
    }

    #[test]
    fn test_multiline() {
        let config = MarkdownConfig::default();
        let text = parse_markdown("Line 1\nLine 2\nLine 3", &config);
        assert_eq!(text.lines.len(), 3);
    }

    #[test]
    fn test_wrap() {
        let config = MarkdownConfig::new().max_width(20);
        let text = parse_markdown("This is a longer line that should wrap", &config);
        assert!(text.lines.len() > 1);
    }
}
