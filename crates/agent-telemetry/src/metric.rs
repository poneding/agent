use serde::{Deserialize, Serialize};

use crate::{TelemetryAttribute, TelemetryError, span::validate_required};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryMetric {
    pub name: String,
    pub unit: String,
    pub value: f64,
    pub attributes: Vec<TelemetryAttribute>,
}

impl TelemetryMetric {
    pub fn new(
        name: impl Into<String>,
        unit: impl Into<String>,
        value: f64,
        attributes: Vec<TelemetryAttribute>,
    ) -> Result<Self, TelemetryError> {
        let name = name.into();
        let unit = unit.into();
        validate_required("metric.name", &name)?;
        validate_required("metric.unit", &unit)?;
        if !value.is_finite() {
            return Err(TelemetryError::InvalidMetricValue { name });
        }
        Ok(Self {
            name,
            unit,
            value,
            attributes,
        })
    }

    pub(crate) fn redacted(mut self) -> Self {
        self.attributes = self
            .attributes
            .into_iter()
            .map(TelemetryAttribute::redacted)
            .collect();
        self
    }
}
