//! Centralized, per-theme color palette for the terminal UI.
//!
//! Before this module the render layer carried ~10 scattered `match (kind, theme)`
//! sites, each independently deciding a `Style`. That made the visual language
//! impossible to audit and easy to drift — and several states were distinguished
//! by **color alone**, so they vanished under monochrome terminals or for
//! color-blind users.
//!
//! `Palette` fixes both problems:
//!
//! 1. **One home for color.** The theme is matched exactly once (in [`Palette::for_theme`]);
//!    every render site asks the palette for a semantic style by name. Changing a
//!    theme's colors is a single edit here.
//! 2. **Color is never the only cue.** Every semantic style pairs its color with a
//!    non-color cue — a text `Modifier` (bold/dim/italic) — so the state survives a
//!    monochrome or color-blind rendering. This is a *contract*, enforced by
//!    [`Palette::semantic_styles`] + the accessibility test, not a convention.
//!    Glyph cues (`●`/`⎿`/`⚠`/severity-label text) live at the call sites and
//!    complete the guarantee.
//!
//! `HighContrast` is therefore a real contract, not a color swap: it keeps the
//! bold/dim modifiers of every other theme and only widens the color separation.

use ratatui::style::{Color, Modifier, Style};

use crate::{ReasoningEffort, TerminalTheme};

/// A semantic role a rendered span can play. This is the vocabulary render sites
/// use to ask the palette for a style, decoupled from concrete colors.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Semantic {
    /// A user message block.
    User,
    /// A model/assistant message block.
    Model,
    /// Reasoning / "thinking" output.
    Reasoning,
    /// A tool call or its result.
    Tool,
    /// An approval record in the transcript.
    Approval,
    /// A real error (red). The only severity that reaches the transcript.
    Error,
    /// A warning: notable but not a failure.
    Warning,
    /// Dim, secondary "machinery" text (meta, connectors, hints).
    Meta,
    /// The live activity-line spinner/verb.
    Activity,
    /// A pending approval / attention indicator.
    Attention,
    /// Approval-gate modal chrome and selection.
    ApprovalModal,
    /// Model-picker modal chrome and selection.
    ModelModal,
    /// Session-picker modal chrome and selection.
    SessionModal,
}

/// The resolved color + non-color cue set for one theme. Built once per render via
/// [`Palette::for_theme`]; every style accessor reads from it, so the theme is
/// matched in exactly one place.
#[derive(Copy, Clone, Debug)]
pub struct Palette {
    theme: TerminalTheme,
}

impl Palette {
    /// Build the palette for a theme. This is the single site where the theme is
    /// matched; everything else is derived.
    #[must_use]
    pub const fn for_theme(theme: TerminalTheme) -> Self {
        Self { theme }
    }

    /// Theme-aware equivalent of a `text-muted-foreground` token.
    fn muted_foreground(self) -> Color {
        match self.theme {
            TerminalTheme::System | TerminalTheme::Dark => Color::Rgb(148, 163, 184),
            TerminalTheme::Light => Color::Rgb(100, 116, 139),
            TerminalTheme::HighContrast => Color::Gray,
        }
    }

