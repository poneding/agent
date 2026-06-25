use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::PluginProcessError;
use crate::process::{
    PLUGIN_PROCESS_PROTOCOL_VERSION, PluginProcessConfig, PluginProcessRequest,
    PluginProcessResponse, PluginProcessResponseKind, PluginProcessResult,
};

#[derive(Clone, Debug, Default)]
pub struct PluginProcessHost {
    _private: (),
}

impl PluginProcessHost {
    pub fn invoke_tool(
        &self,
        config: &PluginProcessConfig,
        request: &PluginProcessRequest,
    ) -> Result<serde_json::Value, PluginProcessError> {
        validate_config(config)?;
        validate_request(request)?;

        let mut request_json = serde_json::to_vec(request)
            .map_err(|error| PluginProcessError::Protocol(error.to_string()))?;
        request_json.push(b'\n');
        let mut child = Command::new(config.entrypoint.trim())
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| PluginProcessError::Spawn(error.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&request_json)
                .map_err(|error| PluginProcessError::Protocol(error.to_string()))?;
            drop(stdin);
        }

        let started_at = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|error| PluginProcessError::Spawn(error.to_string()))?;
                    return decode_output(config, request, output);
                }
                Ok(None) => {
                    if started_at.elapsed() >= Duration::from_millis(config.timeout_ms) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(PluginProcessError::Timeout {
                            timeout_ms: config.timeout_ms,
                        });
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => return Err(PluginProcessError::Spawn(error.to_string())),
            }
        }
    }
}

fn validate_config(config: &PluginProcessConfig) -> Result<(), PluginProcessError> {
    if config.entrypoint.trim().is_empty() {
        return Err(PluginProcessError::InvalidConfig(
            "entrypoint cannot be empty".to_string(),
        ));
    }

    if config.timeout_ms == 0 {
        return Err(PluginProcessError::InvalidConfig(
            "timeout_ms must be greater than zero".to_string(),
        ));
    }

    if config.max_stdout_bytes == 0 {
        return Err(PluginProcessError::InvalidConfig(
            "max_stdout_bytes must be greater than zero".to_string(),
        ));
    }

    Ok(())
}

fn validate_request(request: &PluginProcessRequest) -> Result<(), PluginProcessError> {
    if request.protocol_version != PLUGIN_PROCESS_PROTOCOL_VERSION {
        return Err(PluginProcessError::Protocol(format!(
            "unsupported request protocol version {}",
            request.protocol_version
        )));
    }

    if request.call_id.trim().is_empty() {
        return Err(PluginProcessError::Protocol(
            "call_id cannot be empty".to_string(),
        ));
    }

    Ok(())
}

fn decode_output(
    config: &PluginProcessConfig,
    request: &PluginProcessRequest,
    output: std::process::Output,
) -> Result<serde_json::Value, PluginProcessError> {
    if !output.status.success() {
        return Err(PluginProcessError::Exit {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    if output.stdout.len() > config.max_stdout_bytes {
        return Err(PluginProcessError::OutputTooLarge {
            limit: config.max_stdout_bytes,
            actual: output.stdout.len(),
        });
    }

    let response: PluginProcessResponse = serde_json::from_slice(&output.stdout)
        .map_err(|error| PluginProcessError::Protocol(error.to_string()))?;
    validate_response_envelope(request, &response)?;

    match response.result {
        PluginProcessResult::Success { ok: true, output } => Ok(output),
        PluginProcessResult::Success { ok: false, .. } => Err(PluginProcessError::Protocol(
            "successful response must set ok to true".to_string(),
        )),
        PluginProcessResult::Error {
            ok: false,
            error_code,
            message,
            recoverable,
        } => Err(PluginProcessError::Plugin {
            error_code,
            message,
            recoverable,
        }),
        PluginProcessResult::Error { ok: true, .. } => Err(PluginProcessError::Protocol(
            "error response must set ok to false".to_string(),
        )),
    }
}

fn validate_response_envelope(
    request: &PluginProcessRequest,
    response: &PluginProcessResponse,
) -> Result<(), PluginProcessError> {
    if response.protocol_version != PLUGIN_PROCESS_PROTOCOL_VERSION {
        return Err(PluginProcessError::Protocol(format!(
            "unsupported response protocol version {}",
            response.protocol_version
        )));
    }

    if response.kind != PluginProcessResponseKind::ToolResult {
        return Err(PluginProcessError::Protocol(
            "expected tool_result response".to_string(),
        ));
    }

    if response.call_id != request.call_id {
        return Err(PluginProcessError::Protocol(format!(
            "response call_id {} did not match request call_id {}",
            response.call_id, request.call_id
        )));
    }

    Ok(())
}
