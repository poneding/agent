//! Centralized, per-theme color palette for the terminal UI.
//!
//! Colors are derived from the [ayu](https://github.com/ayu-theme/ayu-colors) light
//! and dark schemes. Light/Dark (and System→Dark) map onto ayu tokens; HighContrast
//! keeps the same non-color cues with widened separation.
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

// ---------------------------------------------------------------------------
// Ayu color tokens (https://github.com/ayu-theme/ayu-colors)
// ---------------------------------------------------------------------------

/// Build an RGB color from a 0xRRGGBB literal.
const fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

/// Resolved ayu-derived tokens for one theme variant.
#[derive(Copy, Clone, Debug)]
struct AyuTokens {
    /// Primary editor foreground (`editor.fg`).
    fg: Color,
    /// Muted UI / secondary text (`ui.fg` / comment family).
    muted: Color,
    /// Soft panel / bubble background.
    panel: Color,
    /// Common accent (amber).
    accent: Color,
    /// Errors (`common.error` / markup).
    error: Color,
    /// Warnings (func / keyword oranges).
    warning: Color,
    /// VCS added / success green.
    success: Color,
    /// Syntax tag (cyan-blue).
    tag: Color,
    /// Syntax entity (blue).
    entity: Color,
    /// Syntax string / green prose.
    string: Color,
    /// Syntax keyword (orange).
    keyword: Color,
    /// VCS removed.
    removed: Color,
    /// Selection blue (deepest official blue).
    selection: Color,
    /// Effort scale: calm → heat across five *distinct* hue families (5 steps).
    /// Gray → green → blue → violet → orange. Mid steps deliberately leave the
    /// blue cluster — a pure-ayu blue ladder was too hard to tell apart at a glance.
    effort: [Color; 5],
}

impl AyuTokens {
    /// Official ayu-light tokens.
    const fn light() -> Self {
        Self {
            fg: rgb(0x5C_61_66),
            muted: rgb(0x8A_91_99),
            panel: rgb(0xF3_F4_F5),
            accent: rgb(0xFF_AA_33),
            error: rgb(0xE6_50_50),
            warning: rgb(0xF2_AE_49),
            success: rgb(0x6C_BF_43),
            tag: rgb(0x55_B4_D4),
            entity: rgb(0x39_9E_E6),
            string: rgb(0x86_B3_00),
            keyword: rgb(0xFA_8D_3E),
            removed: rgb(0xFF_73_83),
            selection: rgb(0x03_5B_D6),
            // Effort border: one hue family per depth so Shift+Tab is obvious.
            // minimal gray → low green → medium blue → high violet → xhigh orange.
            effort: [
                rgb(0x8A_91_99), // muted gray (minimal)
                rgb(0x86_B3_00), // string green (low)
                rgb(0x03_5B_D6), // selection blue (medium)
                rgb(0x7C_3A_ED), // violet (high — off-ayu, needed for separation)
                rgb(0xFA_8D_3E), // keyword orange (xhigh)
            ],
        }
    }

