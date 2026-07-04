use agent_types::Timestamp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{TelemetryAttribute, TelemetryMetric, TelemetryRecord, TelemetrySpan};

pub const TELEMETRY_EXPORT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_MAX_ATTRIBUTE_VALUE_BYTES: usize = 512;
const REDACTED_VALUE: &str = "[redacted]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryExportOptions {
    pub include_spans: bool,
    pub include_metrics: bool,
    pub max_attribute_value_bytes: usize,
}

impl Default for TelemetryExportOptions {
    fn default() -> Self {
        Self {
            include_spans: true,
            include_metrics: true,
            max_attribute_value_bytes: DEFAULT_MAX_ATTRIBUTE_VALUE_BYTES,
        }
    }
}

impl TelemetryExportOptions {
    fn validate(self) -> Result<Self, TelemetryExportError> {
        if self.max_attribute_value_bytes == 0 {
            return Err(TelemetryExportError::InvalidMaxAttributeValueBytes);
        }
        if !self.include_spans && !self.include_metrics {
            return Err(TelemetryExportError::NoRecordTypesSelected);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryExportBundle {
    pub schema_version: u32,
    pub generated_at: Timestamp,
    pub records: Vec<TelemetryExportRecord>,
    pub summary: TelemetryExportSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryExportRecord {
    pub index: u64,
    pub record_type: TelemetryExportRecordType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<crate::TraceContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    pub attributes: Vec<TelemetryAttribute>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryExportRecordType {
    Span,
    Metric,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelemetryExportSummary {
    pub total_input_records: usize,
    pub exported_records: usize,
    pub span_records: usize,
    pub metric_records: usize,
    pub filtered_records: usize,
    pub truncated_attributes: usize,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TelemetryExportError {
    #[error("telemetry export max_attribute_value_bytes must be greater than zero")]
    InvalidMaxAttributeValueBytes,
    #[error("telemetry export must include at least one record type")]
    NoRecordTypesSelected,
}

pub fn export_telemetry_bundle(
    records: &[TelemetryRecord],
    options: TelemetryExportOptions,
) -> Result<TelemetryExportBundle, TelemetryExportError> {
    export_telemetry_bundle_at(records, options, Timestamp::now_utc())
}

pub fn export_telemetry_bundle_at(
    records: &[TelemetryRecord],
    options: TelemetryExportOptions,
    generated_at: Timestamp,
) -> Result<TelemetryExportBundle, TelemetryExportError> {
    let options = options.validate()?;
    let mut summary = TelemetryExportSummary {
        total_input_records: records.len(),
        ..TelemetryExportSummary::default()
    };
    let mut exported = Vec::new();

    for (index, record) in records.iter().enumerate() {
        match record {
            TelemetryRecord::Span(span) if options.include_spans => {
                let (record, truncated) = export_span(
                    index as u64,
                    span.clone(),
                    options.max_attribute_value_bytes,
                );
                summary.span_records += 1;
                summary.truncated_attributes += truncated;
                summary.exported_records += 1;
                exported.push(record);
            }
            TelemetryRecord::Metric(metric) if options.include_metrics => {
                let (record, truncated) = export_metric(
                    index as u64,
                    metric.clone(),
                    options.max_attribute_value_bytes,
                );
                summary.metric_records += 1;
                summary.truncated_attributes += truncated;
                summary.exported_records += 1;
                exported.push(record);
            }
            _ => {
                summary.filtered_records += 1;
            }
        }
    }

    Ok(TelemetryExportBundle {
        schema_version: TELEMETRY_EXPORT_SCHEMA_VERSION,
        generated_at,
        records: exported,
        summary,
    })
}

fn export_span(
    index: u64,
    span: TelemetrySpan,
    max_attribute_value_bytes: usize,
) -> (TelemetryExportRecord, usize) {
    let span = span.redacted();
    let (attributes, diagnostics, truncated) =
        bound_attributes(span.attributes, max_attribute_value_bytes);

    (
        TelemetryExportRecord {
            index,
            record_type: TelemetryExportRecordType::Span,
            name: span.name,
            context: Some(span.context),
            unit: None,
            value: None,
            attributes,
            diagnostics,
        },
        truncated,
    )
}

fn export_metric(
    index: u64,
    metric: TelemetryMetric,
    max_attribute_value_bytes: usize,
) -> (TelemetryExportRecord, usize) {
    let metric = metric.redacted();
    let (attributes, diagnostics, truncated) =
        bound_attributes(metric.attributes, max_attribute_value_bytes);

    (
        TelemetryExportRecord {
            index,
            record_type: TelemetryExportRecordType::Metric,
            name: metric.name,
            context: None,
            unit: Some(metric.unit),
            value: Some(metric.value),
            attributes,
            diagnostics,
        },
        truncated,
    )
}

fn bound_attributes(
    attributes: Vec<TelemetryAttribute>,
    max_attribute_value_bytes: usize,
) -> (Vec<TelemetryAttribute>, Vec<String>, usize) {
    let mut diagnostics = Vec::new();
    let mut truncated = 0;
    let attributes = attributes
        .into_iter()
        .map(|attribute| {
            let (value, was_truncated) =
                bound_attribute_value(&attribute.value, max_attribute_value_bytes);
            if was_truncated {
                truncated += 1;
                diagnostics.push(format!(
                    "attribute {} truncated to {} bytes",
                    attribute.key, max_attribute_value_bytes
                ));
            }
            TelemetryAttribute {
                key: attribute.key,
                value,
            }
        })
        .collect();

    (attributes, diagnostics, truncated)
}

fn bound_attribute_value(value: &str, max_attribute_value_bytes: usize) -> (String, bool) {
    if value == REDACTED_VALUE || value.len() <= max_attribute_value_bytes {
        return (value.to_string(), false);
    }

    let mut bounded = String::new();
    for character in value.chars() {
        if bounded.len() + character.len_utf8() > max_attribute_value_bytes {
            break;
        }
        bounded.push(character);
    }

    (bounded, true)
}
