use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub const SHELL_TOOL_NAME: &str = "execute_shell_command";

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ShellArgs {
    #[schemars(description = "The shell command to execute")]
    pub command: String,
    #[schemars(description = "Working directory (optional)")]
    pub cwd: Option<String>,
    #[schemars(description = "Timeout in seconds (default: 30)")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ShellExecute;

impl ShellExecute {
    pub fn new() -> Self {
        Self
    }

    /// Get the tool definition for the LLM
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": SHELL_TOOL_NAME,
                "description": "Execute a shell command and return the output. Use this for file operations, running scripts, checking system status, or any system task.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        },
                        "cwd": {
                            "type": "string",
                            "description": "Working directory (optional)"
                        },
                        "timeout": {
                            "type": "integer",
                            "description": "Timeout in seconds (default: 30)"
                        }
                    },
                    "required": ["command"]
                }
            }
        })
    }

    /// Execute a shell command
    pub async fn execute(&self, args: ShellArgs) -> Result<ShellResult, ShellToolError> {
        let timeout_duration = Duration::from_secs(args.timeout.unwrap_or(30));
        let cwd = args.cwd.unwrap_or_else(|| ".".to_string());

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(&args.command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let output = timeout(timeout_duration, cmd.output())
            .await
            .map_err(|_| ShellToolError::Timeout)?
            .map_err(|e| ShellToolError::Execution(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        let success = output.status.success();

        Ok(ShellResult {
            stdout,
            stderr,
            exit_code,
            success,
        })
    }

    /// Execute from JSON arguments
    pub async fn execute_json(&self, args_json: &str) -> Result<ShellResult, ShellToolError> {
        let args: ShellArgs = serde_json::from_str(args_json)
            .map_err(|e| ShellToolError::InvalidArgs(e.to_string()))?;
        self.execute(args).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShellToolError {
    #[error("Command execution failed: {0}")]
    Execution(String),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("Invalid working directory: {0}")]
    InvalidCwd(String),
    #[error("Command timed out")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
