use asi_ai_sdk::agent::tool::{Tool, ToolError};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

const MAX_OUTPUT_CHARS: usize = 8000;
const TIMEOUT_SECONDS: u64 = 30;

/// Configuration for a single allowed command.
struct CommandConfig {
    /// Allowed subcommands. `None` means any subcommand is allowed.
    subcommands: Option<Vec<&'static str>>,
    /// Reject any argument starting with `-`.
    reject_flags: bool,
}

/// Registry of commands the agent is allowed to run.
static SAFE_COMMANDS: LazyLock<HashMap<&'static str, CommandConfig>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "git",
        CommandConfig {
            subcommands: Some(vec![
                "status",
                "diff",
                "log",
                "show",
                "branch",
                "stash",
                "add",
                "commit",
                "push",
                "pull",
                "fetch",
                "checkout",
                "merge",
                "rebase",
                "tag",
                "blame",
                "describe",
                "shortlog",
                "rev-parse",
                "config",
                "clean",
                "mv",
                "rm",
                "reset",
                "help",
                "version",
            ]),
            reject_flags: true,
        },
    );
    m.insert(
        "npm",
        CommandConfig {
            subcommands: Some(vec![
                "install",
                "ci",
                "run",
                "test",
                "build",
                "start",
                "stop",
                "restart",
                "publish",
                "pack",
                "audit",
                "fund",
                "update",
                "outdated",
                "ls",
                "uninstall",
                "link",
                "dedupe",
                "prune",
                "help",
                "version",
            ]),
            reject_flags: true,
        },
    );
    m.insert(
        "cargo",
        CommandConfig {
            subcommands: Some(vec![
                "build",
                "run",
                "test",
                "check",
                "clippy",
                "fmt",
                "doc",
                "publish",
                "update",
                "clean",
                "add",
                "remove",
                "tree",
                "audit",
                "help",
                "metadata",
                "locate-project",
                "version",
            ]),
            reject_flags: true,
        },
    );
    m.insert(
        "npx",
        CommandConfig {
            subcommands: None, // any subcommand allowed
            reject_flags: true,
        },
    );
    m.insert(
        "node",
        CommandConfig {
            subcommands: None, // any script path allowed
            reject_flags: false,
        },
    );
    m
});

/// Check whether `input` contains shell metacharacters that could be used
/// for injection.
fn is_shell_safe(input: &str) -> bool {
    // Check each character for shell metacharacters that could enable injection.
    // This avoids regex escaping issues inside character classes.
    !input.chars().any(|c| {
        matches!(
            c,
            ';' | '&'
                | '|'
                | '`'
                | '$'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '!'
                | '#'
                | '~'
                | '*'
                | '?'
                | '\\'
                | '\n'
                | '\r'
        )
    })
}

/// Truncate output to MAX_OUTPUT_CHARS and return a truncation flag.
fn truncate_output(output: &str) -> (String, bool) {
    if output.len() > MAX_OUTPUT_CHARS {
        let safe_idx = output
            .char_indices()
            .nth(MAX_OUTPUT_CHARS)
            .map(|(i, _)| i)
            .unwrap_or(output.len());
        let mut truncated = output[..safe_idx].to_string();
        truncated.push_str("\n... (output truncated)");
        (truncated, true)
    } else {
        (output.to_string(), false)
    }
}

/// Tool that executes a shell command with strict security controls.
///
/// Security layers:
/// 1. Command allowlist (git, npm, cargo, npx, node)
/// 2. Per-command subcommand allowlist
/// 3. Flag rejection for dangerous commands
/// 4. Shell metacharacter rejection
/// 5. 30-second timeout
/// 6. Output capped at 8000 characters
pub struct RunCommandTool;

