//! Colored welcome / start screen for an empty transcript.
//!
//! Layout:
//! ```text
//!   ╭ ─◯  ◯─ ╮     Model
//! ╭─┤────────┤─╮     openai / gpt-5.5
//!   │  RUSE  │       context pending/128k
//!     v0.1.0
//!
//!                  Workspace
//!                    ~/project
//!                    idle · new session
//! ```
//!
//! Wide terminals put the logo on the left and facts on the right; narrow
//! terminals stack logo above facts. Primary (brand / values) and secondary
//! (labels / meta) use distinct palette roles — never color alone for meaning.
//! The `RUSE` wordmark is bold multicolor (one ayu hue per letter); the package
//! version sits centered under the logo mark.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::app::ConnectionState;
use crate::{Palette, TerminalTheme, TuiApp};

/// Prefer a side-by-side logo | facts layout at this content width.
const SIDE_BY_SIDE_MIN_WIDTH: usize = 64;

/// Logo art (exact brand mark the product starts with). The third row is a
/// structural template; the live wordmark is built per-letter in [`logo_lines`].
const LOGO: [&str; 3] = ["  ╭ ─◯  ◯─ ╮", "╭─┤────────┤─╮", "  │  RUSE  │  "];

/// Package version shown under the logo (`env!("CARGO_PKG_VERSION")`).
fn package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Display width of the logo mark (widest art row). Version is centered in this.
fn logo_mark_width() -> usize {
    LOGO.iter()
        .map(|row| UnicodeWidthStr::width(*row))
        .max()
        .unwrap_or(14)
}

/// Build the welcome body as styled lines for the given content width.
#[must_use]
pub fn welcome_lines(
    app: &TuiApp,
    content_width: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    welcome_lines_for(app, content_width, usize::MAX, theme)
}

/// Like [`welcome_lines`], but height-aware: very short viewports get a compact
/// one-line facts strip so Model/Workspace still fit under the logo.
#[must_use]
pub fn welcome_lines_for(
    app: &TuiApp,
    content_width: usize,
    content_height: usize,
    theme: TerminalTheme,
) -> Vec<Line<'static>> {
    let logo = logo_lines(theme);
    let width = content_width.max(16);
    let facts = fact_lines(app, theme);
    let full = if width >= SIDE_BY_SIDE_MIN_WIDTH {
        merge_side_by_side(logo.clone(), facts, width)
    } else {
        merge_stacked(logo.clone(), facts)
    };

    // Prefer a compact strip under the logo when the full card would clip.
    if content_height > 0 && full.len() > content_height {
        let logo_rows = logo.len();
        let mut out = logo;
        if content_height > logo_rows {
            out.push(Line::from(""));
            out.push(compact_facts_line(app, theme));
        }
        return out;
    }
    full
}

fn compact_facts_line(app: &TuiApp, theme: TerminalTheme) -> Line<'static> {
    let primary = primary_style(theme);
    let secondary = secondary_style(theme);
    let model = format!("{} / {}", app.status.provider, app.status.model);
    let workspace = short_workspace_path(&app.status.workspace);
    Line::from(vec![
        Span::styled(model, primary),
        Span::styled("  ·  ", secondary),
        Span::styled(workspace, primary),
    ])
}

fn logo_lines(theme: TerminalTheme) -> Vec<Line<'static>> {
    let frame = logo_frame_style(theme);
    let accent = logo_accent_style(theme);
    let version = secondary_style(theme);

    vec![
        //  ╭ ─◯  ◯─ ╮
        Line::from(vec![
            Span::styled("  ╭ ", frame),
            Span::styled("─", frame),
            Span::styled("◯", accent),
            Span::styled("  ", frame),
            Span::styled("◯", accent),
            Span::styled("─", frame),
            Span::styled(" ╮", frame),
        ]),
        // ╭─┤────────┤─╮
        Line::from(Span::styled(LOGO[1].to_string(), frame)),
        //   │  RUSE  │  — each letter its own ayu hue + bold
        Line::from({
            let mut spans = vec![Span::styled("  │  ", frame)];
            spans.extend(brand_wordmark_spans(theme));
            spans.push(Span::styled("  │  ", frame));
            spans
        }),
        // Version centered under the mark (keeps the left column balanced).
        centered_version_line(version),
    ]
}

/// `RUSE` as four bold, multicolor spans (ayu selection / string / accent / error).
fn brand_wordmark_spans(theme: TerminalTheme) -> Vec<Span<'static>> {
    let styles = Palette::for_theme(theme).brand_letter_styles();
    "RUSE"
        .chars()
        .zip(styles)
        .map(|(ch, style)| Span::styled(ch.to_string(), style))
        .collect()
}

