use ruse_ui_bridge::{UiApprovalRequest, UiEditDiffSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffPanelState {
    pub path: String,
    pub applied: bool,
    pub diff_state: String,
    pub changed_files: Vec<String>,
    pub changed_file_count: u32,
    pub diff_line_count: u32,
    pub added_line_count: u32,
    pub removed_line_count: u32,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub focused_file_index: Option<usize>,
    pub scroll_line: usize,
    pub diagnostic_count: u32,
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

    pub fn reduce(&mut self, action: DiffPanelAction) {
        match action {
            DiffPanelAction::ToggleFold => {}
            DiffPanelAction::NextFile => self.focus_next_file(),
            DiffPanelAction::PreviousFile => self.focus_previous_file(),
            DiffPanelAction::ScrollDown => {
                let max_scroll = self.summary.lines().count().saturating_sub(1);
                self.scroll_line = (self.scroll_line + 1).min(max_scroll);
            }
            DiffPanelAction::ScrollUp => {
                self.scroll_line = self.scroll_line.saturating_sub(1);
            }
        }
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
            path: request.resource.clone(),
            applied: false,
            diff_state: "approval_preview".to_string(),
            changed_files: vec![request.resource.clone()],
            changed_file_count: 1,
            diff_line_count: 0,
            added_line_count: 0,
            removed_line_count: 0,
            resource: request.preview.resource.clone(),
            summary,
            content_hash: request.preview.content_hash.clone(),
            focused_file_index: Some(0),
            scroll_line: 0,
            diagnostic_count: 0,
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
            path: snapshot.path.clone(),
            applied: snapshot.applied,
            diff_state: snapshot.diff_state.clone(),
            changed_files: snapshot.changed_files.clone(),
            changed_file_count: snapshot.changed_file_count,
            diff_line_count: snapshot.diff_line_count,
            added_line_count: snapshot.added_line_count,
            removed_line_count: snapshot.removed_line_count,
            resource: snapshot.resource.clone(),
            summary,
            content_hash: snapshot.content_hash.clone(),
            focused_file_index: (!snapshot.changed_files.is_empty()).then_some(0),
            scroll_line: 0,
            diagnostic_count: snapshot.diagnostic_count,
            truncated: snapshot.truncated || over_limit,
            redacted: snapshot.redacted,
        }
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let applied = if self.applied { "yes" } else { "no" };
        let focused_file = self
            .focused_file_index
            .and_then(|index| self.changed_files.get(index))
            .cloned()
            .unwrap_or_else(|| "none".to_string());
        let visible_summary = self
            .summary
            .lines()
            .skip(self.scroll_line)
            .collect::<Vec<_>>()
            .join("\n");
        let mut text = format!(
            "Diff: {} applied={applied}\nPath: {}\nFiles: {}/{} focused={}\nLines: +{} -{} total={}\n{}",
            self.diff_state,
            self.path,
            self.changed_files.len(),
            self.changed_file_count,
            focused_file,
            self.added_line_count,
            self.removed_line_count,
            self.diff_line_count,
            visible_summary
        );
        if let Some(content_hash) = &self.content_hash {
            text.push_str(&format!("\n{content_hash}"));
        }
        if self.diagnostic_count > 0 {
            text.push_str(&format!("\ndiagnostics={}", self.diagnostic_count));
        }
        if self.truncated {
            text.push_str("\ntruncated");
        }
        if self.redacted {
            text.push_str("\nredacted");
        }
        text
    }

    fn focus_next_file(&mut self) {
        if self.changed_files.is_empty() {
            self.focused_file_index = None;
            return;
        }
        let current = self.focused_file_index.unwrap_or(0);
        self.focused_file_index = Some((current + 1).min(self.changed_files.len() - 1));
    }

    fn focus_previous_file(&mut self) {
        if self.changed_files.is_empty() {
            self.focused_file_index = None;
            return;
        }
        let current = self.focused_file_index.unwrap_or(0);
        self.focused_file_index = Some(current.saturating_sub(1));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiffPanelAction {
    ToggleFold,
    NextFile,
    PreviousFile,
    ScrollDown,
    ScrollUp,
}