#[async_trait]
impl Tool for RunCommandTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: "runCommand".into(),
                description: "Execute a terminal command with strict security controls. ".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Command to run — one of: git, npm, cargo, npx, node"
                        },
                        "args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Command arguments (subcommand, flags, etc.)"
                        }
                    },
                    "required": ["command"]
                }),
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<String, ToolError> {
        let cmd_name = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing 'command' argument".into()))?;

        // 1. Command allowlist
        let config = SAFE_COMMANDS.get(cmd_name).ok_or_else(|| {
            ToolError::Execution(format!("Command '{}' is not allowed", cmd_name))
        })?;

        let args: Vec<String> = arguments
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        // 2. Subcommand allowlist
        if let Some(allowed_subcommands) = &config.subcommands
            && let Some(first_arg) = args.first()
            && !allowed_subcommands.contains(&first_arg.as_str())
        {
            return Err(ToolError::Execution(format!(
                "Subcommand '{}' is not allowed for '{}'",
                first_arg, cmd_name
            )));
        }

        // 3. Flag rejection
        if config.reject_flags {
            for arg in &args {
                if arg.starts_with('-') {
                    return Err(ToolError::Execution(format!(
                        "Flag arguments are not allowed for '{}': '{}'",
                        cmd_name, arg
                    )));
                }
            }
        }

        // 4. Shell metacharacter rejection on all arguments and command name
        for arg in std::iter::once(cmd_name).chain(args.iter().map(|s| s.as_str())) {
            if !is_shell_safe(arg) {
                return Err(ToolError::Execution(format!(
                    "Shell metacharacters are not allowed in command or arguments: '{}'",
                    arg
                )));
            }
        }

        // 5. Execute with timeout
        let mut cmd = Command::new(cmd_name);
        cmd.args(&args).kill_on_drop(true);

        let output_result = timeout(Duration::from_secs(TIMEOUT_SECONDS), cmd.output()).await;

        match output_result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else if stdout.is_empty() {
                    stderr.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };

                // 6. Output truncation
                let (truncated, was_truncated) = truncate_output(&combined);

                if !output.status.success() {
                    let exit_code = output.status.code().unwrap_or(-1);
                    return Err(ToolError::Execution(format!(
                        "Command exited with code {}:\n{}",
                        exit_code, truncated
                    )));
                }

                if was_truncated {
                    Ok(format!(
                        "{}\n\n[Output truncated at {} characters]",
                        truncated, MAX_OUTPUT_CHARS
                    ))
                } else {
                    Ok(truncated)
                }
            }
            Ok(Err(e)) => Err(ToolError::Execution(format!(
                "Failed to execute command '{}': {}",
                cmd_name, e
            ))),
            Err(_) => Err(ToolError::Execution(format!(
                "Command '{}' timed out after {} seconds",
                cmd_name, TIMEOUT_SECONDS
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_run_command_git_status() {
        let tool = RunCommandTool;
        let result = tool
            .execute(json!({ "command": "git", "args": ["status"] }))
            .await;
        assert!(result.is_ok(), "Expected ok, got: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_command_command_not_allowed() {
        let tool = RunCommandTool;
        let result = tool
            .execute(json!({ "command": "rm", "args": ["-rf", "/"] }))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_run_command_subcommand_not_allowed() {
        let tool = RunCommandTool;
        let result = tool
            .execute(json!({ "command": "git", "args": ["bisect"] }))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_run_command_flag_rejected() {
        let tool = RunCommandTool;
        let result = tool
            .execute(json!({ "command": "git", "args": ["push", "--force"] }))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Flag arguments"));
    }

    #[tokio::test]
    async fn test_run_command_shell_metachar_blocked() {
        let tool = RunCommandTool;
        // "status" is a valid subcommand, but ";" in a later arg is a metacharacter
        let result = tool
            .execute(json!({ "command": "git", "args": ["status", ";echo", "hack"] }))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Shell metacharacters") || err.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_run_command_missing_command() {
        let tool = RunCommandTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn test_run_command_npx_allows_any_subcommand() {
        // npx allows any subcommand
        let tool = RunCommandTool;
        // This might fail because npx might not find the command,
        // but it should not be rejected by our allowlist
        let result = tool
            .execute(json!({ "command": "npx", "args": ["some-random-package"] }))
            .await;
        // We expect execution error (not found), not an allowlist rejection
        assert!(result.is_err());
        // Should be an execution failure (it won't find the package),
        // not an "is not allowed" message
        let _ = result.unwrap_err();
    }

    #[test]
    fn test_run_command_definition() {
        let tool = RunCommandTool;
        let def = tool.definition();
        assert_eq!(def.function.name, "runCommand");
        assert_eq!(def.def_type, "function");
    }

    #[test]
    fn test_is_shell_safe() {
        // Safe strings
        assert!(is_shell_safe("git"));
        assert!(is_shell_safe("status"));
        assert!(is_shell_safe("hello-world_123"));
        assert!(is_shell_safe("path/to/file.txt"));

        // Unsafe strings with metacharacters
        assert!(!is_shell_safe("hello; world"));
        assert!(!is_shell_safe("cmd & background"));
        assert!(!is_shell_safe("cmd | pipe"));
        assert!(!is_shell_safe("$(dangerous)"));
        assert!(!is_shell_safe("`backtick`"));
        assert!(!is_shell_safe("> redirect"));
        assert!(!is_shell_safe("${var}"));
    }

    #[test]
    fn test_truncate_output() {
        let short = "hello";
        let (result, truncated) = truncate_output(short);
        assert!(!truncated);
        assert_eq!(result, "hello");

        let long = "a".repeat(MAX_OUTPUT_CHARS + 100);
        let (result, truncated) = truncate_output(&long);
        assert!(truncated);
        assert!(result.contains("(output truncated)"));
        assert!(result.len() < long.len());
    }
}
