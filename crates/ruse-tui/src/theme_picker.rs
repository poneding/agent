use crate::{TerminalTheme, TuiEffect};

/// Actions for the theme-switch modal. Up/Down move the highlight and immediately
/// preview the focused theme; Enter confirms and persists; Esc restores the theme
/// that was active when the picker opened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemePickerAction {
    Open,
    FocusNext,
    FocusPrevious,
    Confirm,
    Dismiss,
}

/// Centered theme picker. Mirrors the model/session modals, but focuses a fixed
/// catalog of [`TerminalTheme`] values rather than a runtime-fed list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemePickerState {
    pub open: bool,
    pub focused_index: usize,
    /// Theme active when the picker opened; restored on Esc so a cancelled
    /// preview never sticks.
    pub original_theme: TerminalTheme,
}

impl Default for ThemePickerState {
    fn default() -> Self {
        Self {
            open: false,
            focused_index: 0,
            original_theme: TerminalTheme::default(),
        }
    }
}

impl ThemePickerState {
    /// Fixed catalog order shown in the modal. Kept as a const so render/tests
    /// share one source of truth with focus wrapping.
    pub const THEMES: [TerminalTheme; 4] = [
        TerminalTheme::System,
        TerminalTheme::Light,
        TerminalTheme::Dark,
        TerminalTheme::HighContrast,
    ];

    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    pub fn focused_theme(&self) -> TerminalTheme {
        Self::THEMES[self.focused_index.min(Self::THEMES.len() - 1)]
    }

    pub fn reduce(&mut self, action: ThemePickerAction, current_theme: TerminalTheme) -> TuiEffect {
        match action {
            ThemePickerAction::Open => self.open(current_theme),
            ThemePickerAction::FocusNext => {
                self.focus_delta(1);
                TuiEffect::None
            }
            ThemePickerAction::FocusPrevious => {
                self.focus_delta(-1);
                TuiEffect::None
            }
            ThemePickerAction::Confirm => {
                self.open = false;
                TuiEffect::PersistTerminalTheme {
                    theme: self.focused_theme().name().to_string(),
                }
            }
            ThemePickerAction::Dismiss => {
                self.open = false;
                // Caller restores `original_theme` onto the app; this effect is a
                // no-op note-free close so Esc never writes a stale theme to disk.
                TuiEffect::None
            }
        }
    }

    fn open(&mut self, current_theme: TerminalTheme) -> TuiEffect {
        self.original_theme = current_theme;
        self.focused_index = Self::THEMES
            .iter()
            .position(|theme| *theme == current_theme)
            .unwrap_or(0);
        self.open = true;
        TuiEffect::None
    }

    fn focus_delta(&mut self, delta: isize) {
        let len = Self::THEMES.len() as isize;
        let next = (self.focused_index as isize + delta).rem_euclid(len);
        self.focused_index = next as usize;
    }
}
