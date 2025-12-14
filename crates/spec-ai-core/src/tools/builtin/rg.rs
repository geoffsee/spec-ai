use crate::tools::{Tool, ToolResult};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// Maximum output size in bytes to prevent context overflow
const MAX_OUTPUT_BYTES: usize = 100 * 1024; // 100 KiB

#[derive(Debug, Deserialize)]
struct RgArgs {
    /// Pattern to search for
    pattern: String,
    /// File or directory to search in
    path: Option<String>,
    /// Glob pattern to filter files (e.g., "*.rs")
    #[serde(default)]
    glob: Option<String>,
    /// File type to search (e.g., "rust", "py", "js")
    #[serde(rename = "type")]
    #[serde(default)]
    file_type: Option<String>,
    /// Case insensitive search
    #[serde(default)]
    case_insensitive: bool,
    /// Match whole words only
    #[serde(default)]
    word_regexp: bool,
    /// Treat pattern as literal string (not regex)
    #[serde(default)]
    fixed_strings: bool,
    /// Lines of context before and after match
    #[serde(default)]
    context: Option<usize>,
    /// Lines of context before match
    #[serde(default)]
    before_context: Option<usize>,
    /// Lines of context after match
    #[serde(default)]
    after_context: Option<usize>,
    /// Max matches per file
    #[serde(default)]
    max_count: Option<usize>,
    /// Search hidden files
    #[serde(default)]
    hidden: bool,
    /// Don't respect .gitignore
    #[serde(default)]
    no_ignore: bool,
    /// Multiline mode
    #[serde(default)]
    multiline: bool,
}

#[derive(Debug, Serialize)]
struct RgResponse {
    success: bool,
    output: String,
    truncated: bool,
    match_count: usize,
}

/// Tool that wraps the external `rg` (ripgrep) binary.
///
/// This tool provides access to ripgrep's powerful search capabilities
/// by shelling out to the `rg` command.
pub struct RgTool {
    root: PathBuf,
}

impl RgTool {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self { root }
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    fn resolve_path(&self, override_path: &Option<String>) -> PathBuf {
        override_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.root.clone())
    }

    fn build_command(&self, args: &RgArgs) -> Command {
        let mut cmd = Command::new("rg");

        // Always use these flags for consistent output
        cmd.arg("--line-number"); // Show line numbers
        cmd.arg("--with-filename"); // Always show filename
        cmd.arg("--color=never"); // No ANSI colors

        // Pattern matching options
        if args.case_insensitive {
            cmd.arg("-i");
        }
        if args.word_regexp {
            cmd.arg("-w");
        }
        if args.fixed_strings {
            cmd.arg("-F");
        }
        if args.multiline {
            cmd.arg("-U");
        }

        // Context options
        if let Some(ctx) = args.context {
            cmd.arg("-C").arg(ctx.to_string());
        } else {
            if let Some(before) = args.before_context {
                cmd.arg("-B").arg(before.to_string());
            }
            if let Some(after) = args.after_context {
                cmd.arg("-A").arg(after.to_string());
            }
        }

        // File filtering
        if let Some(ref glob) = args.glob {
            cmd.arg("-g").arg(glob);
        }
        if let Some(ref file_type) = args.file_type {
            cmd.arg("-t").arg(file_type);
        }

        // Max matches per file
        if let Some(max) = args.max_count {
            cmd.arg("-m").arg(max.to_string());
        }

        // Hidden and ignore options
        if args.hidden {
            cmd.arg("--hidden");
        }
        if args.no_ignore {
            cmd.arg("--no-ignore");
        }

        // The pattern
        cmd.arg(&args.pattern);

        // The search path
        let search_path = self.resolve_path(&args.path);
        cmd.arg(&search_path);

        cmd
    }
}

impl Default for RgTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RgTool {
    fn name(&self) -> &str {
        "rg"
    }

