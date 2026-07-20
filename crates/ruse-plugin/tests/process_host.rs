#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ruse_plugin::{
    PLUGIN_PROCESS_PROTOCOL_VERSION, PluginProcessConfig, PluginProcessError, PluginProcessHost,
    PluginProcessRequest, PluginProcessRequestKind,
};
use ruse_tool::ToolName;
use serde_json::json;

#[test]
fn process_host_invokes_fixture_tool() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "success",
        r#"IFS= read -r _request
printf '%s\n' '{"protocol_version":1,"type":"tool_result","call_id":"call-1","ok":true,"output":{"content":"fixture ok"}}'"#,
    );

    let output = PluginProcessHost::default()
        .invoke_tool(&config(entrypoint), &request("call-1"))
        .expect("invoke fixture");

    assert_eq!(output, json!({ "content": "fixture ok" }));
}

#[test]
fn process_host_maps_plugin_error_response() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "plugin-error",
        r#"IFS= read -r _request
printf '%s\n' '{"protocol_version":1,"type":"tool_result","call_id":"call-1","ok":false,"error_code":"plugin_error","message":"boom","recoverable":true}'"#,
    );

    let err = PluginProcessHost::default()
        .invoke_tool(&config(entrypoint), &request("call-1"))
        .unwrap_err();

    assert!(
        matches!(
            &err,
            PluginProcessError::Plugin {
                error_code,
                message,
                recoverable: true
            } if error_code == "plugin_error" && message == "boom"
        ),
        "unexpected plugin error response: {err:?}"
    );
}

#[test]
fn process_host_rejects_invalid_json_response() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "invalid-json",
        r#"IFS= read -r _request
printf '%s\n' 'not json'"#,
    );

    let err = PluginProcessHost::default()
        .invoke_tool(&config(entrypoint), &request("call-1"))
        .unwrap_err();

    assert!(
        matches!(&err, PluginProcessError::Protocol(_)),
        "unexpected invalid json response error: {err:?}"
    );
}

#[test]
fn process_host_rejects_mismatched_call_id() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "mismatched-call",
        r#"IFS= read -r _request
printf '%s\n' '{"protocol_version":1,"type":"tool_result","call_id":"other-call","ok":true,"output":{"content":"fixture ok"}}'"#,
    );

    let err = PluginProcessHost::default()
        .invoke_tool(&config(entrypoint), &request("call-1"))
        .unwrap_err();

    assert!(
        matches!(&err, PluginProcessError::Protocol(_)),
        "unexpected mismatched call id error: {err:?}"
    );
}

#[test]
fn process_host_rejects_missing_entrypoint() {
    let _guard = process_fixture_guard();
    let err = PluginProcessHost::default()
        .invoke_tool(
            &PluginProcessConfig {
                entrypoint: " ".to_string(),
                timeout_ms: DEFAULT_PROCESS_TEST_TIMEOUT_MS,
                max_stdout_bytes: 1024,
            },
            &request("call-1"),
        )
        .unwrap_err();

    assert!(matches!(err, PluginProcessError::InvalidConfig(_)));
}

#[test]
fn process_host_rejects_oversized_stdout() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "oversized",
        r#"IFS= read -r _request
printf '%s\n' '{"protocol_version":1,"type":"tool_result","call_id":"call-1","ok":true,"output":{"content":"fixture ok"}}'"#,
    );

    let err = PluginProcessHost::default()
        .invoke_tool(
            &PluginProcessConfig {
                entrypoint,
                timeout_ms: DEFAULT_PROCESS_TEST_TIMEOUT_MS,
                max_stdout_bytes: 8,
            },
            &request("call-1"),
        )
        .unwrap_err();

    assert!(
        matches!(&err, PluginProcessError::OutputTooLarge { .. }),
        "unexpected oversized stdout error: {err:?}"
    );
}

#[test]
fn process_host_times_out_slow_process() {
    let _guard = process_fixture_guard();
    let entrypoint = fixture_script(
        "timeout",
        r#"IFS= read -r _request
while :; do :; done
printf '%s\n' '{"protocol_version":1,"type":"tool_result","call_id":"call-1","ok":true,"output":{"content":"late"}}'"#,
    );

    let err = PluginProcessHost::default()
        .invoke_tool(
            &PluginProcessConfig {
                entrypoint,
                timeout_ms: 10,
                max_stdout_bytes: 1024,
            },
            &request("call-1"),
        )
        .unwrap_err();

    assert!(matches!(
        err,
        PluginProcessError::Timeout { timeout_ms: 10 }
    ));
}

fn process_fixture_guard() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

const DEFAULT_PROCESS_TEST_TIMEOUT_MS: u64 = 10_000;

fn request(call_id: &str) -> PluginProcessRequest {
    PluginProcessRequest {
        protocol_version: PLUGIN_PROCESS_PROTOCOL_VERSION,
        kind: PluginProcessRequestKind::ToolCall,
        call_id: call_id.to_string(),
        tool_name: ToolName::new("example.read").expect("valid tool name"),
        input: json!({ "path": "README.md" }),
    }
}

fn config(entrypoint: String) -> PluginProcessConfig {
    PluginProcessConfig {
        entrypoint,
        timeout_ms: DEFAULT_PROCESS_TEST_TIMEOUT_MS,
        max_stdout_bytes: 1024,
    }
}

fn fixture_script(name: &str, body: &str) -> String {
    let path = fixture_path(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write fixture script");

    let mut permissions = fs::metadata(&path).expect("fixture metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("make fixture executable");

    path.to_string_lossy().into_owned()
}

fn fixture_path(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "ruse-plugin-process-host-{}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create fixture directory");
    directory.join("fixture.sh")
}
