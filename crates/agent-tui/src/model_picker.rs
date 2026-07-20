use agent_ui_bridge::UiJsonFrame;

use crate::TuiEffect;

/// A single selectable model provider row in the picker. The catalog tracks one
/// `default_model` per provider, so each row switches to that provider and its
/// default model in one step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelPickerEntry {
    pub provider_id: String,
    pub model_id: String,
    pub kind: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelPickerAction {
    /// Open the picker (via `/model` or its shortcut), focusing the currently
    /// selected provider so a reflexive Enter is a no-op re-select rather than a
    /// surprise switch.
    Open,
    /// Move the highlight down one row.
    FocusNext,
    /// Move the highlight up one row.
    FocusPrevious,
    /// Confirm the highlighted provider/model (Enter).
    Confirm,
    /// Close the picker without switching (Esc).
    Dismiss,
}

/// State for the model-switch modal. Mirrors the approval modal: a centered
/// overlay with Up/Down selection, Enter to confirm, Esc to dismiss. Candidates
/// are populated from the `ConfigSnapshot` frame (the same provider list the
/// status panel uses).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModelPickerState {
    pub entries: Vec<ModelPickerEntry>,
    pub selected_provider: String,
    pub selected_model: String,
    pub focused_index: Option<usize>,
    pub open: bool,
}

impl ModelPickerState {
    pub fn reduce(&mut self, action: ModelPickerAction) -> TuiEffect {
        match action {
            ModelPickerAction::Open => self.open(),
            ModelPickerAction::FocusNext => {
                self.focus_next();
                TuiEffect::None
            }
            ModelPickerAction::FocusPrevious => {
                self.focus_previous();
                TuiEffect::None
            }
            ModelPickerAction::Confirm => self.confirm(),
            ModelPickerAction::Dismiss => {
                self.open = false;
                TuiEffect::None
            }
        }
    }

    /// Keep the candidate list and the currently-selected provider/model in sync
    /// with config frames. This mirrors how the status panel tracks selection, so
    /// the picker always opens focused on the live selection.
    pub fn reduce_frame(&mut self, frame: &UiJsonFrame) {
        match frame {
            UiJsonFrame::ConfigSnapshot(snapshot) => {
                self.entries = snapshot
                    .providers
                    .iter()
                    .map(|provider| ModelPickerEntry {
                        provider_id: provider.id.clone(),
                        model_id: provider.default_model.clone(),
                        kind: provider.kind.clone(),
                        enabled: provider.enabled,
                    })
                    .collect();
                self.selected_provider = snapshot.selected_provider.clone();
                self.selected_model = snapshot.selected_model.clone();
                self.clamp_focus();
            }
            UiJsonFrame::ConfigSummary {
                selected_provider,
                selected_model,
                ..
            } => {
                self.selected_provider = selected_provider.clone();
                self.selected_model = selected_model.clone();
            }
            UiJsonFrame::ModelSelection { provider, model } => {
                self.selected_provider = provider.clone();
                self.selected_model = model.clone();
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    pub fn focused_entry(&self) -> Option<&ModelPickerEntry> {
        self.focused_index.and_then(|index| self.entries.get(index))
    }

    fn open(&mut self) -> TuiEffect {
        if self.entries.is_empty() {
            return TuiEffect::note("no model providers are configured");
        }
        // Focus the live selection so Enter without moving is a harmless
        // re-select, never an accidental switch to a different provider.
        self.focused_index = self
            .entries
            .iter()
            .position(|entry| entry.provider_id == self.selected_provider)
            .or(Some(0));
        self.open = true;
        TuiEffect::None
    }

    fn confirm(&mut self) -> TuiEffect {
        let Some(entry) = self.focused_entry().cloned() else {
            self.open = false;
            return TuiEffect::note("no model is focused");
        };
        self.open = false;
        if !entry.enabled {
            return TuiEffect::warning(format!(
                "provider {} is disabled; enable it in config first",
                entry.provider_id
            ));
        }
        TuiEffect::SwitchModel {
            provider_id: entry.provider_id,
            model_id: entry.model_id,
        }
    }

    fn focus_next(&mut self) {
        if self.entries.is_empty() {
            self.focused_index = None;
            return;
        }
        self.focused_index = Some(match self.focused_index {
            Some(index) => (index + 1) % self.entries.len(),
            None => 0,
        });
    }

    fn focus_previous(&mut self) {
        if self.entries.is_empty() {
            self.focused_index = None;
            return;
        }
        self.focused_index = Some(match self.focused_index {
            Some(0) | None => self.entries.len() - 1,
            Some(index) => index - 1,
        });
    }

    fn clamp_focus(&mut self) {
        if self.entries.is_empty() {
            self.focused_index = None;
            return;
        }
        if let Some(index) = self.focused_index {
            self.focused_index = Some(index.min(self.entries.len() - 1));
        }
    }
}
