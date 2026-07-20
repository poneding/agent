use std::path::{Path, PathBuf};

use ruse_cli::{
    CliCommand, OutputMode, SkillRootCommand, SkillRootCommandRuntime, render_skill_root_command,
};
use ruse_core::ExtensionHealthStatus;
use serde_json::Value;

fn write_skill(root: &Path, name: &str, description: &str, body: &str) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(
        root.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {description}\n---\n{body}\n"),
    )
    .unwrap();
}

fn parse(args: impl IntoIterator<Item = &'static str>) -> ruse_cli::CliInvocation {
    ruse_cli::parse_cli_invocation(args).expect("parse invocation")
}

#[test]
fn skill_root_commands_parse_management_subcommands() {
    assert_eq!(
        parse(["skill", "roots", "add", "skills/code-review", "--enable"]).command,
        CliCommand::SkillRoot(SkillRootCommand::Add {
            path: PathBuf::from("skills/code-review"),
            enable: true,
        })
    );
    assert_eq!(
        parse(["skill", "roots", "list"]).command,
        CliCommand::SkillRoot(SkillRootCommand::List)
    );
    assert_eq!(
        parse(["skill", "roots", "inspect", "skill-root-abc"]).command,
        CliCommand::SkillRoot(SkillRootCommand::Inspect {
            id: "skill-root-abc".to_string(),
        })
    );
    assert_eq!(
        parse(["skill", "roots", "enable", "skill-root-abc"]).command,
        CliCommand::SkillRoot(SkillRootCommand::Enable {
            id: "skill-root-abc".to_string(),
        })
    );
    assert_eq!(
        parse(["skill", "roots", "disable", "skill-root-abc"]).command,
        CliCommand::SkillRoot(SkillRootCommand::Disable {
            id: "skill-root-abc".to_string(),
        })
    );
    assert_eq!(
        parse(["skill", "roots", "remove", "skill-root-abc"]).command,
        CliCommand::SkillRoot(SkillRootCommand::Remove {
            id: "skill-root-abc".to_string(),
        })
    );
}

#[test]
fn skill_root_commands_add_list_inspect_disable_enable_and_remove() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("code-review");
    write_skill(
        &root,
        "Code Review",
        "Review code changes",
        "Check diffs carefully.",
    );

    let mut runtime = SkillRootCommandRuntime::default();
    let add = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Add {
            path: root,
            enable: true,
        },
        OutputMode::Json,
    );
    assert_eq!(add.status_code, 0);
    let value: Value = serde_json::from_str(add.stdout.trim()).unwrap();
    assert_eq!(value["type"], "skill_root_added");
    assert_eq!(value["data"]["enabled"], true);
    let root_id = value["data"]["root_id"].as_str().unwrap().to_string();
    assert_eq!(runtime.registry().snapshot().records.len(), 1);

    let list = render_skill_root_command(&mut runtime, SkillRootCommand::List, OutputMode::Text);
    assert!(list.stdout.contains(&root_id));
    assert!(list.stdout.contains("enabled=true"));

    let inspect = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Inspect {
            id: root_id.clone(),
        },
        OutputMode::Json,
    );
    let inspected: Value = serde_json::from_str(inspect.stdout.trim()).unwrap();
    assert_eq!(inspected["type"], "skill_root_inspection");
    assert_eq!(
        inspected["data"]["discovered"][0]["skill_id"],
        "code-review"
    );
    assert!(!inspect.stdout.contains("Check diffs carefully"));

    let disabled = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Disable {
            id: root_id.clone(),
        },
        OutputMode::Text,
    );
    assert!(disabled.stdout.contains("Disabled skill root"));

    let enabled = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Enable {
            id: root_id.clone(),
        },
        OutputMode::Text,
    );
    assert!(enabled.stdout.contains("Enabled skill root"));

    let removed = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Remove { id: root_id },
        OutputMode::Text,
    );
    assert!(removed.stdout.contains("Removed skill root"));
    assert!(runtime.registry().snapshot().records.is_empty());
}

#[test]
fn skill_root_commands_render_diagnostics_and_redacted_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("broken");
    std::fs::create_dir_all(&root).unwrap();

    let mut runtime = SkillRootCommandRuntime::default();
    let add = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Add {
            path: root,
            enable: true,
        },
        OutputMode::Text,
    );
    assert_eq!(add.status_code, 0);
    let id = runtime.roots().list()[0].root_id.clone();

    let inspect = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Inspect { id },
        OutputMode::Text,
    );
    assert!(inspect.stdout.contains("manifest_not_found"));

    let missing = render_skill_root_command(
        &mut runtime,
        SkillRootCommand::Add {
            path: PathBuf::from("/tmp/sk-secret-token"),
            enable: false,
        },
        OutputMode::Json,
    );
    assert_eq!(missing.status_code, 2);
    assert!(!missing.stdout.contains("sk-secret-token"));
    assert!(!missing.stdout.contains("token"));
    assert!(missing.stdout.contains("[redacted]"));
}

#[test]
fn skill_root_commands_list_refreshes_duplicate_skill_health() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    write_skill(
        &first,
        "Shared Skill",
        "Review code changes",
        "First prompt.",
    );
    write_skill(
        &second,
        "Shared Skill",
        "Review code changes differently",
        "Second prompt.",
    );

    let mut runtime = SkillRootCommandRuntime::default();
    for root in [first, second] {
        let add = render_skill_root_command(
            &mut runtime,
            SkillRootCommand::Add {
                path: root,
                enable: true,
            },
            OutputMode::Json,
        );
        assert_eq!(add.status_code, 0);
    }

    let list = render_skill_root_command(&mut runtime, SkillRootCommand::List, OutputMode::Json);
    assert_eq!(list.status_code, 0);
    let value: Value = serde_json::from_str(list.stdout.trim()).unwrap();
    assert_eq!(value["type"], "skill_root_list");
    assert_eq!(value["data"]["roots"].as_array().unwrap().len(), 2);

    let snapshot = runtime.registry().snapshot();
    assert_eq!(snapshot.records.len(), 2);
    assert!(snapshot.records.iter().all(|record| {
        record.health.status == ExtensionHealthStatus::Unhealthy
            && record.health.message == "skill root has diagnostics"
    }));
    let report = runtime.registry().health_smoke_report();
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "extension.unhealthy")
    );
}

#[test]
fn skill_root_commands_json_invocation_shape_is_script_safe() {
    let invocation = parse(["--json", "skill", "roots", "list"]);

    assert_eq!(invocation.output_mode, OutputMode::Json);
    assert_eq!(
        invocation.command,
        CliCommand::SkillRoot(SkillRootCommand::List)
    );
}
