use crate::tools::{Tool, ToolResult};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Default maximum number of matching lines to return
const DEFAULT_MAX_MATCHES: usize = 50;
/// Hard maximum to prevent context overload
const HARD_MAX_MATCHES: usize = 200;
/// Default context lines before/after match
const DEFAULT_CONTEXT_LINES: usize = 0;
/// Maximum file size to read (1 MiB)
const DEFAULT_MAX_FILE_BYTES: usize = 1024 * 1024;
/// Maximum line length to include (truncate longer lines)
const MAX_LINE_LENGTH: usize = 500;

#[derive(Debug, Deserialize)]
struct GrepArgs {
    /// Pattern to search for (regex or literal)
    pattern: String,
    /// Root directory or file to search in
    path: Option<String>,
    /// Glob pattern to filter files (e.g., "*.rs", "**/*.py")
    #[serde(default)]
    glob: Option<String>,
    /// Interpret pattern as regex (default: true)
    #[serde(default = "default_true")]
    regex: bool,
    /// Case insensitive search
    #[serde(default)]
    case_insensitive: bool,
    /// Lines of context before match
    #[serde(rename = "before_context")]
    before: Option<usize>,
    /// Lines of context after match
    #[serde(rename = "after_context")]
    after: Option<usize>,
    /// Lines of context before and after (overrides before/after if set)
    context: Option<usize>,
    /// Maximum number of matches to return
    max_matches: Option<usize>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
struct GrepMatch {
    /// File path containing the match
    file: String,
    /// Line number (1-indexed)
    line_number: usize,
    /// The matching line content
    content: String,
    /// Context lines before the match (if requested)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    before_context: Vec<ContextLine>,
    /// Context lines after the match (if requested)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    after_context: Vec<ContextLine>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContextLine {
    line_number: usize,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GrepResponse {
    pattern: String,
    total_matches: usize,
    matches: Vec<GrepMatch>,
    truncated: bool,
}

/// Tool that uses grep-like pattern matching to read specific parts of files.
///
/// This tool is designed to help avoid context overload by returning only
/// the relevant portions of files that match a given pattern, with optional
/// surrounding context lines.
pub struct GrepTool {
    root: PathBuf,
    max_file_bytes: usize,
}

impl GrepTool {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            root,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    pub fn with_max_file_bytes(mut self, max_file_bytes: usize) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }

    fn resolve_path(&self, override_path: &Option<String>) -> PathBuf {
        override_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.root.clone())
    }

    fn matches_glob(&self, path: &Path, glob_regex: &Option<Regex>) -> bool {
        match glob_regex {
            None => true,
            Some(regex) => {
                // Try matching against the full path and just the filename
                let path_str = path.to_string_lossy();
                let filename = path.file_name().map(|s| s.to_string_lossy());

                regex.is_match(&path_str) || filename.map(|f| regex.is_match(&f)).unwrap_or(false)
            }
        }
    }

