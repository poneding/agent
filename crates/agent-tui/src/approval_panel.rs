use std::str::FromStr;

use agent_types::ApprovalRequestId;
use agent_ui_bridge::{
    UiApprovalDecision, UiApprovalPreviewSummary, UiApprovalQueueGroup, UiApprovalQueueItem,
    UiApprovalQueueSnapshot, UiApprovalRequest, UiEventPayload, UiJsonFrame,
    approval_queue_snapshot_from_requests,
};

use crate::TuiEffect;

const TUI_ALLOW_REASON: &str = "approved from TUI";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApprovalPanelState {
    pub pending: Vec<UiApprovalRequest>,
    pub groups: Vec<UiApprovalQueueGroup>,
    pub expired_count: u32,
    pub diagnostic_count: u32,
    pub focused_index: Option<usize>,
}

impl ApprovalPanelState {
    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::ApprovalQueueSnapshot(snapshot) => self.apply_snapshot(snapshot),
            UiJsonFrame::Event(event) => match &event.payload {
                UiEventPayload::ApprovalRequested(request) => {
                    self.upsert_pending((**request).clone());
                }
                UiEventPayload::ApprovalResolved { request_id, .. } => {
                    self.remove_pending(request_id)
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn reduce(&mut self, action: ApprovalPanelAction) -> TuiEffect {
        match action {
            ApprovalPanelAction::FocusNext => {
                self.focus_next();
                TuiEffect::None
            }
            ApprovalPanelAction::FocusPrevious => {
                self.focus_previous();
                TuiEffect::None
            }
            ApprovalPanelAction::Allow => self.resolve_focused(
                UiApprovalDecision::Allow,
                TUI_ALLOW_REASON.to_string(),
                self.focused_request()
                    .and_then(|request| request.max_duration_seconds),
            ),
            ApprovalPanelAction::Deny { reason } => {
                let reason = reason.trim();
                if reason.is_empty() {
                    TuiEffect::LocalDiagnostic("deny requires a reason".to_string())
                } else {
                    self.resolve_focused(UiApprovalDecision::Deny, reason.to_string(), None)
                }
            }
            ApprovalPanelAction::Abort => match self.focused_request() {
                Some(request) => TuiEffect::ApprovalAborted(request.request_id.clone()),
                None => TuiEffect::LocalDiagnostic("no approval is focused".to_string()),
            },
        }
    }

    #[must_use]
    pub fn focused_request(&self) -> Option<&UiApprovalRequest> {
        self.focused_index.and_then(|index| self.pending.get(index))
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.pending.is_empty() || self.expired_count > 0 || self.diagnostic_count > 0
    }

    #[must_use]
    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();
        if self.pending.is_empty() {
            lines.push("Approval queue: empty".to_string());
        } else {
            lines.push(format!("Approval queue: {} pending", self.pending.len()));
        }
        if self.expired_count > 0 {
            lines.push(format!("Expired: {}", self.expired_count));
        }
        for group in &self.groups {
            lines.push(format!(
                "group {}/{}/{} count={}",
                group.resource_kind, group.action, group.status, group.count
            ));
        }

        let Some(request) = self.focused_request() else {
            return lines.join("\n");
        };

        let focus = self.focused_index.unwrap_or(0) + 1;
        lines.push(format!("focus {focus}/{}", self.pending.len()));
        lines.push(format!("{} {}", request.action, request.resource_kind));
        lines.push(request.resource.clone());
        lines.push(request.reason.clone());
        lines.push(request.preview.summary.clone());
        if let Some(max_duration_seconds) = request.max_duration_seconds {
            lines.push(format!("lease={max_duration_seconds}s"));
        }
        if let Some(expires_at) = request.expires_at {
            lines.push(format!("expires_at={expires_at:?}"));
        }
        if let Some(content_hash) = &request.preview.content_hash {
            lines.push(content_hash.clone());
        }
        if request.preview.truncated {
            lines.push("truncated".to_string());
        }
        if request.preview.redacted {
            lines.push("redacted".to_string());
        }
        lines.join("\n")
    }

    fn upsert_pending(&mut self, request: UiApprovalRequest) {
        let focused_request_id = self
            .focused_request()
            .map(|request| request.request_id.clone());
        if let Some(existing) = self
            .pending
            .iter_mut()
            .find(|existing| existing.request_id == request.request_id)
        {
            *existing = request;
        } else {
            self.pending.push(request);
        }

        if self.focused_index.is_none() {
            self.focused_index = Some(0);
        }
        self.refresh_snapshot(focused_request_id);
        self.clamp_focus();
    }

    fn remove_pending(&mut self, request_id: &str) {
        let focused_request_id = self
            .focused_request()
            .map(|request| request.request_id.clone());
        self.pending
            .retain(|request| request.request_id != request_id);
        self.refresh_snapshot(focused_request_id);
        self.clamp_focus();
    }

    fn refresh_snapshot(&mut self, focused_request_id: Option<String>) {
        if let Ok(snapshot) = approval_queue_snapshot_from_requests(self.pending.clone()) {
            self.apply_snapshot_with_focus(&snapshot, focused_request_id);
        }
    }

    fn apply_snapshot(&mut self, snapshot: &UiApprovalQueueSnapshot) {
        let focused_request_id = self
            .focused_request()
            .map(|request| request.request_id.clone());
        self.apply_snapshot_with_focus(snapshot, focused_request_id);
    }

    fn apply_snapshot_with_focus(
        &mut self,
        snapshot: &UiApprovalQueueSnapshot,
        focused_request_id: Option<String>,
    ) {
        self.pending = snapshot
            .pending
            .iter()
            .map(approval_request_from_queue_item)
            .collect();
        self.groups = snapshot.groups.clone();
        self.expired_count = snapshot.expired_count;
        self.diagnostic_count = snapshot.diagnostic_count;

        self.focused_index = focused_request_id
            .and_then(|request_id| {
                self.pending
                    .iter()
                    .position(|request| request.request_id == request_id)
            })
            .or_else(|| (!self.pending.is_empty()).then_some(0));
        self.clamp_focus();
    }

    fn focus_next(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        self.focused_index = Some((current + 1).min(self.pending.len() - 1));
    }

    fn focus_previous(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        self.focused_index = Some(current.saturating_sub(1));
    }

    fn resolve_focused(
        &self,
        decision: UiApprovalDecision,
        reason: String,
        lease_duration_seconds: Option<u64>,
    ) -> TuiEffect {
        let Some(request) = self.focused_request() else {
            return TuiEffect::LocalDiagnostic("no approval is focused".to_string());
        };
        let Ok(request_id) = ApprovalRequestId::from_str(&request.request_id) else {
            return TuiEffect::LocalDiagnostic(format!(
                "invalid approval request id: {}",
                request.request_id
            ));
        };

        TuiEffect::ResolveApproval {
            request_id,
            decision,
            reason,
            lease_duration_seconds,
        }
    }

    fn clamp_focus(&mut self) {
        if self.pending.is_empty() {
            self.focused_index = None;
            return;
        }

        let index = self.focused_index.unwrap_or(0).min(self.pending.len() - 1);
        self.focused_index = Some(index);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalPanelAction {
    FocusNext,
    FocusPrevious,
    Allow,
    Deny { reason: String },
    Abort,
}

fn approval_request_from_queue_item(item: &UiApprovalQueueItem) -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: item.request_id.clone(),
        status: item.status.clone(),
        action: item.action.clone(),
        resource_kind: item.resource_kind.clone(),
        resource: item.resource.clone(),
        reason: item.reason.clone(),
        preview: UiApprovalPreviewSummary {
            kind: item.preview.kind.clone(),
            resource: item.preview.resource.clone(),
            summary: item.preview.summary.clone(),
            content_hash: item.preview.content_hash.clone(),
            truncated: item.preview.truncated,
            redacted: item.preview.redacted,
        },
        max_duration_seconds: item.max_duration_seconds,
        expires_at: item.expires_at,
    }
}
