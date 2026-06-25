#![forbid(unsafe_code)]
//! Desktop presentation contracts for agent.

pub mod error;
pub mod session;

pub use error::DesktopError;
pub use session::{
    DesktopRoute, DesktopSession, DesktopSessionState, disconnect_desktop_session,
    start_desktop_session, switch_desktop_route,
};
