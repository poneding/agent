//! Shared transcript entry kinds for plain REPL and TUI.
//!
//! Desktop keeps its own TS kinds; Rust surfaces should converge on this enum
//! so plain REPL is clearly a *subset* of the TUI model rather than a third
//! independent taxonomy.

use serde::{Deserialize, Serialize};

/// Chat transcript row kind. TUI may render additional metadata; plain REPL
/// only emits the first four variants.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTranscriptKind {
    User,
    Assistant,
    System,
    Error,
    Reasoning,
    ToolCall,
    ToolResult,
    Approval,
    Diagnostic,
    Progress,
}

impl UiTranscriptKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Error => "error",
            Self::Reasoning => "reasoning",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::Approval => "approval",
            Self::Diagnostic => "diagnostic",
            Self::Progress => "progress",
        }
    }

    /// Kinds that plain/model-only REPL is allowed to emit.
    #[must_use]
    pub const fn plain_repl_subset(self) -> bool {
        matches!(
            self,
            Self::User | Self::Assistant | Self::System | Self::Error
        )
    }
}
