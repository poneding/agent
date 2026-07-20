use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::bridge::DesktopRuntimeSession;
use ruse_core::TraceEntry;
use ruse_desktop::{DesktopConfigSnapshot, DesktopSettings};
use ruse_ui_bridge::{UiConfigSnapshot, UiModelProviderCapabilities, UiModelProviderSummary};

#[derive(Clone, Debug)]
pub struct DesktopAppState {
    pub app_name: String,
    pub app_version: String,
    pub runtime_session: Arc<Mutex<DesktopRuntimeSession>>,
    pub trace_entries: Arc<Mutex<BTreeMap<String, Vec<TraceEntry>>>>,
    pub config_snapshot: Arc<Mutex<DesktopConfigSnapshot>>,
    pub settings: Arc<Mutex<DesktopSettings>>,
}

impl DesktopAppState {
    #[must_use]
    pub fn new(app_name: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
            runtime_session: Arc::new(Mutex::new(DesktopRuntimeSession::default())),
            trace_entries: Arc::new(Mutex::new(BTreeMap::new())),
            config_snapshot: Arc::new(Mutex::new(default_config_snapshot())),
            settings: Arc::new(Mutex::new(DesktopSettings::default())),
        }
    }
}

impl Default for DesktopAppState {
    fn default() -> Self {
        Self::new("agent", env!("CARGO_PKG_VERSION"))
    }
}

fn default_config_snapshot() -> DesktopConfigSnapshot {
    let capabilities =
        UiModelProviderCapabilities::new(true, false, false, false, Some(4096), Some(1024), None);
    let provider = UiModelProviderSummary::new("fake", "fake", "fake-scripted", true, capabilities)
        .expect("default provider summary is valid");
    let snapshot = UiConfigSnapshot::new("fake", "fake-scripted", vec![provider], Vec::new())
        .expect("default config snapshot is valid");
    DesktopConfigSnapshot::new(snapshot).expect("default desktop config snapshot is valid")
}