    /// Convert a glob pattern to a regex pattern
    fn glob_to_regex(glob: &str) -> Result<Regex> {
        let mut regex = String::with_capacity(glob.len() * 2);
        regex.push('^');

        let mut chars = glob.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                                      // Skip path separator if present after **
                        if chars.peek() == Some(&'/') {
                            chars.next();
                        }
                        // ** matches any path including separators
                        regex.push_str(".*");
                    } else {
                        // * matches anything except path separator
                        regex.push_str("[^/]*");
                    }
                }
                '?' => regex.push_str("[^/]"),
                '.' | '+' | '^' | '$' | '(' | ')' | '{' | '}' | '|' | '\\' => {
                    regex.push('\\');
                    regex.push(c);
                }
                '[' => {
                    // Character class - pass through but escape special regex chars inside
                    regex.push('[');
                    while let Some(c) = chars.next() {
                        if c == ']' {
                            regex.push(']');
                            break;
                        }
                        regex.push(c);
                    }
                }
                _ => regex.push(c),
            }
        }

        regex.push('$');
        Regex::new(&regex).context("Failed to compile glob pattern as regex")
    }

    fn truncate_line(line: &str) -> String {
        if line.len() > MAX_LINE_LENGTH {
            format!("{}...", &line[..MAX_LINE_LENGTH])
        } else {
            line.to_string()
        }
    }

    fn collect_matches(
        &self,
        path: &Path,
        regex: &regex::Regex,
        args: &GrepArgs,
        max_matches: usize,
        current_count: &mut usize,
    ) -> Result<Vec<GrepMatch>> {
        let metadata = fs::metadata(path).context("Failed to read file metadata")?;

        if metadata.len() as usize > self.max_file_bytes {
            return Ok(Vec::new());
        }

        let data = fs::read(path).context("Failed to read file")?;
        let content = match String::from_utf8(data) {
            Ok(text) => text,
            Err(_) => return Ok(Vec::new()), // Skip binary files
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        // Determine context sizes
        let (before_ctx, after_ctx) = match args.context {
            Some(c) => (c, c),
            None => (
                args.before.unwrap_or(DEFAULT_CONTEXT_LINES),
                args.after.unwrap_or(DEFAULT_CONTEXT_LINES),
            ),
        };

        for (idx, line) in lines.iter().enumerate() {
            if *current_count >= max_matches {
                break;
            }

            if regex.is_match(line) {
                let line_number = idx + 1;

                // Collect before context
                let before_context: Vec<ContextLine> = if before_ctx > 0 {
                    let start = idx.saturating_sub(before_ctx);
                    (start..idx)
                        .map(|i| ContextLine {
                            line_number: i + 1,
                            content: Self::truncate_line(lines[i]),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                // Collect after context
                let after_context: Vec<ContextLine> = if after_ctx > 0 {
                    let end = (idx + 1 + after_ctx).min(lines.len());
                    ((idx + 1)..end)
                        .map(|i| ContextLine {
                            line_number: i + 1,
                            content: Self::truncate_line(lines[i]),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                matches.push(GrepMatch {
                    file: path.display().to_string(),
                    line_number,
                    content: Self::truncate_line(line),
                    before_context,
                    after_context,
                });

                *current_count += 1;
            }
        }

        Ok(matches)
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for patterns in files using grep-like matching. Returns matching lines with optional context to avoid loading entire files into context."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Pattern to search for (regex by default, or literal if regex=false)"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in (defaults to current workspace)"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., '*.rs', '**/*.py', 'src/**/*.ts')"
                },
                "regex": {
                    "type": "boolean",
                    "description": "Interpret pattern as regex (default: true)",
                    "default": true
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case insensitive search (default: false)",
                    "default": false
                },
                "before_context": {
                    "type": "integer",
                    "description": "Number of lines to show before each match (like grep -B)"
                },
                "after_context": {
                    "type": "integer",
                    "description": "Number of lines to show after each match (like grep -A)"
                },
                "context": {
                    "type": "integer",
                    "description": "Number of lines to show before and after each match (like grep -C, overrides before/after)"
                },
                "max_matches": {
                    "type": "integer",
                    "description": "Maximum number of matches to return (default: 50, max: 200)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: GrepArgs =
            serde_json::from_value(args).context("Failed to parse grep arguments")?;

        if args.pattern.trim().is_empty() {
            return Err(anyhow!("grep pattern cannot be empty"));
        }

        let search_path = self.resolve_path(&args.path);
        if !search_path.exists() {
            return Err(anyhow!(
                "Search path {} does not exist",
                search_path.display()
            ));
        }

        let max_matches = args
            .max_matches
            .unwrap_or(DEFAULT_MAX_MATCHES)
            .clamp(1, HARD_MAX_MATCHES);

        // Build the regex pattern
        let regex = if args.regex {
            RegexBuilder::new(&args.pattern)
                .case_insensitive(args.case_insensitive)
                .build()
                .context("Invalid regular expression pattern")?
        } else {
            // Escape the pattern for literal matching
            let escaped = regex::escape(&args.pattern);
            RegexBuilder::new(&escaped)
                .case_insensitive(args.case_insensitive)
                .build()
                .context("Failed to build literal pattern")?
        };

        // Parse glob pattern if provided
        let glob_regex = args
            .glob
            .as_ref()
            .map(|g| Self::glob_to_regex(g))
            .transpose()?;

        let mut all_matches = Vec::new();
        let mut match_count = 0;

        if search_path.is_file() {
            // Search single file
            if self.matches_glob(&search_path, &glob_regex) {
                let file_matches = self.collect_matches(
                    &search_path,
                    &regex,
                    &args,
                    max_matches,
                    &mut match_count,
                )?;
                all_matches.extend(file_matches);
            }
        } else {
            // Walk directory
            for entry in WalkDir::new(&search_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if match_count >= max_matches {
                    break;
                }

                let path = entry.path();
                if !entry.file_type().is_file() {
                    continue;
                }

                // Skip hidden files and common non-text directories
                let path_str = path.to_string_lossy();
                if path_str.contains("/.git/")
                    || path_str.contains("/node_modules/")
                    || path_str.contains("/target/")
                    || path_str.contains("/.venv/")
                    || path_str.contains("/__pycache__/")
                {
                    continue;
                }

                if !self.matches_glob(path, &glob_regex) {
                    continue;
                }

                match self.collect_matches(path, &regex, &args, max_matches, &mut match_count) {
                    Ok(file_matches) => all_matches.extend(file_matches),
                    Err(_) => continue, // Skip files we can't read
                }
            }
        }

        let truncated = match_count >= max_matches;
        let response = GrepResponse {
            pattern: args.pattern,
            total_matches: all_matches.len(),
            matches: all_matches,
            truncated,
        };

        Ok(ToolResult::success(
            serde_json::to_string(&response).context("Failed to serialize grep results")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_basic_grep() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() {\n    println!(\"Hello\");\n}\n\nfn other() {\n    println!(\"World\");\n}",
        )
        .unwrap();

        let tool = GrepTool::new().with_root(dir.path());
        let args = serde_json::json!({
            "pattern": "fn \\w+",
            "path": dir.path().to_string_lossy()
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let payload: GrepResponse = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload.total_matches, 2);
    }

    #[tokio::test]
    async fn test_grep_with_context() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "line 1\nline 2\nMATCH\nline 4\nline 5").unwrap();

        let tool = GrepTool::new().with_root(dir.path());
        let args = serde_json::json!({
            "pattern": "MATCH",
            "path": file_path.to_string_lossy(),
            "context": 1,
            "regex": false
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let payload: GrepResponse = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload.total_matches, 1);
        assert_eq!(payload.matches[0].before_context.len(), 1);
        assert_eq!(payload.matches[0].after_context.len(), 1);
        assert_eq!(payload.matches[0].before_context[0].content, "line 2");
        assert_eq!(payload.matches[0].after_context[0].content, "line 4");
    }

    #[tokio::test]
    async fn test_grep_case_insensitive() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello World\nhello world\nHELLO WORLD").unwrap();

        let tool = GrepTool::new().with_root(dir.path());
        let args = serde_json::json!({
            "pattern": "hello",
            "path": file_path.to_string_lossy(),
            "case_insensitive": true,
            "regex": false
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let payload: GrepResponse = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload.total_matches, 3);
    }

    #[tokio::test]
    async fn test_grep_with_glob() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main()").unwrap();
        fs::write(dir.path().join("test.py"), "def main()").unwrap();
        fs::write(dir.path().join("test.txt"), "main function").unwrap();

        let tool = GrepTool::new().with_root(dir.path());
        let args = serde_json::json!({
            "pattern": "main",
            "path": dir.path().to_string_lossy(),
            "glob": "*.rs",
            "regex": false
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let payload: GrepResponse = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload.total_matches, 1);
        assert!(payload.matches[0].file.ends_with("test.rs"));
    }

    #[tokio::test]
    async fn test_grep_max_matches() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = (0..100)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, content).unwrap();

        let tool = GrepTool::new().with_root(dir.path());
        let args = serde_json::json!({
            "pattern": "line",
            "path": file_path.to_string_lossy(),
            "max_matches": 5,
            "regex": false
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let payload: GrepResponse = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload.total_matches, 5);
        assert!(payload.truncated);
    }
}
