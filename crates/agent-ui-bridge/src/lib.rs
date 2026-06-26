#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod command;
pub mod error;
pub mod event;
pub mod frame;

pub use command::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiPermissionDecision, validate_ui_command,
};
pub use error::UiBridgeError;
pub use event::{UiDiagnosticLevel, UiEventEnvelope, UiEventPayload, normalize_runtime_event};
pub use frame::{
    UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot, UiJsonFrame,
    UiModelProviderSummary, encode_json_frame,
};
