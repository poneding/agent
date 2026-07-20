use ruse_core::ModelInvocationAttempt;
use ruse_ui_bridge::{UiJsonFrame, UiModelInvocationAttemptSnapshot, encode_json_frame};

#[test]
fn model_invocation_attempt_frame_serializes_redacted_bounded_diagnostic() {
    let attempts = vec![
        ModelInvocationAttempt {
            provider_id: "freemodel".to_string(),
            model: "gpt-5.5".to_string(),
            attempt_index: 1,
            request_id: Some("req-sk-secret".to_string()),
            elapsed_ms: Some(42),
        },
        ModelInvocationAttempt {
            provider_id: "backup-token-provider".to_string(),
            model: "fallback-model".to_string(),
            attempt_index: 2,
            request_id: Some(format!("fallback-{}", "x".repeat(300))),
            elapsed_ms: Some(84),
        },
    ];

    let snapshot = UiModelInvocationAttemptSnapshot::new(
        "failed",
        &attempts,
        Some(format!(
            "transport failed with token sk-secret {}",
            "x".repeat(3_000)
        )),
    )
    .expect("snapshot");

    assert_eq!(snapshot.attempts.len(), 2);
    assert_eq!(snapshot.attempts[0].elapsed_ms, Some(42));
    assert!(
        snapshot.attempts[0]
            .request_id
            .as_deref()
            .is_some_and(
                |request_id| request_id.contains("[redacted]") && !request_id.contains("sk-secret")
            )
    );
    assert!(
        snapshot.attempts[1]
            .request_id
            .as_ref()
            .expect("request id")
            .chars()
            .count()
            <= 120
    );
    assert!(
        snapshot
            .message
            .as_ref()
            .expect("message")
            .contains("[redacted]")
    );
    assert!(
        !snapshot
            .message
            .as_ref()
            .expect("message")
            .contains("sk-secret")
    );
    assert!(snapshot.message.as_ref().expect("message").chars().count() <= 2_000);

    let line = encode_json_frame(&UiJsonFrame::ModelInvocationAttempts(snapshot)).expect("encode");
    let decoded: UiJsonFrame = serde_json::from_str(line.trim_end()).expect("decode");
    assert!(matches!(decoded, UiJsonFrame::ModelInvocationAttempts(_)));
}
