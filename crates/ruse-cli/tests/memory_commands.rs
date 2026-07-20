use ruse_cli::{
    CliCommand, MemoryCommand, MemoryCommandRuntime, OutputMode, parse_cli_command,
    parse_cli_invocation, render_cli_command, render_cli_invocation, render_memory_command,
};
use ruse_memory::{
    MemoryArchiveRecord, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource,
    MemoryTombstone,
};
use ruse_types::{EventId, WorkspaceId};
use serde_json::json;

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000002605"
        .parse::<WorkspaceId>()
        .expect("workspace id")
}

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000002615"
        .parse::<EventId>()
        .expect("event id")
}

fn archive_json() -> serde_json::Value {
    let scope = MemoryScope::Project {
        workspace_id: workspace_id(),
    };
    let record = MemoryRecord {
        id: "imported-memory".to_string(),
        scope: scope.clone(),
        text: "imported portable memory".to_string(),
        source: MemorySource {
            event_id: Some(event_id()),
            description: "import fixture".to_string(),
        },
        confidence: 90,
        sensitivity: MemorySensitivity::Public,
    };

    json!({
        "schema_version": 1,
        "manifest": {
            "exported_by": "ruse-memory",
            "record_count": 1,
            "sensitive_record_count": 0
        },
        "records": [MemoryArchiveRecord::from(record)],
        "tombstones": [MemoryTombstone {
            tombstone_id: "tombstone:imported".to_string(),
            scope,
            record_id: "old-memory".to_string(),
            reason: "already deleted".to_string(),
            source_event_id: Some(event_id()),
        }]
    })
}

#[test]
fn memory_commands_parse_memory_subcommands() {
    assert_eq!(
        parse_cli_command(["memory", "list"]).expect("parse list"),
        CliCommand::Memory(MemoryCommand::List)
    );
    assert_eq!(
        parse_cli_command(["memory", "inspect", "public-memory"]).expect("parse inspect"),
        CliCommand::Memory(MemoryCommand::Inspect {
            id: "public-memory".to_string()
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "delete", "public-memory", "--reason", "cleanup"])
            .expect("parse delete"),
        CliCommand::Memory(MemoryCommand::Delete {
            id: "public-memory".to_string(),
            reason: "cleanup".to_string()
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "export"]).expect("parse export"),
        CliCommand::Memory(MemoryCommand::Export)
    );
    assert_eq!(
        parse_cli_command(["memory", "import", "memory.json"]).expect("parse import"),
        CliCommand::Memory(MemoryCommand::Import {
            path: "memory.json".into()
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "rebuild-index"]).expect("parse rebuild"),
        CliCommand::Memory(MemoryCommand::RebuildIndex)
    );
    assert_eq!(
        parse_cli_command(["memory", "snapshot"]).expect("parse snapshot"),
        CliCommand::Memory(MemoryCommand::Snapshot)
    );
    assert_eq!(
        parse_cli_command([
            "memory",
            "retrieve",
            "--query",
            "fixture memory",
            "--max-selected",
            "2",
        ])
        .expect("parse retrieve"),
        CliCommand::Memory(MemoryCommand::Retrieve {
            query: "fixture memory".to_string(),
            max_selected: 2,
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "proposal", "list"]).expect("parse proposal list"),
        CliCommand::Memory(MemoryCommand::ProposalList)
    );
    assert_eq!(
        parse_cli_command(["memory", "proposal", "inspect", "proposal-public"])
            .expect("parse proposal inspect"),
        CliCommand::Memory(MemoryCommand::ProposalInspect {
            id: "proposal-public".to_string(),
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "proposal", "approve", "proposal-public"])
            .expect("parse proposal approve"),
        CliCommand::Memory(MemoryCommand::ProposalApprove {
            id: "proposal-public".to_string(),
        })
    );
    assert_eq!(
        parse_cli_command([
            "memory",
            "proposal",
            "reject",
            "proposal-sensitive",
            "--reason",
            "contains sk-secret token",
        ])
        .expect("parse proposal reject"),
        CliCommand::Memory(MemoryCommand::ProposalReject {
            id: "proposal-sensitive".to_string(),
            reason: "contains sk-secret token".to_string(),
        })
    );
    assert_eq!(
        parse_cli_command([
            "memory",
            "proposal",
            "expire",
            "proposal-sensitive",
            "--reason",
            "stale",
        ])
        .expect("parse proposal expire"),
        CliCommand::Memory(MemoryCommand::ProposalExpire {
            id: "proposal-sensitive".to_string(),
            reason: "stale".to_string(),
        })
    );
    assert_eq!(
        parse_cli_command(["memory", "proposal", "commit", "proposal-public"])
            .expect("parse proposal commit"),
        CliCommand::Memory(MemoryCommand::ProposalCommit {
            id: "proposal-public".to_string(),
        })
    );
}