    fn description(&self) -> &str {
        "Search for patterns in files using ripgrep (rg). Requires the 'rg' binary to be installed on the system. Returns matching lines with file paths and line numbers."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Pattern to search for (regex by default, or literal if fixed_strings=true)"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in (defaults to current workspace)"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., '*.rs', '*.{js,ts}')"
                },
                "type": {
                    "type": "string",
                    "description": "File type to search (e.g., 'rust', 'py', 'js', 'ts'). Use 'rg --type-list' to see all types."
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case insensitive search (-i)",
                    "default": false
                },
                "word_regexp": {
                    "type": "boolean",
                    "description": "Match whole words only (-w)",
                    "default": false
                },
                "fixed_strings": {
                    "type": "boolean",
                    "description": "Treat pattern as literal string, not regex (-F)",
                    "default": false
                },
                "context": {
                    "type": "integer",
                    "description": "Lines of context before and after each match (-C)"
                },
                "before_context": {
                    "type": "integer",
                    "description": "Lines of context before each match (-B)"
                },
                "after_context": {
                    "type": "integer",
                    "description": "Lines of context after each match (-A)"
                },
                "max_count": {
                    "type": "integer",
                    "description": "Maximum matches per file (-m)"
                },
                "hidden": {
                    "type": "boolean",
                    "description": "Search hidden files and directories (--hidden)",
                    "default": false
                },
                "no_ignore": {
                    "type": "boolean",
                    "description": "Don't respect .gitignore and other ignore files (--no-ignore)",
                    "default": false
                },
                "multiline": {
                    "type": "boolean",
                    "description": "Enable multiline matching (-U)",
                    "default": false
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: RgArgs = serde_json::from_value(args).context("Failed to parse rg arguments")?;

        if args.pattern.trim().is_empty() {
            return Err(anyhow!("rg pattern cannot be empty"));
        }

        let search_path = self.resolve_path(&args.path);
        if !search_path.exists() {
            return Err(anyhow!(
                "Search path {} does not exist",
                search_path.display()
            ));
        }

        let mut cmd = self.build_command(&args);

        let output = cmd.output().context(
            "Failed to execute 'rg' command. Is ripgrep installed? Install with: brew install ripgrep (macOS), apt install ripgrep (Debian/Ubuntu), or cargo install ripgrep",
        )?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // rg exits with code 1 when no matches found (not an error)
        // rg exits with code 2 for actual errors
        if !output.status.success() && output.status.code() == Some(2) {
            return Err(anyhow!("rg error: {}", stderr));
        }

        let mut result_output = stdout.to_string();
        let mut truncated = false;

        // Count matches (lines that look like file:line:content)
        let match_count = result_output
            .lines()
            .filter(|line| {
                // Context lines start with file:line-, match lines start with file:line:
                line.contains(':') && !line.starts_with("--")
            })
            .count();

        // Truncate if too large
        if result_output.len() > MAX_OUTPUT_BYTES {
            result_output.truncate(MAX_OUTPUT_BYTES);
            // Try to truncate at a line boundary
            if let Some(last_newline) = result_output.rfind('\n') {
                result_output.truncate(last_newline);
            }
            result_output.push_str("\n... [output truncated]");
            truncated = true;
        }

        if result_output.is_empty() {
            result_output = "No matches found.".to_string();
        }

        let response = RgResponse {
            success: true,
            output: result_output,
            truncated,
            match_count,
        };

        Ok(ToolResult::success(
            serde_json::to_string(&response).context("Failed to serialize rg results")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_basic() {
        let tool = RgTool::new();
        let args = RgArgs {
            pattern: "test".to_string(),
            path: Some("/tmp".to_string()),
            glob: None,
            file_type: None,
            case_insensitive: false,
            word_regexp: false,
            fixed_strings: false,
            context: None,
            before_context: None,
            after_context: None,
            max_count: None,
            hidden: false,
            no_ignore: false,
            multiline: false,
        };

        let cmd = tool.build_command(&args);
        let program = cmd.get_program().to_string_lossy();
        assert_eq!(program, "rg");

        let args_vec: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
        assert!(args_vec.contains(&"--line-number".into()));
        assert!(args_vec.contains(&"--with-filename".into()));
        assert!(args_vec.contains(&"--color=never".into()));
        assert!(args_vec.contains(&"test".into()));
        assert!(args_vec.contains(&"/tmp".into()));
    }

    #[test]
    fn test_build_command_with_options() {
        let tool = RgTool::new();
        let args = RgArgs {
            pattern: "TODO".to_string(),
            path: None,
            glob: Some("*.rs".to_string()),
            file_type: Some("rust".to_string()),
            case_insensitive: true,
            word_regexp: true,
            fixed_strings: true,
            context: Some(3),
            before_context: None,
            after_context: None,
            max_count: Some(10),
            hidden: true,
            no_ignore: true,
            multiline: true,
        };

        let cmd = tool.build_command(&args);
        let args_vec: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();

        assert!(args_vec.contains(&"-i".into()));
        assert!(args_vec.contains(&"-w".into()));
        assert!(args_vec.contains(&"-F".into()));
        assert!(args_vec.contains(&"-U".into()));
        assert!(args_vec.contains(&"-C".into()));
        assert!(args_vec.contains(&"3".into()));
        assert!(args_vec.contains(&"-g".into()));
        assert!(args_vec.contains(&"*.rs".into()));
        assert!(args_vec.contains(&"-t".into()));
        assert!(args_vec.contains(&"rust".into()));
        assert!(args_vec.contains(&"-m".into()));
        assert!(args_vec.contains(&"10".into()));
        assert!(args_vec.contains(&"--hidden".into()));
        assert!(args_vec.contains(&"--no-ignore".into()));
    }

    #[test]
    fn test_build_command_before_after_context() {
        let tool = RgTool::new();
        let args = RgArgs {
            pattern: "test".to_string(),
            path: None,
            glob: None,
            file_type: None,
            case_insensitive: false,
            word_regexp: false,
            fixed_strings: false,
            context: None,
            before_context: Some(2),
            after_context: Some(5),
            max_count: None,
            hidden: false,
            no_ignore: false,
            multiline: false,
        };

        let cmd = tool.build_command(&args);
        let args_vec: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();

        assert!(args_vec.contains(&"-B".into()));
        assert!(args_vec.contains(&"2".into()));
        assert!(args_vec.contains(&"-A".into()));
        assert!(args_vec.contains(&"5".into()));
    }
}
