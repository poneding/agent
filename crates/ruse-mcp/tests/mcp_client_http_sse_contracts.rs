use ruse_mcp::parse_mcp_client_http_sse_events;

#[test]
fn mcp_client_http_sse_contracts_parse_single_data_event() {
    let events = parse_mcp_client_http_sse_events("data: {\"ok\":true}\n\n").expect("events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "{\"ok\":true}");
    assert_eq!(events[0].event, None);
    assert_eq!(events[0].id, None);
    assert_eq!(events[0].retry_ms, None);
}

#[test]
fn mcp_client_http_sse_contracts_parse_multiline_data_event() {
    let events = parse_mcp_client_http_sse_events("data: first line\ndata: second line\n\n")
        .expect("events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "first line\nsecond line");
}

#[test]
fn mcp_client_http_sse_contracts_parse_metadata_and_ignore_comments() {
    let events = parse_mcp_client_http_sse_events(
        ": comment with sk-secret-token\nid: event-7\nevent: message\nretry: 2500\ndata: payload\n\n",
    )
    .expect("events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id.as_deref(), Some("event-7"));
    assert_eq!(events[0].event.as_deref(), Some("message"));
    assert_eq!(events[0].retry_ms, Some(2500));
    assert_eq!(events[0].data, "payload");
}

#[test]
fn mcp_client_http_sse_contracts_ignore_invalid_retry_unknown_fields_and_empty_blocks() {
    let events = parse_mcp_client_http_sse_events(
        "retry: not-a-number\nignored: field\n\nid: no-data\n\nretry: 100\ndata: done\n\n",
    )
    .expect("events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].retry_ms, Some(100));
    assert_eq!(events[0].data, "done");
}

#[test]
fn mcp_client_http_sse_contracts_dispatch_final_event_without_trailing_blank_line() {
    let events = parse_mcp_client_http_sse_events("data: final").expect("events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "final");
}

#[test]
fn mcp_client_http_sse_contracts_reject_oversized_body_without_leaking_content() {
    let body = format!("data: {}\n\n", "sk-secret-token".repeat(30_000));

    let error = parse_mcp_client_http_sse_events(&body).expect_err("oversized body");
    let message = error.to_string();

    assert!(message.contains("SSE response body too large"));
    assert!(!message.contains("sk-secret-token"));
}

#[test]
fn mcp_client_http_sse_contracts_reject_oversized_event_data_without_leaking_content() {
    let body = format!("data: {}\n\n", "sk-secret-token".repeat(6_000));

    let error = parse_mcp_client_http_sse_events(&body).expect_err("oversized data");
    let message = error.to_string();

    assert!(message.contains("SSE event data too large"));
    assert!(!message.contains("sk-secret-token"));
}