#[test]
fn memory_commands_render_text_list_and_inspect_without_sensitive_text() {
    let list = render_cli_command(CliCommand::Memory(MemoryCommand::List));

    assert_eq!(list.status_code, 0);
    assert!(list.stdout.contains("public-memory"));
    assert!(list.stdout.contains("sensitive-memory"));
    assert!(!list.stdout.contains("private raw memory"));

    let inspect = render_cli_command(CliCommand::Memory(MemoryCommand::Inspect {
        id: "public-memory".to_string(),
    }));
    assert_eq!(inspect.status_code, 0);
    assert!(inspect.stdout.contains("Memory public-memory"));
    assert!(inspect.stdout.contains("public fixture memory"));
}

#[test]
fn memory_commands_render_json_list_with_safe_summaries() {
    let invocation = parse_cli_invocation(["--json", "memory", "list"]).expect("parse json list");
    assert_eq!(invocation.output_mode, OutputMode::Json);

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("\"type\":\"memory_list\""));
    assert!(output.stdout.contains("public-memory"));
    assert!(output.stdout.contains("sensitive-memory"));
    assert!(!output.stdout.contains("private raw memory"));
}

#[test]
fn memory_commands_render_json_snapshot_with_product_contracts() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "memory", "snapshot"]).expect("parse snapshot"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("\"type\":\"memory_snapshot\""));
    assert!(output.stdout.contains("\"records\""));
    assert!(output.stdout.contains("\"proposals\""));
    assert!(output.stdout.contains("\"retention\""));
    assert!(output.stdout.contains("public-memory"));
    assert!(output.stdout.contains("proposal-public"));
    assert!(!output.stdout.contains("private raw memory"));
    assert!(!output.stdout.contains("raw sk-secret token"));
}

#[test]
fn memory_commands_render_retrieval_preview_without_sensitive_text() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "memory",
            "retrieve",
            "--query",
            "fixture memory",
            "--max-selected",
            "1",
        ])
        .expect("parse retrieve"),
    );

    assert_eq!(output.status_code, 0);
    assert!(
        output
            .stdout
            .contains("\"type\":\"memory_retrieval_preview\"")
    );
    assert!(output.stdout.contains("\"selected_count\":1"));
    assert!(output.stdout.contains("\"rejected_count\":1"));
    assert!(output.stdout.contains("\"influence\""));
    assert!(output.stdout.contains("[redacted]"));
    assert!(!output.stdout.contains("private raw memory"));
}

