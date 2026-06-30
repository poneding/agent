use agent_ui_bridge::{UiApprovalRequest, UiEditDiffSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffPanelState {
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

impl DiffPanelState {
    #[must_use]
    pub fn maybe_from_approval(
        request: &UiApprovalRequest,
        max_summary_chars: usize,
    ) -> Option<Self> {
        if request.preview.kind != "file_edit" {
            return None;
        }

        Some(Self::from_approval(request, max_summary_chars))
    }

    #[must_use]
    pub fn from_approval(request: &UiApprovalRequest, max_summary_chars: usize) -> Self {
        let max_summary_chars = max_summary_chars.max(1);
        let mut summary = request.preview.summary.clone();
        let over_limit = summary.chars().count() > max_summary_chars;
        if over_limit {
            summary = summary.chars().take(max_summary_chars).collect();
        }

        Self {
            resource: request.preview.resource.clone(),
            summary,
            content_hash: request.preview.content_hash.clone(),
            truncated: request.preview.truncated || over_limit,
            redacted: request.preview.redacted,
        }
    }

    #[must_use]
    pub fn from_edit_diff_snapshot(
        snapshot: &UiEditDiffSnapshot,
        max_summary_chars: usize,
    ) -> Self {
        let max_summary_chars = max_summary_chars.max(1);
        let mut summary = if snapshot.diff_preview.is_empty() {
            snapshot.summary.clone()
        } else {
            snapshot.diff_preview.clone()
        };
        let over_limit = summary.chars().count() > max_summary_chars;
        if over_limit {
            summary = summary.chars().take(max_summary_chars).collect();
        }

        Self {
            resource: snapshot.resource.clone(),
            summary,
            content_hash: snapshot.content_hash.clone(),
            truncated: snapshot.truncated || over_limit,
            redacted: snapshot.redacted,
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mut text = format!("{}\n{}", self.resource, self.summary);
        if let Some(content_hash) = &self.content_hash {
            text.push_str(&format!("\n{content_hash}"));
        }
        if self.truncated {
            text.push_str("\ntruncated");
        }
        if self.redacted {
            text.push_str("\nredacted");
        }
        text
    }
}
