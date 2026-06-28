#![forbid(unsafe_code)]
//! Provider-neutral telemetry contracts for agent.

pub mod export;
pub mod metric;
pub mod sink;
pub mod span;

use thiserror::Error;

pub use export::{
    TELEMETRY_EXPORT_SCHEMA_VERSION, TelemetryExportBundle, TelemetryExportError,
    TelemetryExportOptions, TelemetryExportRecord, TelemetryExportRecordType,
    TelemetryExportSummary, export_telemetry_bundle, export_telemetry_bundle_at,
};
pub use metric::TelemetryMetric;
pub use sink::{InMemoryTelemetrySink, TelemetryRecord, TelemetrySink};
pub use span::{TelemetryAttribute, TelemetrySpan, TraceContext};

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TelemetryError {
    #[error("telemetry field {field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("telemetry metric {name} must have a finite value")]
    InvalidMetricValue { name: String },
}
