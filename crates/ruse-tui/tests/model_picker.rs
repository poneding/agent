use ruse_tui::{ModelPickerAction, ModelPickerState, TuiEffect};
use ruse_ui_bridge::{
    UiConfigSnapshot, UiJsonFrame, UiModelProviderCapabilities, UiModelProviderSummary,
};

fn provider(id: &str, default_model: &str, enabled: bool) -> UiModelProviderSummary {
    UiModelProviderSummary::new(
        id,
        "openai_compatible",
        default_model,
        enabled,
        UiModelProviderCapabilities::new(
            true,
            true,
            true,
            false,
            Some(128_000),
            Some(16_384),
            None,
        ),
    )
    .expect("provider summary")
}

fn config_snapshot_frame() -> UiJsonFrame {
    UiJsonFrame::ConfigSnapshot(
        UiConfigSnapshot::new(
            "primary",
            "gpt-5.5",
            vec![
                provider("primary", "gpt-5.5", true),
                provider("backup", "gpt-5.5-mini", true),
                provider("legacy", "gpt-4.1", false),
            ],
            Vec::new(),
        )
        .expect("config snapshot"),
    )
}

#[test]
fn model_picker_reduces_snapshot_and_tracks_selection() {
    let mut picker = ModelPickerState::default();

    picker.reduce_frame(&config_snapshot_frame());

    // The catalog is sorted by id when the snapshot is built (backup, legacy,
    // primary), and the live selection is tracked so opening focuses it.
    assert_eq!(picker.entries.len(), 3);
    assert_eq!(picker.selected_provider, "primary");
    assert_eq!(picker.selected_model, "gpt-5.5");
    assert!(!picker.is_open());
}

#[test]
fn model_picker_opens_focused_on_live_selection() {
    let mut picker = ModelPickerState::default();
    picker.reduce_frame(&config_snapshot_frame());

    assert_eq!(picker.reduce(ModelPickerAction::Open), TuiEffect::None);
    assert!(picker.is_open());
    // Focus lands on the currently-selected provider, not the first row, so a
    // reflexive Enter re-selects the active model rather than switching.
    assert_eq!(
        picker.focused_entry().map(|e| e.provider_id.as_str()),
        Some("primary")
    );
}

#[test]
fn model_picker_moves_focus_with_wraparound() {
    let mut picker = ModelPickerState::default();
    picker.reduce_frame(&config_snapshot_frame());
    picker.reduce(ModelPickerAction::Open);

    // Providers are sorted: backup(0), legacy(1), primary(2). Open focuses primary(2).
    assert_eq!(picker.focused_index, Some(2));
    assert_eq!(picker.reduce(ModelPickerAction::FocusNext), TuiEffect::None);
    assert_eq!(picker.focused_index, Some(0));
    assert_eq!(
        picker.reduce(ModelPickerAction::FocusPrevious),
        TuiEffect::None
    );
    assert_eq!(picker.focused_index, Some(2));
}

#[test]
fn model_picker_confirm_emits_switch_effect_and_closes() {
    let mut picker = ModelPickerState::default();
    picker.reduce_frame(&config_snapshot_frame());
    picker.reduce(ModelPickerAction::Open);
    // Move to backup (index 0).
    picker.reduce(ModelPickerAction::FocusNext);

    let effect = picker.reduce(ModelPickerAction::Confirm);

    assert_eq!(
        effect,
        TuiEffect::SwitchModel {
            provider_id: "backup".to_string(),
            model_id: "gpt-5.5-mini".to_string(),
        }
    );
    assert!(!picker.is_open());
}

#[test]
fn model_picker_confirm_on_disabled_provider_warns_without_switching() {
    let mut picker = ModelPickerState::default();
    picker.reduce_frame(&config_snapshot_frame());
    picker.reduce(ModelPickerAction::Open);
    // Move to legacy (index 1), which is disabled.
    picker.reduce(ModelPickerAction::FocusNext);
    picker.reduce(ModelPickerAction::FocusNext);

    let effect = picker.reduce(ModelPickerAction::Confirm);

    assert!(matches!(effect, TuiEffect::LocalDiagnostic { .. }));
    assert!(!picker.is_open());
}

#[test]
fn model_picker_dismiss_closes_without_switching() {
    let mut picker = ModelPickerState::default();
    picker.reduce_frame(&config_snapshot_frame());
    picker.reduce(ModelPickerAction::Open);

    assert_eq!(picker.reduce(ModelPickerAction::Dismiss), TuiEffect::None);
    assert!(!picker.is_open());
}

#[test]
fn model_picker_open_with_empty_catalog_notes_and_stays_closed() {
    let mut picker = ModelPickerState::default();

    let effect = picker.reduce(ModelPickerAction::Open);

    assert!(matches!(effect, TuiEffect::LocalDiagnostic { .. }));
    assert!(!picker.is_open());
}
