use ruse_context::{CompactRequest, ContextError, PinnedMessage, PromptMessage, compact_messages};

fn message(role: &str, content: &str) -> PromptMessage {
    PromptMessage {
        role: role.to_string(),
        content: content.to_string(),
    }
}

fn pin(id: &str, reason: &str, role: &str, content: &str) -> PinnedMessage {
    PinnedMessage {
        id: id.to_string(),
        reason: reason.to_string(),
        message: message(role, content),
    }
}

fn request() -> CompactRequest {
    CompactRequest {
        messages: vec![
            message("system", "first compactable fact"),
            message("user", "second compactable request"),
            message("assistant", "third compactable answer"),
        ],
        pinned: vec![
            pin(
                "pin-system",
                "system constraint",
                "system",
                "always ask before shell",
            ),
            pin(
                "pin-user",
                "user preference",
                "user",
                "keep responses concise",
            ),
        ],
        max_summary_tokens: 32,
    }
}

#[test]
fn compact_messages_preserves_pinned_messages() {
    let original = request();
    let result = compact_messages(original.clone()).expect("compact messages");

    assert_eq!(result.pinned, original.pinned);
    assert_eq!(result.dropped_message_count, 3);
    assert!(result.summary.is_some());
}

#[test]
fn compact_messages_produces_stable_summary() {
    let first = compact_messages(request()).expect("first compact");
    let second = compact_messages(request()).expect("second compact");
    let summary = first.summary.expect("summary");

    assert_eq!(Some(summary.clone()), second.summary);
    assert_eq!(summary.source_message_count, 3);
    assert!(summary.estimated_tokens <= 32);
    assert_eq!(
        summary.text,
        "system: first compactable fact\nuser: second compactable request\nassistant: third compactable answer"
    );
}

#[test]
fn compact_messages_rejects_duplicate_pin_ids() {
    let mut request = request();
    request
        .pinned
        .push(pin("pin-system", "duplicate", "system", "duplicate pin"));

    let error = compact_messages(request).expect_err("duplicate pin rejected");

    assert!(matches!(
        error,
        ContextError::InvalidPin(message) if message.contains("pin-system")
    ));
}

#[test]
fn compact_messages_rejects_zero_summary_budget_for_unpinned_messages() {
    let mut request = request();
    request.max_summary_tokens = 0;

    let error = compact_messages(request).expect_err("zero budget rejected");

    assert!(matches!(
        error,
        ContextError::InvalidCompaction(message) if message.contains("max_summary_tokens")
    ));
}
