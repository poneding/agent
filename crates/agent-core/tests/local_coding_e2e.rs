use agent_core::{
    replay_local_coding_outcome, run_local_coding_fixture,
    run_local_coding_fixture_with_shell_denied,
};
use agent_types::RuntimeEventPayload;

#[test]
fn local_coding_e2e_runs_read_edit_shell_git_workflow() {
    let run = run_local_coding_fixture().expect("local coding workflow");

    assert_eq!(run.outcome.changed_files, vec!["README.md"]);
    assert_eq!(
        run.outcome.command_results,
        vec!["git status --short exited 0: M README.md"]
    );
    assert_eq!(run.outcome.audit_event_count, 30);
    assert_eq!(
        run.outcome.final_answer,
        "Local coding workflow completed. Changed files: README.md. Commands: git status --short exited 0: M README.md. Audit events: 30."
    );
    assert!(
        run.events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::FinalResponseProduced(_)))
    );
}

#[test]
fn local_coding_e2e_denied_command_is_not_executed() {
    let run = run_local_coding_fixture_with_shell_denied().expect("local coding workflow");
    let shell_call_id = run
        .events
        .iter()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(call)
                if call.tool_name == "workspace.shell_command" =>
            {
                Some(call.call_id.clone())
            }
            _ => None,
        })
        .expect("shell call proposed");

    assert_eq!(
        run.outcome.command_results,
        vec!["git status --short denied: denied by fixture policy"]
    );
    assert_eq!(run.outcome.audit_event_count, 27);
    assert!(
        !run.events.iter().any(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionStarted(started) => started.call_id == shell_call_id,
            _ => false,
        }),
        "denied shell command must not start execution"
    );
}

#[test]
fn local_coding_e2e_replays_final_summary_from_events() {
    let run = run_local_coding_fixture().expect("local coding workflow");

    let replayed = replay_local_coding_outcome(&run.events).expect("replay outcome");

    assert_eq!(replayed, run.outcome);
}
