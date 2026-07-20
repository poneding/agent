use serde::{Deserialize, Serialize};

use crate::TelemetryError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    pub fn new(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        parent_span_id: Option<&str>,
    ) -> Result<Self, TelemetryError> {
        let trace_id = trace_id.into();
        let span_id = span_id.into();
        validate_required("trace_id", &trace_id)?;
        validate_required("span_id", &span_id)?;
        if let Some(parent_span_id) = parent_span_id {
            validate_required("parent_span_id", parent_span_id)?;
        }

        Ok(Self {
            trace_id,
            span_id,
            parent_span_id: parent_span_id.map(ToOwned::to_owned),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelemetryAttribute {
    pub key: String,
    pub value: String,
}

impl TelemetryAttribute {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, TelemetryError> {
        let key = key.into();
        validate_required("attribute.key", &key)?;
        Ok(Self {
            key,
            value: value.into(),
        })
    }

    pub(crate) fn redacted(mut self) -> Self {
        if is_secret_like_key(&self.key) || is_secret_like_value(&self.value) {
            self.value = "[redacted]".to_string();
        }
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySpan {
    pub name: String,
    pub context: TraceContext,
    pub attributes: Vec<TelemetryAttribute>,
}

impl TelemetrySpan {
    pub fn new(
        name: impl Into<String>,
        context: TraceContext,
        attributes: Vec<TelemetryAttribute>,
    ) -> Result<Self, TelemetryError> {
        let name = name.into();
        validate_required("span.name", &name)?;
        Ok(Self {
            name,
            context,
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

pub(crate) fn validate_required(field: &'static str, value: &str) -> Result<(), TelemetryError> {
    if value.trim().is_empty() {
        return Err(TelemetryError::EmptyField { field });
    }
    Ok(())
}

fn is_secret_like_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["secret", "token", "password", "key"]
        .iter()
        .any(|needle| key.contains(needle))
}

fn is_secret_like_value(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains(" sk-")
        || value.contains("bearer sk-")
        || value.contains("secret")
        || value.contains("password")
        || value.contains("token")
        || value.contains("begin private key")
}