/// Center `v{version}` within the logo mark width using the secondary/meta style.
fn centered_version_line(style: Style) -> Line<'static> {
    let label = format!("v{}", package_version());
    let mark_w = logo_mark_width();
    let label_w = UnicodeWidthStr::width(label.as_str());
    let pad = mark_w.saturating_sub(label_w) / 2;
    Line::from(vec![Span::raw(" ".repeat(pad)), Span::styled(label, style)])
}

fn fact_lines(app: &TuiApp, theme: TerminalTheme) -> Vec<Line<'static>> {
    let primary = primary_style(theme);
    let secondary = secondary_style(theme);
    let mut lines = Vec::new();

    // Model — primary values; omit the uninformative "ready" status.
    lines.push(section_label("Model", secondary));
    let model = format!("{} / {}", app.status.provider, app.status.model);
    if let Some(health) = provider_health_for_welcome(app) {
        lines.push(Line::from(vec![
            Span::styled(model, primary),
            Span::styled("  ", primary),
            Span::styled(health, secondary),
        ]));
    } else {
        lines.push(Line::from(Span::styled(model, primary)));
    }
    lines.push(Line::from(Span::styled(
        format!("context {}", app.status.context.render()),
        secondary,
    )));
    lines.push(Line::from(""));

    // Workspace — short path (~/…), connection + session as meta.
    lines.push(section_label("Workspace", secondary));
    lines.push(Line::from(Span::styled(
        short_workspace_path(&app.status.workspace),
        primary,
    )));
    let session_hint = if app.session_panel.sessions.is_empty() {
        "new session".to_string()
    } else {
        format!("{} recent sessions", app.session_panel.sessions.len())
    };
    let connection = ConnectionState::parse(&app.status.connection).to_string();
    lines.push(Line::from(Span::styled(
        format!("{connection} · {session_hint}"),
        secondary,
    )));
    lines.push(Line::from(""));

    // Extensions — secondary block.
    lines.push(section_label("Extensions", secondary));
    lines.push(Line::from(Span::styled(
        extension_health_summary(app),
        secondary,
    )));

    lines
}

fn section_label(label: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(
        label.to_string(),
        style.add_modifier(Modifier::BOLD),
    ))
}

fn merge_stacked(logo: Vec<Line<'static>>, facts: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let mut out = logo;
    out.push(Line::from(""));
    out.extend(facts);
    out
}

fn merge_side_by_side(
    logo: Vec<Line<'static>>,
    facts: Vec<Line<'static>>,
    content_width: usize,
) -> Vec<Line<'static>> {
    let logo_w = logo_mark_width();
    let gap = 4;
    let left_w = logo_w;
    let right_w = content_width.saturating_sub(left_w + gap);

    let rows = logo.len().max(facts.len());
    let mut out = Vec::with_capacity(rows);
    for i in 0..rows {
        let left = logo.get(i).cloned().unwrap_or_else(|| Line::from(""));
        let right = facts.get(i).cloned().unwrap_or_else(|| Line::from(""));
        if right_w == 0 {
            out.push(left);
            continue;
        }
        let left_pad = pad_line_to_width(left, left_w);
        let mut spans = left_pad.spans;
        spans.push(Span::raw(" ".repeat(gap)));
        // Truncate right side by display width if needed.
        spans.extend(truncate_line(right, right_w).spans);
        out.push(Line::from(spans));
    }
    out
}

fn pad_line_to_width(mut line: Line<'static>, width: usize) -> Line<'static> {
    let used = line.width();
    if used < width {
        line.spans.push(Span::raw(" ".repeat(width - used)));
    }
    line
}

fn truncate_line(line: Line<'static>, max_width: usize) -> Line<'static> {
    if line.width() <= max_width {
        return line;
    }
    let mut spans = Vec::new();
    let mut used = 0usize;
    for span in line.spans {
        let text = span.content.as_ref();
        let w = UnicodeWidthStr::width(text);
        if used + w <= max_width {
            used += w;
            spans.push(span);
            continue;
        }
        // Fit a prefix of this span.
        let mut piece = String::new();
        for ch in text.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if used + cw > max_width {
                break;
            }
            piece.push(ch);
            used += cw;
        }
        if !piece.is_empty() {
            spans.push(Span::styled(piece, span.style));
        }
        break;
    }
    Line::from(spans)
}

/// Collapse `$HOME/...` → `~/...` (and bare home → `~`). Already-short paths pass through.
#[must_use]
pub fn short_workspace_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return ".".to_string();
    }
    if trimmed == "~" || trimmed.starts_with("~/") {
        return trimmed.to_string();
    }

    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| h.to_string_lossy().into_owned());

    if let Some(home) = home.as_deref() {
        let home = home.trim_end_matches('/');
        if trimmed == home {
            return "~".to_string();
        }
        let prefix = format!("{home}/");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return format!("~/{rest}");
        }
    }
    trimmed.to_string()
}

