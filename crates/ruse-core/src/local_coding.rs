use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_storage::{EventStore, InMemoryEventStore};
use ruse_tool::ToolRegistry;
use ruse_tool::builtins::{
    FileEditTool, GitDiffTool, GitStatusTool, ReadFileTool, ShellCommandTool,
};
use ruse_types::{
    ActorRef, ConversationId, CorrelationId, FinalResponseProduced, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, TurnId, WorkspaceId,
};
use serde_json::json;
use thiserror::Error;

use crate::{RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopOutcome, ToolLoopStatus};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCodingOutcome {
    pub final_answer: String,
    pub changed_files: Vec<String>,
    pub command_results: Vec<String>,
    pub audit_event_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCodingRun {
    pub outcome: LocalCodingOutcome,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Error)]
pub enum LocalCodingError {
    #[error("fixture setup failed: {0}")]
    Fixture(String),
    #[error("workspace setup failed: {0}")]
    Sandbox(String),
    #[error("tool registration failed: {0}")]
    ToolRegistration(String),
    #[error("tool workflow failed: {0}")]
    ToolWorkflow(String),
    #[error("storage failed: {0}")]
    Storage(String),
    #[error("replay failed: {0}")]
    Replay(String),
}

pub fn run_local_coding_fixture() -> Result<LocalCodingRun, LocalCodingError> {
    run_local_coding_fixture_with_shell_policy(PolicyDecision::Allow {
        reason: "fixture allows shell verification".to_string(),
    })
}

pub fn run_local_coding_fixture_with_shell_denied() -> Result<LocalCodingRun, LocalCodingError> {
    run_local_coding_fixture_with_shell_policy(PolicyDecision::Deny {
        reason: "denied by fixture policy".to_string(),
    })
}

pub fn replay_local_coding_outcome(
    events: &[RuntimeEvent],
) -> Result<LocalCodingOutcome, LocalCodingError> {
    let mut tool_by_call_id = BTreeMap::new();
    let mut last_call_id = None;
    let mut changed_files = Vec::new();
    let mut command_results = Vec::new();
    let mut final_answer = None;
    let audit_event_count = count_audit_events(events);

    for event in events {
        match &event.payload {
            RuntimeEventPayload::ToolCallProposed(call) => {
                tool_by_call_id.insert(call.call_id.clone(), call.tool_name.clone());
                last_call_id = Some(call.call_id.clone());
            }
            RuntimeEventPayload::PermissionDecisionRecorded(decision) => {
                if decision.decision == "deny"
                    && let Some(call_id) = last_call_id.as_ref()
                    && tool_by_call_id
                        .get(call_id)
                        .is_some_and(|tool| tool == "workspace.shell_command")
                {
                    command_results.push(format!("git status --short denied: {}", decision.reason));
                }
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                let Some(tool_name) = tool_by_call_id.get(&completed.call_id) else {
                    continue;
                };
                match tool_name.as_str() {
                    "workspace.git_status" => {
                        changed_files = changed_files_from_status(&completed.output_json)?;
                    }
                    "workspace.shell_command" => {
                        command_results.push(command_result_from_shell(&completed.output_json)?);
                    }
                    _ => {}
                }
            }
            RuntimeEventPayload::FinalResponseProduced(response) => {
                final_answer = Some(response.message.clone());
            }
            _ => {}
        }
    }

    let final_answer = final_answer.unwrap_or_else(|| {
        render_final_answer(&changed_files, &command_results, audit_event_count)
    });

    Ok(LocalCodingOutcome {
        final_answer,
        changed_files,
        command_results,
        audit_event_count,
    })
}

