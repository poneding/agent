#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod error;
pub mod event;

pub use error::UiBridgeError;
pub use event::{UiDiagnosticLevel, UiEventEnvelope, UiEventPayload, normalize_runtime_event};