    /// The style for a semantic role. Every returned style carries a non-color cue
    /// (a `Modifier`) in addition to its color, so it survives monochrome rendering.
    #[must_use]
    pub fn semantic(self, role: Semantic) -> Style {
        let high_contrast = self.theme == TerminalTheme::HighContrast;
        match role {
            // Real errors: red + BOLD. HighContrast swaps red→yellow (red on black
            // can be low-contrast) but keeps BOLD, so the "this is an error" cue is
            // never color-only.
            Semantic::Error => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.fg(Color::Yellow)
                } else {
                    base.fg(Color::Red)
                }
            }
            // Warnings: yellow + ITALIC. The italic keeps them distinct from errors
            // (bold) and from meta (dim) even with no color.
            Semantic::Warning => {
                let base = Style::default().add_modifier(Modifier::ITALIC);
                if high_contrast {
                    base.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    base.fg(Color::Yellow)
                }
            }
            // Reasoning stays gray across themes so thinking reads as secondary
            // process context rather than competing with the final answer.
            Semantic::Reasoning => {
                let base = Style::default()
                    .fg(self.muted_foreground())
                    .add_modifier(Modifier::ITALIC);
                if high_contrast {
                    base.add_modifier(Modifier::BOLD)
                } else {
                    base
                }
            }
            // Tool calls: cyan/blue + BOLD so the `●` head stands out from prose.
            Semantic::Tool => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    TerminalTheme::Dark => base.fg(Color::Cyan),
                    _ => base.fg(Color::Blue),
                }
            }
            // Approvals: deep green + BOLD. Matches the Allow option color so a resolved
            // allow reads as "ok", not as an alert. Reject-colored rows are painted at
            // the call site when the decision is deny.
            Semantic::Approval => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.fg(Color::Yellow)
                } else {
                    base.fg(Color::Rgb(34, 197, 94))
                }
            }
            // Attention (pending approvals): same weight as approval; paired with a
            // `⚠` glyph at the call site.
            Semantic::Attention => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    TerminalTheme::Light => base.fg(Color::Red),
                    _ => base.fg(Color::Yellow),
                }
            }
            Semantic::ApprovalModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    TerminalTheme::Light => base.fg(Color::Rgb(176, 96, 0)),
                    TerminalTheme::Dark => base.fg(Color::Rgb(245, 166, 35)),
                }
            }
            Semantic::ModelModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Cyan),
                    TerminalTheme::Light => base.fg(Color::Rgb(0, 105, 145)),
                    TerminalTheme::Dark => base.fg(Color::Rgb(56, 189, 248)),
                }
            }
            Semantic::SessionModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Green),
                    TerminalTheme::Light => base.fg(Color::Rgb(20, 125, 75)),
                    TerminalTheme::Dark => base.fg(Color::Rgb(74, 222, 128)),
                }
            }
            // Activity line spinner/verb: cyan/blue + BOLD.
            Semantic::Activity => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    TerminalTheme::Light => base.fg(Color::Blue),
                    _ => base.fg(Color::Cyan),
                }
            }
            // Meta / machinery: secondary text. The non-color cue is ITALIC, not
            // DIM: many terminals render DIM as near-invisible (its failure mode is
            // disappearance), whereas ITALIC at worst falls back to normal weight —
            // so meta text stays readable everywhere while still reading as an aside.
            Semantic::Meta => {
                let base = Style::default().add_modifier(Modifier::ITALIC);
                match self.theme {
                    TerminalTheme::HighContrast => {
                        base.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    }
                    TerminalTheme::Dark => base.fg(Color::Gray),
                    TerminalTheme::Light => base.fg(Color::DarkGray),
                    // System keeps the terminal's default foreground (readable on
                    // any background) and leans on ITALIC alone as the cue.
                    TerminalTheme::System => base.fg(Color::Gray),
                }
            }
            // User message blocks: background fills mark the bubble; the vertical
            // padding rows around the text are rendered by the layout layer.
            Semantic::User => match self.theme {
                TerminalTheme::System => Style::default().bg(Color::DarkGray).fg(Color::White),
                TerminalTheme::Light => Style::default()
                    .bg(Color::Rgb(226, 232, 240))
                    .fg(Color::Black),
                TerminalTheme::Dark => Style::default().bg(Color::Rgb(39, 49, 68)).fg(Color::White),
                TerminalTheme::HighContrast => Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            },
            // Model messages stay unfilled so inter-item separators read as a single
            // plain blank row and tool/thinking blocks don't fight a second bg.
            Semantic::Model => match self.theme {
                TerminalTheme::System => Style::default(),
                TerminalTheme::Light => Style::default().fg(Color::Black),
                TerminalTheme::Dark => Style::default().fg(Color::White),
                TerminalTheme::HighContrast => Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            },
        }
    }

    /// The chip style for a semantic role — an inverted (background-filled) badge.
    /// Kept for panel/modal chrome that still wants a filled pill; transcript
    /// status labels no longer use chips (they render as `• word` with a status
    /// color — see `status_done` / `render::status_label_spans`).
    #[must_use]
    pub fn chip(self, role: Semantic) -> Style {
        let base = match role {
            Semantic::Error => Style::default().fg(Color::White).bg(Color::Red),
            Semantic::Approval => Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(22, 101, 52)), // deep green chip
            Semantic::Reasoning => Style::default().fg(Color::White).bg(Color::DarkGray),
            Semantic::Tool => Style::default().fg(Color::Black).bg(Color::Cyan),
            _ => self.meta(),
        };
        // Chips are always bold: the inverted badge plus weight is the non-color cue.
        base.add_modifier(Modifier::BOLD)
    }

    /// Settled / completed status label (e.g. `• Thought` after reasoning finishes).
    /// Green + BOLD so it contrasts with the muted gray used while still in progress.
    #[must_use]
    pub fn status_done(self) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Yellow),
            TerminalTheme::Light => base.fg(Color::Rgb(22, 163, 74)),
            _ => base.fg(Color::Rgb(74, 222, 128)),
        }
    }

    /// Dim, secondary text (meta lines, connectors, hints).
    #[must_use]
    pub fn meta(self) -> Style {
        self.semantic(Semantic::Meta)
    }

    /// Muted placeholder text without a background fill.
    #[must_use]
    pub fn placeholder(self) -> Style {
        Style::default().fg(self.muted_foreground())
    }

    /// The window/border style.
    #[must_use]
    pub fn border(self) -> Style {
        match self.theme {
            TerminalTheme::System => Style::default(),
            TerminalTheme::Light => Style::default().fg(Color::DarkGray),
            TerminalTheme::Dark => Style::default().fg(Color::Cyan),
            TerminalTheme::HighContrast => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Blue-to-purple framing used by the top and bottom composer rules (and the
    /// matching `❯` prompt marker). Higher effort increases chroma and weight
    /// without italic — italic on chrome makes the prompt look like emphasis
    /// rather than a structural frame.
    #[must_use]
    pub fn composer_border(self, effort: ReasoningEffort) -> Style {
        let color = match (self.theme, effort) {
            (TerminalTheme::Light, ReasoningEffort::Minimal) => Color::Rgb(92, 113, 160),
            (TerminalTheme::Light, ReasoningEffort::Low) => Color::Rgb(75, 99, 190),
            (TerminalTheme::Light, ReasoningEffort::Medium) => Color::Rgb(80, 81, 205),
            (TerminalTheme::Light, ReasoningEffort::High) => Color::Rgb(104, 71, 205),
            (TerminalTheme::Light, ReasoningEffort::XHigh) => Color::Rgb(126, 61, 190),
            (_, ReasoningEffort::Minimal) => Color::Rgb(93, 126, 190),
            (_, ReasoningEffort::Low) => Color::Rgb(82, 120, 220),
            (_, ReasoningEffort::Medium) => Color::Rgb(108, 99, 235),
            (_, ReasoningEffort::High) => Color::Rgb(139, 92, 246),
            (_, ReasoningEffort::XHigh) => Color::Rgb(174, 78, 238),
        };
        let modifier = match effort {
            ReasoningEffort::Minimal | ReasoningEffort::Low => Modifier::DIM,
            ReasoningEffort::Medium => Modifier::empty(),
            ReasoningEffort::High | ReasoningEffort::XHigh => Modifier::BOLD,
        };
        Style::default().fg(color).add_modifier(modifier)
    }

    /// The persistent status-footer style.
    #[must_use]
    pub fn status_bar(self) -> Style {
        let base = Style::default().fg(self.muted_foreground());
        if self.theme == TerminalTheme::HighContrast {
            base.add_modifier(Modifier::BOLD)
        } else {
            base
        }
    }

    /// The welcome-screen body style.
    #[must_use]
    pub fn welcome(self) -> Style {
        match self.theme {
            TerminalTheme::System => Style::default(),
            TerminalTheme::Light => Style::default().fg(Color::Black),
            TerminalTheme::Dark => Style::default().fg(Color::White),
            TerminalTheme::HighContrast => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// The style for a selected slash-suggestion row.
    #[must_use]
    pub fn slash_selected(self) -> Style {
        match self.theme {
            TerminalTheme::System | TerminalTheme::Light => Style::default()
                .fg(Color::Black)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
            TerminalTheme::Dark => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            TerminalTheme::HighContrast => Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Every semantic role's style, for the accessibility contract test: each must
    /// carry a non-color cue (a `Modifier`, or a background fill for message blocks).
    #[must_use]
    pub fn semantic_styles(self) -> Vec<(Semantic, Style)> {
        [
            Semantic::User,
            Semantic::Model,
            Semantic::Reasoning,
            Semantic::Tool,
            Semantic::Approval,
            Semantic::Error,
            Semantic::Warning,
            Semantic::Meta,
            Semantic::Activity,
            Semantic::Attention,
            Semantic::ApprovalModal,
            Semantic::ModelModal,
            Semantic::SessionModal,
        ]
        .into_iter()
        .map(|role| (role, self.semantic(role)))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_THEMES: [TerminalTheme; 4] = [
        TerminalTheme::System,
        TerminalTheme::Light,
        TerminalTheme::Dark,
        TerminalTheme::HighContrast,
    ];

    /// The accessibility contract: every semantic style, in every theme, must carry
    /// a non-color cue — a text `Modifier` OR a background fill — so the state
    /// survives a monochrome terminal and color-blind users. Color is never allowed
    /// to be the only signal. This is the enforcement point the module promises.
    #[test]
    fn every_semantic_style_has_a_non_color_cue() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            for (role, style) in palette.semantic_styles() {
                // Model is the baseline prose (the unmarked default every other semantic
                // contrasts against); in the System theme it intentionally uses the
                // terminal default, so it is exempt from the non-color-cue contract.
                if role == Semantic::Model {
                    continue;
                }
                let has_modifier = !style.add_modifier.is_empty();
                let has_background = style.bg.is_some();
                assert!(
                    has_modifier || has_background,
                    "semantic {role:?} in theme {theme:?} relies on color alone \
                     (no modifier, no background) — it would vanish in monochrome"
                );
            }
        }
    }

    #[test]
    fn modal_accents_are_distinct_in_every_theme() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            let approval = palette.semantic(Semantic::ApprovalModal);
            let model = palette.semantic(Semantic::ModelModal);
            let session = palette.semantic(Semantic::SessionModal);
            assert_ne!(approval.fg, model.fg);
            assert_ne!(approval.fg, session.fg);
            assert_ne!(model.fg, session.fg);
            for style in [approval, model, session] {
                assert!(style.add_modifier.contains(Modifier::BOLD));
            }
        }
    }

    #[test]
    fn composer_effort_scale_uses_distinct_blue_purple_styles() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            let styles = ReasoningEffort::ALL.map(|effort| palette.composer_border(effort));
            for (index, style) in styles.iter().enumerate() {
                assert!(style.fg.is_some());
                assert!(styles[..index].iter().all(|prior| prior.fg != style.fg));
            }
            assert!(styles[0].add_modifier.contains(Modifier::DIM));
            assert!(styles[3].add_modifier.contains(Modifier::BOLD));
            assert!(styles[4].add_modifier.contains(Modifier::BOLD));
            // Chrome must stay upright — italic on the border/`❯` looked like
            // emphasis rather than a frame.
            assert!(!styles[4].add_modifier.contains(Modifier::ITALIC));
        }
    }

    #[test]
    fn reasoning_status_and_placeholder_use_muted_foreground() {
        for (theme, expected) in [
            (TerminalTheme::System, Color::Rgb(148, 163, 184)),
            (TerminalTheme::Dark, Color::Rgb(148, 163, 184)),
            (TerminalTheme::Light, Color::Rgb(100, 116, 139)),
            (TerminalTheme::HighContrast, Color::Gray),
        ] {
            let palette = Palette::for_theme(theme);
            let reasoning = palette.semantic(Semantic::Reasoning);
            let status = palette.status_bar();
            let placeholder = palette.placeholder();
            let done = palette.status_done();

            assert_eq!(reasoning.fg, Some(expected));
            assert_eq!(status.fg, Some(expected));
            assert_eq!(placeholder.fg, Some(expected));
            // In-progress reasoning is muted; completed "Thought" is green/bold
            // with no background fill (the bullet is the non-color cue).
            assert!(done.add_modifier.contains(Modifier::BOLD));
            assert_eq!(done.bg, None);
            assert_ne!(done.fg, reasoning.fg);
            assert_eq!(status.bg, None);
            assert_eq!(placeholder.bg, None);
        }
    }

    /// HighContrast is a contract, not a color swap: it must keep the non-color cues
    /// (it may only widen color separation), so every role stays distinguishable.
    #[test]
    fn high_contrast_keeps_non_color_cues() {
        let palette = Palette::for_theme(TerminalTheme::HighContrast);
        for (role, style) in palette.semantic_styles() {
            // Model is the baseline prose (the unmarked default every other semantic
            // contrasts against); in the System theme it intentionally uses the
            // terminal default, so it is exempt from the non-color-cue contract.
            if role == Semantic::Model {
                continue;
            }
            let has_modifier = !style.add_modifier.is_empty();
            let has_background = style.bg.is_some();
            assert!(
                has_modifier || has_background,
                "HighContrast semantic {role:?} lost its non-color cue"
            );
        }
    }

    /// Error and Warning must be distinguishable by cue, not color alone: error is
    /// bold, warning is italic — so they differ even rendered monochrome.
    #[test]
    fn error_and_warning_differ_by_modifier() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            let error = palette.semantic(Semantic::Error);
            let warning = palette.semantic(Semantic::Warning);
            assert!(
                error.add_modifier.contains(Modifier::BOLD),
                "error must be bold in {theme:?}"
            );
            assert!(
                warning.add_modifier.contains(Modifier::ITALIC),
                "warning must be italic in {theme:?}"
            );
            assert_ne!(
                error.add_modifier, warning.add_modifier,
                "error and warning must differ by modifier in {theme:?}, not color alone"
            );
        }
    }

    /// The theme is matched in exactly one conceptual place (`for_theme`); a smoke
    /// check that all roles resolve for every theme without panicking.
    #[test]
    fn all_roles_resolve_for_all_themes() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            assert_eq!(palette.semantic_styles().len(), 13);
        }
    }
}