fn run_local_coding_fixture_with_shell_policy(
    shell_decision: PolicyDecision,
) -> Result<LocalCodingRun, LocalCodingError> {
    let temp = tempfile::tempdir().map_err(|error| LocalCodingError::Fixture(error.to_string()))?;
    let root = temp.path().join("workspace");
    fs::create_dir(&root).map_err(|error| LocalCodingError::Fixture(error.to_string()))?;
    initialize_git_fixture(&root)?;

    let sandbox = WorkspaceSandbox::new(&root)
        .map_err(|error| LocalCodingError::Sandbox(error.to_string()))?;
    let registry = registry(sandbox)?;
    let policy = policy(shell_decision)?;
    let ids = RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    };
    let mut store = InMemoryEventStore::default();

    let readme = "README.md";
    let read_outcome = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-read-readme".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({ "path": readme }),
        },
    )?;
    require_completed(&read_outcome, "read README")?;

    let clean_status = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-git-status-clean".to_string(),
            tool_name: "workspace.git_status".to_string(),
            input_json: json!({ "cwd": "." }),
        },
    )?;
    require_completed(&clean_status, "git status before edit")?;

    let edit_outcome = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-edit-readme".to_string(),
            tool_name: "workspace.edit_file".to_string(),
            input_json: json!({
                "path": readme,
                "expected": "old text\n",
                "replacement": "new text\n",
                "apply": true
            }),
        },
    )?;
    require_completed(&edit_outcome, "edit README")?;

    let shell_outcome = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-shell-git-status".to_string(),
            tool_name: "workspace.shell_command".to_string(),
            input_json: json!({
                "program": "git",
                "args": ["status", "--short"],
                "cwd": ".",
                "timeout_ms": 30000,
                "env": {}
            }),
        },
    )?;
    if !matches!(
        shell_outcome.status,
        ToolLoopStatus::Completed | ToolLoopStatus::Denied
    ) {
        return Err(LocalCodingError::ToolWorkflow(format!(
            "shell command reached unexpected status: {:?}",
            shell_outcome.status
        )));
    }

    let changed_status = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-git-status-changed".to_string(),
            tool_name: "workspace.git_status".to_string(),
            input_json: json!({ "cwd": "." }),
        },
    )?;
    require_completed(&changed_status, "git status after edit")?;

    let diff_outcome = run_call(
        ids,
        &mut store,
        &registry,
        &policy,
        ToolCallRequest {
            call_id: "call-git-diff".to_string(),
            tool_name: "workspace.git_diff".to_string(),
            input_json: json!({
                "cwd": ".",
                "path": readme
            }),
        },
    )?;
    require_completed(&diff_outcome, "git diff")?;

    let events = stored_events(&store, ids.conversation_id)?;
    let audit_event_count = count_audit_events(&events);
    let mut outcome = replay_local_coding_outcome(&events)?;
    outcome.audit_event_count = audit_event_count;
    outcome.final_answer = render_final_answer(
        &outcome.changed_files,
        &outcome.command_results,
        audit_event_count,
    );

    store
        .append(runtime_event(
            ids,
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: outcome.final_answer.clone(),
            }),
        ))
        .map_err(|error| LocalCodingError::Storage(error.to_string()))?;

    Ok(LocalCodingRun {
        outcome,
        events: stored_events(&store, ids.conversation_id)?,
    })
}

fn initialize_git_fixture(root: &Path) -> Result<(), LocalCodingError> {
    run_fixture_git(root, &["init", "-q"])?;
    run_fixture_git(root, &["config", "user.name", "Agent"])?;
    run_fixture_git(root, &["config", "user.email", "agent@example.invalid"])?;
    fs::write(root.join("README.md"), "old text\n")
        .map_err(|error| LocalCodingError::Fixture(error.to_string()))?;
    run_fixture_git(root, &["add", "README.md"])?;
    run_fixture_git(root, &["commit", "-m", "initial", "--quiet"])?;
    Ok(())
}

