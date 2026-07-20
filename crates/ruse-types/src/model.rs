use serde::{Deserialize, Serialize};

/// Provider-neutral reasoning depth requested for a model turn.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
    #[default]
    XHigh,
}

impl ReasoningEffort {
    pub const ALL: [Self; 5] = [
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::XHigh,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Minimal => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::XHigh,
            Self::XHigh => Self::Minimal,
        }
    }
}
