use serde::{Deserialize, Serialize};

use crate::{TelemetryMetric, TelemetrySpan};

pub trait TelemetrySink {
    fn record_span(&mut self, span: TelemetrySpan);
    fn record_metric(&mut self, metric: TelemetryMetric);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryRecord {
    Span(TelemetrySpan),
    Metric(TelemetryMetric),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InMemoryTelemetrySink {
    records: Vec<TelemetryRecord>,
    spans: Vec<TelemetrySpan>,
    metrics: Vec<TelemetryMetric>,
}

impl InMemoryTelemetrySink {
    #[must_use]
    pub fn records(&self) -> &[TelemetryRecord] {
        &self.records
    }

    #[must_use]
    pub fn spans(&self) -> &[TelemetrySpan] {
        &self.spans
    }

    #[must_use]
    pub fn metrics(&self) -> &[TelemetryMetric] {
        &self.metrics
    }
}

impl TelemetrySink for InMemoryTelemetrySink {
    fn record_span(&mut self, span: TelemetrySpan) {
        let span = span.redacted();
        self.spans.push(span.clone());
        self.records.push(TelemetryRecord::Span(span));
    }

    fn record_metric(&mut self, metric: TelemetryMetric) {
        let metric = metric.redacted();
        self.metrics.push(metric.clone());
        self.records.push(TelemetryRecord::Metric(metric));
    }
}