fn run_fixture_git(root: &Path, args: &[&str]) -> Result<(), LocalCodingError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| LocalCodingError::Fixture(error.to_string()))?;
    if !output.status.success() {
        return Err(LocalCodingError::Fixture(format!(
            "git fixture command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

fn registry(sandbox: WorkspaceSandbox) -> Result<ToolRegistry, LocalCodingError> {
    let mut registry = ToolRegistry::default();
    registry
        .register(ReadFileTool::new(sandbox.clone()))
        .map_err(|error| LocalCodingError::ToolRegistration(error.to_string()))?;
    registry
        .register(FileEditTool::new(sandbox.clone()))
        .map_err(|error| LocalCodingError::ToolRegistration(error.to_string()))?;
    registry
        .register(ShellCommandTool::new(sandbox.clone()))
        .map_err(|error| LocalCodingError::ToolRegistration(error.to_string()))?;
    registry
        .register(GitStatusTool::new(sandbox.clone()))
        .map_err(|error| LocalCodingError::ToolRegistration(error.to_string()))?;
    registry
        .register(GitDiffTool::new(sandbox))
        .map_err(|error| LocalCodingError::ToolRegistration(error.to_string()))?;
    Ok(registry)
}

fn policy(shell_decision: PolicyDecision) -> Result<StaticPolicyEngine, LocalCodingError> {
    let allow = PolicyDecision::Allow {
        reason: "fixture policy allows tool".to_string(),
    };
    let mut policy = StaticPolicyEngine::default();
    for tool_name in [
        "workspace.read_file",
        "workspace.edit_file",
        "workspace.git_status",
        "workspace.git_diff",
    ] {
        policy = policy.with_decision(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::Tool, tool_name)
                .map_err(|error| LocalCodingError::ToolWorkflow(error.to_string()))?,
            allow.clone(),
        );
    }

    Ok(policy.with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.shell_command")
            .map_err(|error| LocalCodingError::ToolWorkflow(error.to_string()))?,
        shell_decision,
    ))
}

fn run_call(
    ids: RuntimeIds,
    store: &mut InMemoryEventStore,
    registry: &ToolRegistry,
    policy: &StaticPolicyEngine,
    call: ToolCallRequest,
) -> Result<ToolLoopOutcome, LocalCodingError> {
    let outcome = ToolLoop
        .run(ids, call, registry, policy)
        .map_err(|error| LocalCodingError::ToolWorkflow(error.to_string()))?;
    for event in outcome.events.iter().cloned() {
        store
            .append(event)
            .map_err(|error| LocalCodingError::Storage(error.to_string()))?;
    }
    Ok(outcome)
}

fn require_completed(outcome: &ToolLoopOutcome, label: &str) -> Result<(), LocalCodingError> {
    if outcome.status != ToolLoopStatus::Completed {
        return Err(LocalCodingError::ToolWorkflow(format!(
            "{label} reached unexpected status: {:?}",
            outcome.status
        )));
    }
    Ok(())
}

fn stored_events(
    store: &InMemoryEventStore,
    conversation_id: ConversationId,
) -> Result<Vec<RuntimeEvent>, LocalCodingError> {
    Ok(store
        .list_by_conversation(conversation_id)
        .map_err(|error| LocalCodingError::Storage(error.to_string()))?
        .into_iter()
        .map(|stored| stored.event)
        .collect())
}

fn changed_files_from_status(value: &serde_json::Value) -> Result<Vec<String>, LocalCodingError> {
    let files = value
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| LocalCodingError::Replay("git status output missing files".to_string()))?;

    files
        .iter()
        .map(|file| {
            file.get("path")
                .and_then(serde_json::Value::as_str)
                .map(std::string::ToString::to_string)
                .ok_or_else(|| LocalCodingError::Replay("git status file missing path".to_string()))
        })
        .collect()
}

fn command_result_from_shell(value: &serde_json::Value) -> Result<String, LocalCodingError> {
    let status = value
        .get("status")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| LocalCodingError::Replay("shell output missing status".to_string()))?;
    let stdout = value
        .get("stdout")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim();

    if stdout.is_empty() {
        Ok(format!("git status --short exited {status}"))
    } else {
        Ok(format!("git status --short exited {status}: {stdout}"))
    }
}

fn count_audit_events(events: &[RuntimeEvent]) -> usize {
    events
        .iter()
        .filter(|event| {
            matches!(
                event.payload,
                RuntimeEventPayload::ToolCallProposed(_)
                    | RuntimeEventPayload::PermissionDecisionRecorded(_)
                    | RuntimeEventPayload::ToolExecutionStarted(_)
                    | RuntimeEventPayload::ToolExecutionCompleted(_)
                    | RuntimeEventPayload::ToolExecutionFailed(_)
                    | RuntimeEventPayload::ToolObservationRecorded(_)
            )
        })
        .count()
}

fn render_final_answer(
    changed_files: &[String],
    command_results: &[String],
    audit_event_count: usize,
) -> String {
    let files = if changed_files.is_empty() {
        "none".to_string()
    } else {
        changed_files.join(", ")
    };
    let commands = if command_results.is_empty() {
        "none".to_string()
    } else {
        command_results.join("; ")
    };

    format!(
        "Local coding workflow completed. Changed files: {files}. Commands: {commands}. Audit events: {audit_event_count}."
    )
}

fn runtime_event(ids: RuntimeIds, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload,
    }
}
