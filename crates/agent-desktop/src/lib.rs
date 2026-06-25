#![forbid(unsafe_code)]
//! Desktop presentation contracts for agent.

pub mod conversation;
pub mod error;
pub mod session;
pub mod snapshot;
pub mod task;

pub use conversation::DesktopConversationItem;
pub use error::DesktopError;
pub use session::{
    DesktopRoute, DesktopSession, DesktopSessionState, disconnect_desktop_session,
    start_desktop_session, switch_desktop_route,
};
pub use snapshot::{DesktopWorkspaceSnapshot, build_workspace_snapshot};
pub use task::DesktopTaskItem;
