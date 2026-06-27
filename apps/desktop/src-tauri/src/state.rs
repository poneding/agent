use std::sync::{Arc, Mutex};

use crate::bridge::DesktopRuntimeSession;

#[derive(Clone, Debug)]
pub struct DesktopAppState {
    pub app_name: String,
    pub app_version: String,
    pub runtime_session: Arc<Mutex<DesktopRuntimeSession>>,
}

impl DesktopAppState {
    #[must_use]
    pub fn new(app_name: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
            runtime_session: Arc::new(Mutex::new(DesktopRuntimeSession::default())),
        }
    }
}

impl Default for DesktopAppState {
    fn default() -> Self {
        Self::new("agent", env!("CARGO_PKG_VERSION"))
    }
}