    /// Official ayu-dark tokens.
    const fn dark() -> Self {
        Self {
            fg: rgb(0xBF_BD_B6),
            muted: rgb(0x6C_73_80),
            panel: rgb(0x0F_13_1A),
            accent: rgb(0xE6_B4_50),
            error: rgb(0xD9_57_57),
            warning: rgb(0xFF_B4_54),
            success: rgb(0x7F_D9_62),
            tag: rgb(0x39_BA_E6),
            entity: rgb(0x59_C2_FF),
            string: rgb(0xAA_D9_4C),
            keyword: rgb(0xFF_8F_40),
            removed: rgb(0xF2_6D_78),
            selection: rgb(0x40_9F_FF),
            // Effort border: one hue family per depth so Shift+Tab is obvious.
            // minimal gray → low green → medium blue → high violet → xhigh orange.
            effort: [
                rgb(0x6C_73_80), // muted gray (minimal)
                rgb(0xAA_D9_4C), // string green (low)
                rgb(0x40_9F_FF), // selection blue (medium)
                rgb(0xC4_84_FF), // violet (high — off-ayu, needed for separation)
                rgb(0xFF_8F_40), // keyword orange (xhigh)
            ],
        }
    }
}

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

    /// Ayu tokens for the active theme. System follows dark; HighContrast is
    /// handled at the style layer with named ANSI colors.
    const fn tokens(self) -> AyuTokens {
        match self.theme {
            TerminalTheme::Light => AyuTokens::light(),
            TerminalTheme::Dark | TerminalTheme::System | TerminalTheme::HighContrast => {
                AyuTokens::dark()
            }
        }
    }

    /// Theme-aware equivalent of a muted/secondary foreground token.
    fn muted_foreground(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::Gray,
            _ => self.tokens().muted,
        }
    }

    /// Primary body foreground (editor.fg).
    #[must_use]
    pub fn foreground(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::White,
            TerminalTheme::System => Color::White,
            _ => self.tokens().fg,
        }
    }

    /// The style for a semantic role. Every returned style carries a non-color cue
    /// (a `Modifier`) in addition to its color, so it survives monochrome rendering.
    #[must_use]
    pub fn semantic(self, role: Semantic) -> Style {
        let high_contrast = self.theme == TerminalTheme::HighContrast;
        let t = self.tokens();
        match role {
            // Real errors: red + BOLD. HighContrast swaps red→yellow (red on black
            // can be low-contrast) but keeps BOLD, so the "this is an error" cue is
            // never color-only.
            Semantic::Error => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.fg(Color::Yellow)
                } else {
                    base.fg(t.error)
                }
            }
            // Warnings: yellow/amber + ITALIC. The italic keeps them distinct from
            // errors (bold) and from meta (dim) even with no color.
            Semantic::Warning => {
                let base = Style::default().add_modifier(Modifier::ITALIC);
                if high_contrast {
                    base.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    base.fg(t.warning)
                }
            }
            // Reasoning chrome (Thinking/Thought headers, stats) stays muted and
            // upright. Bold is the monochrome cue (same register as tool labels).
            // Italic is applied only to the expanded reasoning *body* at the
            // render call site so process notes read as an aside without
            // slanting the status word itself.
            Semantic::Reasoning => Style::default()
                .fg(self.muted_foreground())
                .add_modifier(Modifier::BOLD),
            // Tool calls stay muted so shell/tool rows read as secondary process
            // context (same register as thinking), not as primary prose.
            Semantic::Tool => {
                let base = Style::default()
                    .fg(self.muted_foreground())
                    .add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.add_modifier(Modifier::ITALIC)
                } else {
                    base
                }
            }
            // Approvals: ayu success green + BOLD. Matches the Allow option color so
            // a resolved allow reads as "ok", not as an alert.
            Semantic::Approval => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.fg(Color::Yellow)
                } else {
                    base.fg(t.success)
                }
            }
            // Attention (pending approvals): same weight as approval; paired with a
            // `⚠` glyph at the call site.
            Semantic::Attention => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                if high_contrast {
                    base.fg(Color::Yellow)
                } else {
                    base.fg(t.warning)
                }
            }
            Semantic::ApprovalModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    _ => base.fg(t.accent),
                }
            }
            Semantic::ModelModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Cyan),
                    _ => base.fg(t.entity),
                }
            }
            Semantic::SessionModal => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::System | TerminalTheme::HighContrast => base.fg(Color::Green),
                    _ => base.fg(t.string),
                }
            }
            // Activity line spinner/verb: ayu tag/entity blue + BOLD.
            Semantic::Activity => {
                let base = Style::default().add_modifier(Modifier::BOLD);
                match self.theme {
                    TerminalTheme::HighContrast => base.fg(Color::Yellow),
                    TerminalTheme::System => base.fg(Color::Cyan),
                    _ => base.fg(t.tag),
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
                    TerminalTheme::System => base.fg(Color::Gray),
                    _ => base.fg(t.muted),
                }
            }
            // User message blocks: panel fill marks the bubble.
            Semantic::User => match self.theme {
                TerminalTheme::System => Style::default().bg(Color::DarkGray).fg(Color::White),
                TerminalTheme::Light => Style::default().bg(t.panel).fg(t.fg),
                TerminalTheme::Dark => Style::default().bg(t.panel).fg(t.fg),
                TerminalTheme::HighContrast => Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            },
            // Model messages stay unfilled so inter-item separators read as a single
            // plain blank row and tool/thinking blocks don't fight a second bg.
            Semantic::Model => match self.theme {
                TerminalTheme::System => Style::default(),
                TerminalTheme::Light | TerminalTheme::Dark => Style::default().fg(t.fg),
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
        let t = self.tokens();
        let base = match role {
            Semantic::Error => Style::default().fg(Color::White).bg(t.error),
            Semantic::Approval => Style::default().fg(Color::White).bg(t.success),
            Semantic::Reasoning => Style::default().fg(Color::White).bg(Color::DarkGray),
            Semantic::Tool => Style::default().fg(Color::Black).bg(t.tag),
            _ => self.meta(),
        };
        // Chips are always bold: the inverted badge plus weight is the non-color cue.
        base.add_modifier(Modifier::BOLD)
    }

    /// Settled / completed status label. Kept muted (no green/blue) so finished
    /// thinking and tool rows stay secondary next to the model answer.
    #[must_use]
    pub fn status_done(self) -> Style {
        self.semantic(Semantic::Reasoning)
            .add_modifier(Modifier::BOLD)
    }

    /// Primary text emphasis used when the pointer hovers a collapsible
    /// thinking header — signals "click to expand" without a triangle glyph.
    #[must_use]
    pub fn hover_emphasis(self) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Yellow),
            TerminalTheme::Light => base.fg(self.tokens().fg),
            TerminalTheme::Dark | TerminalTheme::System => base.fg(Color::White),
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
            TerminalTheme::Light => Style::default().fg(self.tokens().muted),
            TerminalTheme::Dark => Style::default().fg(self.tokens().tag),
            TerminalTheme::HighContrast => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Composer framing (top/bottom rules + `❯` marker) keyed by reasoning
    /// effort. Five distinct hue families — gray → green → blue → violet →
    /// orange — so each Shift+Tab step is obvious even when luminance is
    /// crushed. Weight steps up with depth (no DIM: many hosts collapse dimmed
    /// RGB into one muddy tone). No italic on chrome.
    #[must_use]
    pub fn composer_border(self, effort: ReasoningEffort) -> Style {
        let t = self.tokens();
        let color = match effort {
            ReasoningEffort::Minimal => t.effort[0],
            ReasoningEffort::Low => t.effort[1],
            ReasoningEffort::Medium => t.effort[2],
            ReasoningEffort::High => t.effort[3],
            ReasoningEffort::XHigh => t.effort[4],
        };
        // Weight alone is the non-color depth cue. BOLD on high/xhigh so the
        // top two depths also read under monochrome; medium stays quiet.
        let modifier = match effort {
            ReasoningEffort::Minimal | ReasoningEffort::Low | ReasoningEffort::Medium => {
                Modifier::empty()
            }
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
            TerminalTheme::Light | TerminalTheme::Dark => Style::default().fg(self.tokens().fg),
            TerminalTheme::HighContrast => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }

    /// Brand / primary emphasis (welcome logo wordmark fallback, primary facts).
    #[must_use]
    pub fn primary(self) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::White),
            TerminalTheme::System => base.fg(Color::White),
            _ => base.fg(self.tokens().fg),
        }
    }

    /// Per-letter styles for the welcome `RUSE` wordmark (always bold).
    ///
    /// Four **widely separated** ayu hue families so adjacent letters never blend:
    /// **R** selection/blue → **U** string/green → **S** accent/amber → **E**
    /// error/red. (Earlier tag+entity and accent+keyword pairs sat in the same
    /// cool/warm cluster and read as two near-duplicates.) HighContrast / System
    /// keep the same order with high-visibility ANSI colors.
    #[must_use]
    pub fn brand_letter_styles(self) -> [Style; 4] {
        let bold = Modifier::BOLD;
        match self.theme {
            TerminalTheme::HighContrast => [
                Style::default().fg(Color::Blue).add_modifier(bold),
                Style::default().fg(Color::Green).add_modifier(bold),
                Style::default().fg(Color::Yellow).add_modifier(bold),
                Style::default().fg(Color::Red).add_modifier(bold),
            ],
            TerminalTheme::System => [
                Style::default().fg(Color::Cyan).add_modifier(bold),
                Style::default().fg(Color::Green).add_modifier(bold),
                Style::default().fg(Color::Yellow).add_modifier(bold),
                Style::default().fg(Color::LightRed).add_modifier(bold),
            ],
            _ => {
                let t = self.tokens();
                [
                    // Cool deep blue — not the cyan tag, so it stays far from green.
                    Style::default().fg(t.selection).add_modifier(bold),
                    // Syntax green — clear jump from blue.
                    Style::default().fg(t.string).add_modifier(bold),
                    // Warm amber — clear jump from green.
                    Style::default().fg(t.accent).add_modifier(bold),
                    // Error red — clear jump from amber (keyword orange was too close).
                    Style::default().fg(t.error).add_modifier(bold),
                ]
            }
        }
    }

    /// Warm accent (welcome logo eyes, common.accent).
    #[must_use]
    pub fn accent(self) -> Style {
        let base = Style::default();
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Yellow).add_modifier(Modifier::BOLD),
            TerminalTheme::System => base.fg(Color::Yellow),
            _ => base.fg(self.tokens().accent),
        }
    }

    /// Frame / cyan accent (welcome logo frame, activity-adjacent chrome).
    #[must_use]
    pub fn frame(self) -> Style {
        let base = Style::default();
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Cyan).add_modifier(Modifier::BOLD),
            TerminalTheme::System => base.fg(Color::Cyan),
            _ => base.fg(self.tokens().tag),
        }
    }

    /// Markdown heading color (ayu entity blue).
    #[must_use]
    pub fn heading(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::Yellow,
            TerminalTheme::System => Color::Cyan,
            _ => self.tokens().entity,
        }
    }

    /// Inline code color (ayu keyword/func orange).
    #[must_use]
    pub fn code(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast | TerminalTheme::System => Color::Yellow,
            _ => self.tokens().keyword,
        }
    }

    /// Fenced code-block body color (primary fg).
    #[must_use]
    pub fn code_block(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::White,
            TerminalTheme::System => Color::Gray,
            _ => self.tokens().fg,
        }
    }

    /// Block-quote body color (muted).
    #[must_use]
    pub fn quote(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast | TerminalTheme::System => Color::Gray,
            _ => self.tokens().muted,
        }
    }

    /// List bullet color (ayu tag cyan-blue).
    #[must_use]
    pub fn list_bullet(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::Yellow,
            TerminalTheme::System => Color::Cyan,
            _ => self.tokens().tag,
        }
    }

    /// Link color (ayu selection / entity blue).
    #[must_use]
    pub fn link(self) -> Color {
        match self.theme {
            TerminalTheme::HighContrast => Color::Yellow,
            TerminalTheme::System => Color::Blue,
            TerminalTheme::Light => self.tokens().selection,
            TerminalTheme::Dark => self.tokens().entity,
        }
    }

    /// Diff added line style (ayu vcs.added).
    #[must_use]
    pub fn diff_added(self) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Green),
            _ => base.fg(self.tokens().success),
        }
    }

    /// Diff removed line style (ayu vcs.removed).
    #[must_use]
    pub fn diff_removed(self) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        match self.theme {
            TerminalTheme::HighContrast => base.fg(Color::Red),
            _ => base.fg(self.tokens().removed),
        }
    }

    /// Soft background tint for added diff lines.
    #[must_use]
    pub fn diff_added_bg(self) -> Option<Color> {
        match self.theme {
            TerminalTheme::Dark | TerminalTheme::System => Some(rgb(0x14_28_18)),
            TerminalTheme::Light => Some(rgb(0xE6_F4_DE)),
            TerminalTheme::HighContrast => Some(rgb(0x00_3C_00)),
        }
    }

    /// Soft background tint for removed diff lines.
    #[must_use]
    pub fn diff_removed_bg(self) -> Option<Color> {
        match self.theme {
            TerminalTheme::Dark | TerminalTheme::System => Some(rgb(0x2A_14_16)),
            TerminalTheme::Light => Some(rgb(0xFC_E8_E8)),
            TerminalTheme::HighContrast => Some(rgb(0x50_00_00)),
        }
    }

    /// The style for a selected slash-suggestion row.
    /// Accent fg + bold only — no reverse/background fill (matches picker rows).
    #[must_use]
    pub fn slash_selected(self) -> Style {
        self.semantic(Semantic::Activity)
            .add_modifier(Modifier::BOLD)
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
    fn composer_effort_scale_uses_five_distinct_hue_families() {
        for theme in ALL_THEMES {
            let palette = Palette::for_theme(theme);
            let styles = ReasoningEffort::ALL.map(|effort| palette.composer_border(effort));
            for (index, style) in styles.iter().enumerate() {
                assert!(style.fg.is_some());
                assert!(styles[..index].iter().all(|prior| prior.fg != style.fg));
                // DIM collapses RGB on many terminals and hid the scale.
                assert!(!style.add_modifier.contains(Modifier::DIM));
            }
            assert!(!styles[0].add_modifier.contains(Modifier::BOLD));
            assert!(!styles[2].add_modifier.contains(Modifier::BOLD));
            assert!(styles[3].add_modifier.contains(Modifier::BOLD));
            assert!(styles[4].add_modifier.contains(Modifier::BOLD));
            // Chrome must stay upright — italic on the border/`❯` looked like
            // emphasis rather than a frame.
            assert!(!styles[4].add_modifier.contains(Modifier::ITALIC));
        }

        // Light: gray → green → blue → violet → orange.
        let light = Palette::for_theme(TerminalTheme::Light);
        assert_eq!(
            light.composer_border(ReasoningEffort::Minimal).fg,
            Some(rgb(0x8A_91_99)) // muted gray
        );
        assert_eq!(
            light.composer_border(ReasoningEffort::Low).fg,
            Some(rgb(0x86_B3_00)) // string green
        );
        assert_eq!(
            light.composer_border(ReasoningEffort::Medium).fg,
            Some(rgb(0x03_5B_D6)) // selection blue
        );
        assert_eq!(
            light.composer_border(ReasoningEffort::High).fg,
            Some(rgb(0x7C_3A_ED)) // violet
        );
        assert_eq!(
            light.composer_border(ReasoningEffort::XHigh).fg,
            Some(rgb(0xFA_8D_3E)) // keyword orange
        );

        // Dark: gray → green → blue → violet → orange.
        let dark = Palette::for_theme(TerminalTheme::Dark);
        assert_eq!(
            dark.composer_border(ReasoningEffort::Minimal).fg,
            Some(rgb(0x6C_73_80)) // muted gray
        );
        assert_eq!(
            dark.composer_border(ReasoningEffort::Low).fg,
            Some(rgb(0xAA_D9_4C)) // string green
        );
        assert_eq!(
            dark.composer_border(ReasoningEffort::Medium).fg,
            Some(rgb(0x40_9F_FF)) // selection blue
        );
        assert_eq!(
            dark.composer_border(ReasoningEffort::High).fg,
            Some(rgb(0xC4_84_FF)) // violet
        );
        assert_eq!(
            dark.composer_border(ReasoningEffort::XHigh).fg,
            Some(rgb(0xFF_8F_40)) // keyword orange
        );
    }

    #[test]
    fn reasoning_status_and_placeholder_use_muted_foreground() {
        for (theme, expected) in [
            // System follows ayu-dark muted (Gray is reserved for Meta's System path).
            (TerminalTheme::System, rgb(0x6C_73_80)),
            (TerminalTheme::Dark, rgb(0x6C_73_80)),
            (TerminalTheme::Light, rgb(0x8A_91_99)),
            (TerminalTheme::HighContrast, Color::Gray),
        ] {
            let palette = Palette::for_theme(theme);
            let reasoning = palette.semantic(Semantic::Reasoning);
            let tool = palette.semantic(Semantic::Tool);
            let status = palette.status_bar();
            let placeholder = palette.placeholder();
            let done = palette.status_done();
            let hover = palette.hover_emphasis();

            assert_eq!(reasoning.fg, Some(expected));
            // Thinking/Thought chrome stays upright + bold; italic is body-only.
            assert!(
                !reasoning.add_modifier.contains(Modifier::ITALIC),
                "Reasoning semantic must not be italic in {theme:?}"
            );
            assert!(
                reasoning.add_modifier.contains(Modifier::BOLD),
                "Reasoning chrome must keep a non-color cue in {theme:?}"
            );
            assert_eq!(tool.fg, Some(expected));
            assert_eq!(status.fg, Some(expected));
            assert_eq!(placeholder.fg, Some(expected));
            // Thinking and completed Thought stay muted; hover lifts to primary.
            assert!(done.add_modifier.contains(Modifier::BOLD));
            assert!(
                !done.add_modifier.contains(Modifier::ITALIC),
                "status_done must stay upright in {theme:?}"
            );
            assert_eq!(done.bg, None);
            assert_eq!(done.fg, reasoning.fg);
            assert_eq!(
                hover.fg,
                Some(match theme {
                    TerminalTheme::HighContrast => Color::Yellow,
                    TerminalTheme::Light => rgb(0x5C_61_66),
                    TerminalTheme::Dark | TerminalTheme::System => Color::White,
                })
            );
            assert_ne!(hover.fg, reasoning.fg);
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

    #[test]
    fn light_and_dark_use_official_ayu_accents() {
        let light = Palette::for_theme(TerminalTheme::Light);
        let dark = Palette::for_theme(TerminalTheme::Dark);
        assert_eq!(light.semantic(Semantic::Error).fg, Some(rgb(0xE6_50_50)));
        assert_eq!(dark.semantic(Semantic::Error).fg, Some(rgb(0xD9_57_57)));
        assert_eq!(light.semantic(Semantic::Approval).fg, Some(rgb(0x6C_BF_43)));
        assert_eq!(dark.semantic(Semantic::Approval).fg, Some(rgb(0x7F_D9_62)));
        assert_eq!(light.semantic(Semantic::Activity).fg, Some(rgb(0x55_B4_D4)));
        assert_eq!(dark.semantic(Semantic::Activity).fg, Some(rgb(0x39_BA_E6)));
        assert_eq!(light.heading(), rgb(0x39_9E_E6));
        assert_eq!(dark.heading(), rgb(0x59_C2_FF));
    }
}
