use agent_telemetry::{
    InMemoryTelemetrySink, TelemetryAttribute, TelemetryError, TelemetryMetric, TelemetryRecord,
    TelemetrySink, TelemetrySpan, TraceContext,
};
use serde_json::json;

#[test]
fn telemetry_span_serializes_stable_shape() {
    let context = TraceContext::new("trace-1", "span-1", Some("parent-1")).expect("valid context");
    let span = TelemetrySpan::new(
        "turn.execute",
        context,
        vec![TelemetryAttribute::new("workspace_id", "workspace-1").expect("attribute")],
    )
    .expect("valid span");

    assert_eq!(
        serde_json::to_value(span).expect("serialize span"),
        json!({
            "name": "turn.execute",
            "context": {
                "trace_id": "trace-1",
                "span_id": "span-1",
                "parent_span_id": "parent-1"
            },
            "attributes": [{
                "key": "workspace_id",
                "value": "workspace-1"
            }]
        })
    );
}

#[test]
fn telemetry_metric_rejects_non_finite_value() {
    let error = TelemetryMetric::new("tool.duration", "ms", f64::NAN, Vec::new())
        .expect_err("nan metric value rejected");

    assert!(matches!(
        error,
        TelemetryError::InvalidMetricValue { name } if name == "tool.duration"
    ));
}

#[test]
fn telemetry_sink_records_spans_and_metrics_in_order() {
    let context = TraceContext::new("trace-1", "span-1", None).expect("valid context");
    let span = TelemetrySpan::new("turn.execute", context, Vec::new()).expect("valid span");
    let metric =
        TelemetryMetric::new("tool.duration", "ms", 42.0, Vec::new()).expect("valid metric");
    let mut sink = InMemoryTelemetrySink::default();

    sink.record_span(span);
    sink.record_metric(metric);

    assert!(matches!(
        &sink.records()[0],
        TelemetryRecord::Span(stored) if stored.name == "turn.execute"
    ));
    assert!(matches!(
        &sink.records()[1],
        TelemetryRecord::Metric(stored) if stored.name == "tool.duration"
    ));
    assert_eq!(sink.spans().len(), 1);
    assert_eq!(sink.metrics().len(), 1);
}

#[test]
fn telemetry_redacts_secret_like_attributes() {
    let context = TraceContext::new("trace-1", "span-1", None).expect("valid context");
    let span = TelemetrySpan::new(
        "tool.call",
        context,
        vec![
            TelemetryAttribute::new("api_token", "sk-live-secret").expect("secret key attribute"),
            TelemetryAttribute::new("workspace", "workspace-1").expect("plain attribute"),
            TelemetryAttribute::new("auth_header", "Bearer sk-env-secret")
                .expect("secret value attribute"),
        ],
    )
    .expect("valid span");
    let mut sink = InMemoryTelemetrySink::default();

    sink.record_span(span);

    let attributes = &sink.spans()[0].attributes;
    assert_eq!(attributes[0].value, "[redacted]");
    assert_eq!(attributes[1].value, "workspace-1");
    assert_eq!(attributes[2].value, "[redacted]");
}

#[test]
fn telemetry_trace_context_rejects_empty_identifiers() {
    assert!(matches!(
        TraceContext::new("", "span-1", None),
        Err(TelemetryError::EmptyField { field }) if field == "trace_id"
    ));
    assert!(matches!(
        TraceContext::new("trace-1", " ", None),
        Err(TelemetryError::EmptyField { field }) if field == "span_id"
    ));
    assert!(matches!(
        TraceContext::new("trace-1", "span-1", Some("")),
        Err(TelemetryError::EmptyField { field }) if field == "parent_span_id"
    ));
}