/// Provider/model health for the welcome screen. Suppresses the uninformative
/// `"ready"` status so healthy setups stay clean.
fn provider_health_for_welcome(app: &TuiApp) -> Option<String> {
    let summary = provider_health_summary(app);
    if summary == "ready" {
        None
    } else {
        Some(summary)
    }
}

fn provider_health_summary(app: &TuiApp) -> String {
    if let Some(selected) = app
        .status_panel
        .provider_support
        .iter()
        .find(|provider| provider.provider_id == app.status.provider)
    {
        if selected.enabled && selected.compatible {
            return "ready".to_string();
        }
        if selected.enabled && !selected.missing.is_empty() {
            return format!("needs {}", selected.missing.join(","));
        }
        if !selected.enabled {
            return "disabled".to_string();
        }
    }

    match ConnectionState::parse(&app.status.connection) {
        ConnectionState::Failed => "failed".to_string(),
        ConnectionState::Degraded => "needs attention".to_string(),
        ConnectionState::Streaming => "streaming".to_string(),
        ConnectionState::Idle | ConnectionState::Disconnected => "ready".to_string(),
    }
}

fn extension_health_summary(app: &TuiApp) -> String {
    let Some(snapshot) = &app.extension_panel.snapshot else {
        return "plugin and MCP health pending".to_string();
    };
    if snapshot.extensions.is_empty() {
        return "no plugins · MCP idle".to_string();
    }
    let enabled = snapshot
        .extensions
        .iter()
        .filter(|extension| extension.enabled)
        .count();
    let healthy = snapshot
        .extensions
        .iter()
        .filter(|extension| extension.health == "healthy")
        .count();
    let diagnostics = snapshot.diagnostics.len();
    format!("{enabled} enabled · {healthy} healthy · {diagnostics} diagnostics")
}

// -- theme-aware colors (ayu via Palette) -------------------------------------

fn logo_frame_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).frame()
}

fn logo_accent_style(theme: TerminalTheme) -> Style {
    // The two "eyes" (◯) — ayu common.accent so the mark reads as a face.
    Palette::for_theme(theme).accent()
}

/// Primary body — model name, workspace path.
fn primary_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).primary()
}

/// Secondary body — labels, context line, session meta.
fn secondary_style(theme: TerminalTheme) -> Style {
    Palette::for_theme(theme).meta()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_path_rewrites_home_prefix() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/test".into());
        let full = format!("{home}/workspace/ruse");
        assert_eq!(short_workspace_path(&full), "~/workspace/ruse");
        assert_eq!(short_workspace_path(&home), "~");
        assert_eq!(short_workspace_path("~/already"), "~/already");
        assert_eq!(short_workspace_path("/tmp/other"), "/tmp/other");
    }

    #[test]
    fn side_by_side_threshold() {
        let app = TuiApp::default();
        let wide = welcome_lines(&app, 80, TerminalTheme::Dark);
        let narrow = welcome_lines(&app, 40, TerminalTheme::Dark);
        // Wide: logo and model share a row → fewer total lines than stacked.
        assert!(
            wide.len() < narrow.len(),
            "wide={} narrow={}",
            wide.len(),
            narrow.len()
        );
    }

    #[test]
    fn logo_contains_brand() {
        let app = TuiApp::default();
        let lines = welcome_lines(&app, 80, TerminalTheme::Dark);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("RUSE"), "{text}");
        assert!(!text.contains("AGENT"), "{text}");
        assert!(
            !text.contains("ready"),
            "healthy welcome must hide ready: {text}"
        );
        // Version under the logo (package version from Cargo.toml).
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        assert!(
            text.contains(&version),
            "welcome must show package version {version}: {text}"
        );
    }

    #[test]
    fn brand_wordmark_is_multicolor_bold() {
        let spans = brand_wordmark_spans(TerminalTheme::Dark);
        assert_eq!(spans.len(), 4);
        let letters: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(letters, "RUSE");
        // Four distinct colors so the mark is not a flat monochrome word.
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(
                    spans[i].style.fg, spans[j].style.fg,
                    "letters {i} and {j} share a color: {:?}",
                    spans[i].style.fg
                );
            }
        }
        for style in spans.iter().map(|s| s.style) {
            assert!(
                style.add_modifier.contains(Modifier::BOLD),
                "each brand letter must be bold: {style:?}"
            );
        }
    }
}