#[test]
fn memory_commands_render_proposal_review_and_decisions() {
    let mut runtime = MemoryCommandRuntime::default();

    let list = render_memory_command(&mut runtime, MemoryCommand::ProposalList, OutputMode::Json);
    assert_eq!(list.status_code, 0);
    assert!(list.stdout.contains("\"type\":\"memory_proposal_list\""));
    assert!(list.stdout.contains("proposal-public"));
    assert!(list.stdout.contains("proposal-sensitive"));
    assert!(!list.stdout.contains("raw sk-secret token"));

    let inspect = render_memory_command(
        &mut runtime,
        MemoryCommand::ProposalInspect {
            id: "proposal-sensitive".to_string(),
        },
        OutputMode::Text,
    );
    assert_eq!(inspect.status_code, 0);
    assert!(inspect.stdout.contains("Proposal proposal-sensitive"));
    assert!(inspect.stdout.contains("requires_explicit_approval=true"));
    assert!(!inspect.stdout.contains("raw sk-secret token"));

    let approved = render_memory_command(
        &mut runtime,
        MemoryCommand::ProposalApprove {
            id: "proposal-public".to_string(),
        },
        OutputMode::Json,
    );
    assert_eq!(approved.status_code, 0);
    assert!(
        approved
            .stdout
            .contains("\"type\":\"memory_proposal_decision\"")
    );
    assert!(approved.stdout.contains("\"status\":\"approved\""));

    let committed = render_memory_command(
        &mut runtime,
        MemoryCommand::ProposalCommit {
            id: "proposal-public".to_string(),
        },
        OutputMode::Json,
    );
    assert_eq!(committed.status_code, 0);
    assert!(
        committed
            .stdout
            .contains("\"type\":\"memory_proposal_commit\"")
    );
    assert!(committed.stdout.contains("\"record\""));
    assert!(committed.stdout.contains("proposal-committed-memory"));

    let rejected = render_memory_command(
        &mut runtime,
        MemoryCommand::ProposalReject {
            id: "proposal-sensitive".to_string(),
            reason: "contains sk-secret token".to_string(),
        },
        OutputMode::Json,
    );
    assert_eq!(rejected.status_code, 0);
    assert!(rejected.stdout.contains("\"status\":\"rejected\""));
    assert!(rejected.stdout.contains("[redacted]"));
    assert!(!rejected.stdout.contains("sk-secret"));
}

#[test]
fn memory_commands_reject_invalid_proposal_transitions_with_redacted_errors() {
    let mut runtime = MemoryCommandRuntime::default();

    let output = render_memory_command(
        &mut runtime,
        MemoryCommand::ProposalCommit {
            id: "proposal-sensitive".to_string(),
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("\"type\":\"memory_error\""));
    assert!(output.stdout.contains("approved"));
    assert!(!output.stdout.contains("raw sk-secret token"));
}

#[test]
fn memory_commands_delete_redacts_reason_in_text_and_json() {
    let text = render_cli_command(CliCommand::Memory(MemoryCommand::Delete {
        id: "public-memory".to_string(),
        reason: "remove sk-secret token".to_string(),
    }));
    assert_eq!(text.status_code, 0);
    assert!(text.stdout.contains("remove [redacted] [redacted]"));
    assert!(!text.stdout.contains("sk-secret"));

    let json = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "memory",
            "delete",
            "public-memory",
            "--reason",
            "remove sk-secret token",
        ])
        .expect("parse json delete"),
    );
    assert_eq!(json.status_code, 0);
    assert!(json.stdout.contains("\"type\":\"memory_deleted\""));
    assert!(json.stdout.contains("[redacted]"));
    assert!(!json.stdout.contains("sk-secret"));
}

#[test]
fn memory_commands_export_json_omits_sensitive_records_by_default() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "memory", "export"]).expect("parse export"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("\"type\":\"memory_export\""));
    assert!(output.stdout.contains("public-memory"));
    assert!(!output.stdout.contains("private raw memory"));
    assert!(!output.stdout.contains("embedding"));
}

#[test]
fn memory_commands_import_archive_and_rebuild_index() {
    let temp = tempfile::NamedTempFile::new().expect("temp archive");
    std::fs::write(
        temp.path(),
        serde_json::to_string(&archive_json()).expect("archive json"),
    )
    .expect("write archive");

    let import = render_cli_invocation(
        parse_cli_invocation(["--json", "memory", "import", temp.path().to_str().unwrap()])
            .expect("parse import"),
    );
    assert_eq!(import.status_code, 0);
    assert!(import.stdout.contains("\"type\":\"memory_import\""));
    assert!(import.stdout.contains("\"accepted\":1"));
    assert!(import.stdout.contains("\"tombstones\":1"));

    let rebuild = render_cli_command(CliCommand::Memory(MemoryCommand::RebuildIndex));
    assert_eq!(rebuild.status_code, 0);
    assert!(rebuild.stdout.contains("indexed_records="));
    assert!(rebuild.stdout.contains("stale_embeddings_removed="));
}
