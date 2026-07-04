use agent_telemetry::{
    InMemoryTelemetrySink, TelemetryAttribute, TelemetryExportError, TelemetryExportOptions,
    TelemetryExportRecordType, TelemetryMetric, TelemetrySink, TelemetrySpan, TraceContext,
    export_telemetry_bundle,
};
use serde_json::json;

fn span(name: &str, span_id: &str, attributes: Vec<TelemetryAttribute>) -> TelemetrySpan {
    TelemetrySpan::new(
        name,
        TraceContext::new("trace-1", span_id, None).expect("valid context"),
        attributes,
    )
    .expect("valid span")
}

fn metric(name: &str, attributes: Vec<TelemetryAttribute>) -> TelemetryMetric {
    TelemetryMetric::new(name, "ms", 12.5, attributes).expect("valid metric")
}

#[test]
fn telemetry_export_serializes_stable_bundle_shape() {
    let mut sink = InMemoryTelemetrySink::default();
    sink.record_span(span(
        "turn.execute",
        "span-1",
        vec![TelemetryAttribute::new("workspace_id", "workspace-1").expect("attribute")],
    ));
    sink.record_metric(metric(
        "tool.duration",
        vec![TelemetryAttribute::new("tool", "workspace.read_file").expect("attribute")],
    ));

    let bundle = export_telemetry_bundle(sink.records(), TelemetryExportOptions::default())
        .expect("export telemetry");
    let value = serde_json::to_value(bundle).expect("serialize bundle");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(
        value["summary"],
        json!({
            "total_input_records": 2,
            "exported_records": 2,
            "span_records": 1,
            "metric_records": 1,
            "filtered_records": 0,
            "truncated_attributes": 0
        })
    );
    assert_eq!(value["records"][0]["index"], 0);
    assert_eq!(value["records"][0]["record_type"], "span");
    assert_eq!(value["records"][0]["name"], "turn.execute");
    assert_eq!(value["records"][0]["context"]["span_id"], "span-1");
    assert_eq!(value["records"][1]["index"], 1);
    assert_eq!(value["records"][1]["record_type"], "metric");
    assert_eq!(value["records"][1]["unit"], "ms");
    assert_eq!(value["records"][1]["value"], 12.5);
}

#[test]
fn telemetry_export_filters_spans_and_metrics() {
    let mut sink = InMemoryTelemetrySink::default();
    sink.record_span(span("turn.execute", "span-1", Vec::new()));
    sink.record_metric(metric("tool.duration", Vec::new()));

    let bundle = export_telemetry_bundle(
        sink.records(),
        TelemetryExportOptions {
            include_spans: false,
            include_metrics: true,
            ..TelemetryExportOptions::default()
        },
    )
    .expect("export metrics only");

    assert_eq!(bundle.records.len(), 1);
    assert_eq!(
        bundle.records[0].record_type,
        TelemetryExportRecordType::Metric
    );
    assert_eq!(bundle.records[0].index, 1);
    assert_eq!(bundle.summary.total_input_records, 2);
    assert_eq!(bundle.summary.filtered_records, 1);
}

#[test]
fn telemetry_export_redacts_and_truncates_attributes() {
    let mut sink = InMemoryTelemetrySink::default();
    sink.record_span(span(
        "tool.call",
        "span-1",
        vec![
            TelemetryAttribute::new("api_token", "sk-live-secret").expect("secret attribute"),
            TelemetryAttribute::new("note", "abcdefghij").expect("long attribute"),
        ],
    ));

    let bundle = export_telemetry_bundle(
        sink.records(),
        TelemetryExportOptions {
            max_attribute_value_bytes: 5,
            ..TelemetryExportOptions::default()
        },
    )
    .expect("export telemetry");

    let attributes = &bundle.records[0].attributes;
    assert_eq!(attributes[0].value, "[redacted]");
    assert_eq!(attributes[1].value, "abcde");
    assert_eq!(bundle.summary.truncated_attributes, 1);
    assert_eq!(
        bundle.records[0].diagnostics,
        vec!["attribute note truncated to 5 bytes"]
    );
}

#[test]
fn telemetry_export_allows_empty_records() {
    let bundle = export_telemetry_bundle(&[], TelemetryExportOptions::default())
        .expect("empty export is valid");

    assert!(bundle.records.is_empty());
    assert_eq!(bundle.summary.total_input_records, 0);
    assert_eq!(bundle.summary.exported_records, 0);
}

#[test]
fn telemetry_export_rejects_invalid_options() {
    let no_budget = export_telemetry_bundle(
        &[],
        TelemetryExportOptions {
            max_attribute_value_bytes: 0,
            ..TelemetryExportOptions::default()
        },
    )
    .expect_err("zero budget rejected");
    assert_eq!(
        no_budget,
        TelemetryExportError::InvalidMaxAttributeValueBytes
    );

    let empty_selection = export_telemetry_bundle(
        &[],
        TelemetryExportOptions {
            include_spans: false,
            include_metrics: false,
            ..TelemetryExportOptions::default()
        },
    )
    .expect_err("empty selection rejected");
    assert_eq!(empty_selection, TelemetryExportError::NoRecordTypesSelected);
}
