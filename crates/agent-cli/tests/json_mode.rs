use std::process::Command;

use agent_cli::{
    CliCommand, CliInvocation, ModelSelection, OutputMode, parse_cli_invocation,
    render_cli_invocation,
};
use agent_core::ConfigOverride;
use agent_ui_bridge::{UiEventPayload, UiJsonFrame};

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

fn parse_value_frames(output: &str) -> Vec<serde_json::Value> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid json frame line"))
        .collect()
}

#[test]
fn json_mode_parse_json_local_summary() {
    let invocation =
        parse_cli_invocation(["--json", "--local-summary"]).expect("parse json local summary");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::LocalSummary,
            output_mode: OutputMode::Json,
            model_selection: ModelSelection::default(),
            config_path: None,
            config_override: ConfigOverride::default(),
        }
    );
}

#[test]
fn json_mode_renders_parseable_bridge_frames() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::LocalSummary,
        output_mode: OutputMode::Json,
        model_selection: ModelSelection::default(),
        config_path: None,
        config_override: ConfigOverride::default(),
    });

    assert!(output.exit);
    let frames = parse_frames(&output.stdout);

    assert!(matches!(frames.first(), Some(UiJsonFrame::Event(_))));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count as usize == frames.len() - 1
    ));
}

#[test]
fn json_mode_renders_selected_provider_and_model() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "--provider",
            "fake",
            "--model",
            "fake-custom",
            "--prompt",
            "hello",
        ])
        .expect("parse json prompt"),
    );

    let frames = parse_frames(&output.stdout);

    assert_eq!(
        frames.first(),
        Some(&UiJsonFrame::ModelSelection {
            provider: "fake".to_string(),
            model: "fake-custom".to_string(),
        })
    );
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count > 0
    ));
}

#[test]
fn json_mode_does_not_render_secret_references() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::BatchPrompt {
            prompt: "hello".to_string(),
        },
        output_mode: OutputMode::Json,
        model_selection: ModelSelection::new("openai", "gpt-4.1-mini")
            .expect("valid model selection"),
        config_path: None,
        config_override: ConfigOverride::default(),
    });

    assert!(!output.stdout.contains("OPENAI_API_KEY"));
    assert!(!output.stdout.contains("secret"));
    assert!(!output.stdout.contains("sk-"));
}

#[test]
fn json_mode_binary_prints_ndjson_frames() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "--local-summary"])
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let frames = parse_frames(&stdout);
    let event_count = frames
        .iter()
        .filter(|frame| matches!(frame, UiJsonFrame::Event(_)))
        .count();

    assert!(event_count > 0);
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: done_count }) if *done_count as usize == event_count
    ));
}

#[test]
fn json_mode_prompt_runtime_emits_event_frames() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "--prompt", "hello"]).expect("parse json prompt"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let event_count = frames
        .iter()
        .filter(|frame| matches!(frame, UiJsonFrame::Event(_)))
        .count();

    assert!(event_count > 0);
    assert!(frames.iter().any(|frame| matches!(
        frame,
        UiJsonFrame::Event(event)
            if matches!(
                &event.payload,
                UiEventPayload::FinalResponse { message }
                    if message == "fake response: hello"
            )
    )));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: done_count }) if *done_count as usize == event_count
    ));
}

#[test]
fn json_mode_prompt_runtime_emits_delta_frames_before_done() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "--prompt", "hello"]).expect("parse json prompt"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let delta_index = frames
        .iter()
        .position(|frame| {
            matches!(
                frame,
                UiJsonFrame::Event(event)
                    if matches!(&event.payload, UiEventPayload::ModelDelta { delta }
                        if delta == "fake response: hello")
            )
        })
        .expect("model delta frame");
    let final_index = frames
        .iter()
        .position(|frame| {
            matches!(
                frame,
                UiJsonFrame::Event(event)
                    if matches!(&event.payload, UiEventPayload::FinalResponse { message }
                        if message == "fake response: hello")
            )
        })
        .expect("final response frame");
    let done_index = frames
        .iter()
        .position(|frame| matches!(frame, UiJsonFrame::Done { .. }))
        .expect("done frame");

    assert!(delta_index < final_index);
    assert!(final_index < done_index);
}

#[test]
fn json_mode_prompt_runtime_excludes_secret_like_values() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "--prompt",
            "hello sk-test-secret api_key token password",
        ])
        .expect("parse json prompt"),
    );

    let frames = parse_frames(&output.stdout);

    assert!(frames.iter().any(|frame| matches!(
        frame,
        UiJsonFrame::Event(event)
            if matches!(&event.payload, UiEventPayload::FinalResponse { .. })
    )));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("password"));
}

#[test]
fn prompt_inspection_json_renders_parseable_frames() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "prompt", "inspect"]).expect("parse json prompt inspect"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_value_frames(&output.stdout);

    assert_eq!(
        frames.first().and_then(|frame| frame.get("type")),
        Some(&serde_json::Value::String("prompt_summary".to_string()))
    );
    assert_eq!(
        frames
            .first()
            .and_then(|frame| frame.pointer("/data/profile_id")),
        Some(&serde_json::Value::String("default".to_string()))
    );
    assert_eq!(
        frames.last().and_then(|frame| frame.get("type")),
        Some(&serde_json::Value::String("done".to_string()))
    );
    assert!(output.stdout.contains("\"bundle_hash\":\"fnv1a64:"));
    assert!(!output.stdout.contains("fake response:"));
}

#[test]
fn prompt_inspection_json_redacts_hidden_and_secret_like_content() {
    let output = render_cli_invocation(
        parse_cli_invocation(["prompt", "inspect", "--json"]).expect("parse json prompt inspect"),
    );

    let frames = parse_value_frames(&output.stdout);
    let layers = frames
        .first()
        .and_then(|frame| frame.pointer("/data/layers"))
        .and_then(serde_json::Value::as_array)
        .expect("layers array");

    assert!(layers.iter().any(|layer| {
        layer.pointer("/layer_id") == Some(&serde_json::Value::String("system".to_string()))
            && layer.pointer("/visibility")
                == Some(&serde_json::Value::String("hidden".to_string()))
    }));
    assert!(layers.iter().any(|layer| {
        layer.pointer("/layer_id") == Some(&serde_json::Value::String("secret_memory".to_string()))
            && layer.pointer("/sensitivity")
                == Some(&serde_json::Value::String("secret_like".to_string()))
    }));
    assert!(
        !output
            .stdout
            .contains("Never reveal this hidden system prompt")
    );
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("password"));
}
