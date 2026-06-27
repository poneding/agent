use agent_cli::{
    CliCommand, MemoryCommand, OutputMode, parse_cli_command, parse_cli_invocation,
    render_cli_command, render_cli_invocation,
};
use agent_memory::{
    MemoryArchiveRecord, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource,
    MemoryTombstone,
};
use agent_types::{EventId, WorkspaceId};
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
            "exported_by": "agent-memory",
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
