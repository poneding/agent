use agent_ui_bridge::{UiExtensionSnapshot, UiJsonFrame};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtensionPanelState {
    pub snapshot: Option<UiExtensionSnapshot>,
}

impl ExtensionPanelState {
    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        if let UiJsonFrame::ExtensionSnapshot(snapshot) = frame {
            self.snapshot = Some(snapshot.clone());
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let Some(snapshot) = &self.snapshot else {
            return "No extensions".to_string();
        };
        if snapshot.extensions.is_empty() {
            return "No extensions".to_string();
        }

        let mut lines = vec![format!(
            "Extensions total={} reviews={}",
            snapshot.extensions.len(),
            snapshot.capability_reviews.len()
        )];
        for extension in &snapshot.extensions {
            lines.push(format!(
                "{} {} enabled={} capabilities={}",
                extension.id, extension.health, extension.enabled, extension.capability_count
            ));
        }
        for diagnostic in &snapshot.diagnostics {
            lines.push(format!(
                "{} {}: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            ));
        }
        lines.push(format!("reviews={}", snapshot.capability_reviews.len()));
        lines.join("\n")
    }
}
