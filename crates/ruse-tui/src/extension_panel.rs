use ruse_ui_bridge::{UiExtensionSnapshot, UiJsonFrame};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtensionPanelState {
    pub snapshot: Option<UiExtensionSnapshot>,
}

impl ExtensionPanelState {
    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        if let UiJsonFrame::Admin(admin) = frame {
            if admin.kind == "extension_snapshot" {
                if let Ok(snapshot) = admin.decode_as::<UiExtensionSnapshot>() {
                    self.snapshot = Some(snapshot);
                }
            }
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
        lines.join("\n")
    }
}
