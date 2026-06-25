use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use agent_sandbox::WorkspaceSandbox;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const POLL_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Clone, Debug)]
pub struct ShellCommandTool {
    sandbox: WorkspaceSandbox,
    allowed_env: BTreeSet<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShellCommandResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug)]
struct ShellCommandInput {
    program: String,
    args: Vec<String>,
    cwd: String,
    timeout: Duration,
    env: BTreeMap<String, String>,
}

impl ShellCommandTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self {
            sandbox,
            allowed_env: BTreeSet::new(),
        }
    }

    pub fn with_allowed_env<const N: usize>(
        sandbox: WorkspaceSandbox,
        allowed_env: [&str; N],
    ) -> Self {
        Self {
            sandbox,
            allowed_env: allowed_env
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }
}

impl Tool for ShellCommandTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.shell_command").expect("static tool name is valid"),
            description: "Execute a controlled workspace-confined command".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["program"],
                "properties": {
                    "program": { "type": "string" },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "cwd": { "type": "string" },
                    "timeout_ms": {
                        "type": "integer",
                        "minimum": 0
                    },
                    "env": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let input = parse_input(input)?;
        let cwd = self
            .sandbox
            .resolve_read_path(&input.cwd)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        if !cwd.is_dir() {
            return Err(ToolError::Validation(format!(
                "cwd is not a directory: {}",
                input.cwd
            )));
        }
        self.validate_env(&input.env)?;

        let output = execute_command(input, &cwd)?;
        Ok(ToolOutput {
            value: serde_json::to_value(output)
                .map_err(|error| ToolError::Execution(error.to_string()))?,
        })
    }
}

impl ShellCommandTool {
    fn validate_env(&self, env: &BTreeMap<String, String>) -> Result<(), ToolError> {
        for key in env.keys() {
            if key.is_empty() {
                return Err(ToolError::Validation(
                    "environment key cannot be empty".to_string(),
                ));
            }
            if !self.allowed_env.contains(key) {
                return Err(ToolError::Validation(format!(
                    "environment key is not allowlisted: {key}"
                )));
            }
        }
        Ok(())
    }
}

fn parse_input(input: ToolInput) -> Result<ShellCommandInput, ToolError> {
    let program = input
        .value
        .get("program")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ToolError::Validation("program must be a non-empty string".to_string()))?
        .to_string();
    let args = parse_args(input.value.get("args"))?;
    let cwd = input
        .value
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".")
        .to_string();
    let timeout_ms = input
        .value
        .get("timeout_ms")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(DEFAULT_TIMEOUT_MS);
    let env = parse_env(input.value.get("env"))?;

    Ok(ShellCommandInput {
        program,
        args,
        cwd,
        timeout: Duration::from_millis(timeout_ms),
        env,
    })
}

fn parse_args(value: Option<&serde_json::Value>) -> Result<Vec<String>, ToolError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let args = value
        .as_array()
        .ok_or_else(|| ToolError::Validation("args must be an array".to_string()))?;
    args.iter()
        .map(|value| {
            value
                .as_str()
                .map(std::string::ToString::to_string)
                .ok_or_else(|| ToolError::Validation("args must contain only strings".to_string()))
        })
        .collect()
}

fn parse_env(value: Option<&serde_json::Value>) -> Result<BTreeMap<String, String>, ToolError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let env = value
        .as_object()
        .ok_or_else(|| ToolError::Validation("env must be an object".to_string()))?;
    env.iter()
        .map(|(key, value)| {
            let value = value.as_str().ok_or_else(|| {
                ToolError::Validation("env values must contain only strings".to_string())
            })?;
            Ok((key.clone(), value.to_string()))
        })
        .collect()
}

fn execute_command(input: ShellCommandInput, cwd: &Path) -> Result<ShellCommandResult, ToolError> {
    let program = resolve_program(&input.program);
    let mut command = Command::new(program);
    command
        .args(&input.args)
        .current_dir(cwd)
        .env_clear()
        .envs(&input.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| ToolError::Execution(error.to_string()))?;
    let started = Instant::now();

    loop {
        if child
            .try_wait()
            .map_err(|error| ToolError::Execution(error.to_string()))?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| ToolError::Execution(error.to_string()))?;
            return Ok(ShellCommandResult {
                status: status_code(output.status),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                timed_out: false,
            });
        }

        if started.elapsed() >= input.timeout {
            kill_child(&mut child)?;
            let output = child
                .wait_with_output()
                .map_err(|error| ToolError::Execution(error.to_string()))?;
            return Ok(ShellCommandResult {
                status: -1,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                timed_out: true,
            });
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn kill_child(child: &mut std::process::Child) -> Result<(), ToolError> {
    match child.kill() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
        Err(error) => Err(ToolError::Execution(error.to_string())),
    }
}

fn status_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(-1)
}

fn resolve_program(program: &str) -> PathBuf {
    if has_path_separator(program) {
        return PathBuf::from(program);
    }

    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .flat_map(|path| executable_candidates(&path, program))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(program))
}

fn executable_candidates(path: &Path, program: &str) -> Vec<PathBuf> {
    let candidate = path.join(program);
    #[cfg(windows)]
    {
        let mut candidates = vec![candidate.clone()];
        if candidate.extension().is_none() {
            candidates.push(path.join(format!("{program}.exe")));
        }
        candidates
    }
    #[cfg(not(windows))]
    {
        vec![candidate]
    }
}

fn has_path_separator(program: &str) -> bool {
    program.contains('/') || program.contains('\\')
}
