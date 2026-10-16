//! In-app text selection (transcript + composer).
//!
//! Mouse capture steals native terminal drag-select. This module restores a
//! usable select+copy path inside the app:
//!
//! - **Click** (no drag) on the transcript expands/collapses thinking/tools via
//!   hit-test. A bare click in the composer is a no-op (caret is keyboard-driven).
//! - **Drag** paints a selection highlight and, on release, **copies** the
//!   selected plain text while **keeping** the highlight until the next gesture.
//!   A short toast confirms the copy above the composer.
//! - A drag is bound to the surface it starts in ([`SelectionRegion`]): the
//!   transcript scrollback or the composer input. The two never cross-highlight
//!   or cross-copy.
//! - Transcript anchors use absolute wrapped-row coordinates, so wheel/PageUp
//!   scrolling during a drag extends the selection across viewport boundaries.
//! - Coordinates use **display columns** (Unicode width), not UTF-8 char counts,
//!   so CJK selection matches what the cursor paints.
//! - **Shift+click/drag** is left to the host terminal when supported.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::TuiApp;

/// Which on-screen surface a selection targets. A drag is bound to the region it
/// starts in so the transcript and the composer input never cross-highlight or
/// cross-copy: extraction and paint both consult this to pick the right text
/// source and geometry.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum SelectionRegion {
    /// The scrollback transcript content area (the default surface).
    #[default]
    Transcript,
    /// The composer input box's text area (excludes the `❯ ` gutter and borders).
    Composer,
}

/// Inclusive-start / exclusive-end range using **display columns**.
/// Composer ranges are viewport-local; transcript ranges stored in
/// [`SelectionState::committed`] use absolute wrapped-row coordinates so they
/// survive scrolling.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SelectionRange {
    pub start_col: u16,
    pub start_row: u16,
    pub end_col: u16,
    pub end_row: u16,
}

impl SelectionRange {
    #[must_use]
    pub fn normalized(self) -> Self {
        let (a_row, a_col, b_row, b_col) =
            if (self.start_row, self.start_col) <= (self.end_row, self.end_col) {
                (self.start_row, self.start_col, self.end_row, self.end_col)
            } else {
                (self.end_row, self.end_col, self.start_row, self.start_col)
            };
        Self {
            start_col: a_col,
            start_row: a_row,
            end_col: b_col,
            end_row: b_row,
        }
    }

    /// Whether the drag moved far enough to count as a selection (not a click).
    #[must_use]
    pub fn is_drag(self) -> bool {
        self.start_col != self.end_col || self.start_row != self.end_row
    }

    /// Whether a content-local **display** cell is inside this (normalized) range.
    #[must_use]
    pub fn contains_cell(self, col: u16, row: u16) -> bool {
        let n = self.normalized();
        if row < n.start_row || row > n.end_row {
            return false;
        }
        if n.start_row == n.end_row {
            let end = n.end_col.max(n.start_col.saturating_add(1));
            return col >= n.start_col && col < end;
        }
        if row == n.start_row {
            return col >= n.start_col;
        }
        if row == n.end_row {
            return col < n.end_col.max(1);
        }
        true
    }
}

/// Live selection state owned by [`TuiApp`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectionState {
    /// Anchor of an in-progress drag, in terminal-absolute coordinates.
    pub drag_anchor: Option<(u16, u16)>,
    /// Current pointer position while dragging (terminal-absolute).
    pub drag_current: Option<(u16, u16)>,
    /// Transcript anchor converted to absolute wrapped-row coordinates when the
    /// drag starts. It stays fixed while the viewport scrolls beneath the mouse.
    pub transcript_drag_anchor: Option<(u16, u16)>,
    /// Committed selection after mouse-up. Transcript rows are absolute document
    /// rows; composer rows are local to the composer text area. The selection is
    /// not cleared by transcript scrolling, so its highlight can move off/on screen.
    pub committed: Option<SelectionRange>,
    /// True once the pointer has moved from the anchor — distinguishes click
    /// from drag on mouse-up.
    pub moved: bool,
    /// Which surface the current gesture / committed selection targets. Set when
    /// a drag begins from the pointer's starting region and read by copy/paint so
    /// the transcript and composer never highlight or extract from each other.
    pub region: SelectionRegion,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            drag_anchor: None,
            drag_current: None,
            transcript_drag_anchor: None,
            committed: None,
            moved: false,
            region: SelectionRegion::Transcript,
        }
    }
}

impl SelectionState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn begin_drag(
        &mut self,
        column: u16,
        row: u16,
        region: SelectionRegion,
        content: &ContentGeometry,
        transcript_viewport_start: usize,
    ) {
        self.drag_anchor = Some((column, row));
        self.drag_current = Some((column, row));
        self.transcript_drag_anchor = if region == SelectionRegion::Transcript {
            content.to_local(column, row).map(|(local_col, local_row)| {
                (
                    local_col,
                    absolute_transcript_row(transcript_viewport_start, local_row),
                )
            })
        } else {
            None
        };
        self.moved = false;
        self.committed = None;
        self.region = region;
    }

    pub fn update_drag(&mut self, column: u16, row: u16) {
        if self.drag_anchor.is_none() {
            return;
        }
        if self.drag_current != Some((column, row)) {
            self.moved = true;
        }
        self.drag_current = Some((column, row));
    }

    /// Finish a drag. Returns `true` if this was a real drag (caller should
    /// keep the committed highlight), `false` if it was a click (caller should
    /// hit-test toggle). Transcript endpoints are resolved against the current
    /// viewport so wheel/PageUp scrolling during the drag extends the selection.
    pub fn end_drag(
        &mut self,
        content: &ContentGeometry,
        transcript_viewport_start: usize,
    ) -> bool {
        let Some(terminal_anchor) = self.drag_anchor.take() else {
            return false;
        };
        let current = self.drag_current.take().unwrap_or(terminal_anchor);
        let committed = match self.region {
            SelectionRegion::Transcript => {
                let anchor = self.transcript_drag_anchor.take().or_else(|| {
                    content
                        .to_local(terminal_anchor.0, terminal_anchor.1)
                        .map(|(col, row)| {
                            (col, absolute_transcript_row(transcript_viewport_start, row))
                        })
                });
                let current = content.to_local(current.0, current.1).map(|(col, row)| {
                    (col, absolute_transcript_row(transcript_viewport_start, row))
                });
                match (anchor, current) {
                    (Some(anchor), Some(current)) if anchor != current => {
                        Some(selection_range_from_cells(anchor, current, content.width))
                    }
                    _ => None,
                }
            }
            SelectionRegion::Composer => {
                self.transcript_drag_anchor = None;
                match (
                    content.to_local(terminal_anchor.0, terminal_anchor.1),
                    content.to_local(current.0, current.1),
                ) {
                    (Some(anchor), Some(current)) if self.moved && anchor != current => {
                        Some(selection_range_from_cells(anchor, current, content.width))
                    }
                    _ => None,
                }
            }
        };
        self.moved = false;
        self.committed = committed;
        committed.is_some()
    }

    /// Active composer-local range for painting: live drag if any, else committed.
    #[must_use]
    pub fn paint_range(&self, content: &ContentGeometry) -> Option<SelectionRange> {
        if self.region != SelectionRegion::Composer {
            return None;
        }
        if let (Some(anchor), Some(current)) = (self.drag_anchor, self.drag_current) {
            if !self.moved {
                return None;
            }
            let anchor = content.to_local(anchor.0, anchor.1)?;
            let current = content.to_local(current.0, current.1)?;
            return Some(selection_range_from_cells(anchor, current, content.width));
        }
        self.committed
    }

    /// Active transcript range projected from absolute document rows into the
    /// current viewport. Off-screen portions remain committed and reappear when
    /// the user scrolls back to them.
    #[must_use]
    pub fn paint_transcript_range(
        &self,
        content: &ContentGeometry,
        transcript_viewport_start: usize,
    ) -> Option<SelectionRange> {
        if self.region != SelectionRegion::Transcript {
            return None;
        }
        let absolute = if let (Some(anchor), Some(current)) =
            (self.transcript_drag_anchor, self.drag_current)
        {
            let current = content
                .to_local(current.0, current.1)
                .map(|(col, row)| (col, absolute_transcript_row(transcript_viewport_start, row)))?;
            if anchor == current {
                return None;
            }
            selection_range_from_cells(anchor, current, content.width)
        } else {
            self.committed?
        };
        project_transcript_range(absolute, transcript_viewport_start, content)
    }
}

fn absolute_transcript_row(viewport_start: usize, local_row: u16) -> u16 {
    u16::try_from(viewport_start.saturating_add(usize::from(local_row))).unwrap_or(u16::MAX)
}

fn selection_range_from_cells(first: (u16, u16), second: (u16, u16), width: u16) -> SelectionRange {
    let (start, end) = if (first.1, first.0) <= (second.1, second.0) {
        (first, second)
    } else {
        (second, first)
    };
    SelectionRange {
        start_col: start.0,
        start_row: start.1,
        end_col: end.0.saturating_add(1).min(width),
        end_row: end.1,
    }
}

fn project_transcript_range(
    range: SelectionRange,
    viewport_start: usize,
    content: &ContentGeometry,
) -> Option<SelectionRange> {
    if content.height == 0 {
        return None;
    }
    let range = range.normalized();
    let range_start = usize::from(range.start_row);
    let range_end = usize::from(range.end_row);
    let viewport_end = viewport_start.saturating_add(usize::from(content.height) - 1);
    if range_end < viewport_start || range_start > viewport_end {
        return None;
    }

    let visible_start = range_start.max(viewport_start);
    let visible_end = range_end.min(viewport_end);
    Some(SelectionRange {
        start_col: if range_start < viewport_start {
            0
        } else {
            range.start_col
        },
        start_row: u16::try_from(visible_start - viewport_start).unwrap_or(u16::MAX),
        end_col: if range_end > viewport_end {
            content.width
        } else {
            range.end_col
        },
        end_row: u16::try_from(visible_end - viewport_start).unwrap_or(u16::MAX),
    })
}

/// Geometry of the transcript content area (matches render + hit-test).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ContentGeometry {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ContentGeometry {
    #[must_use]
    pub fn from_app(app: &TuiApp) -> Option<Self> {
        let width = app.terminal_size.width;
        let height = app.terminal_size.height;
        if width == 0 || height == 0 {
            return None;
        }
        let suggestion_height = if app.composer.slash_suggestions.visible() {
            if app.composer.slash_suggestions.matches.is_empty() {
                1
            } else {
                u16::try_from(app.composer.slash_suggestions.matches.len().min(7)).unwrap_or(7)
            }
        } else {
            0
        };
        let gap_height = crate::render::pre_input_gap_height(
            app.approval_modal_visible(),
            app.turn_state().is_active(),
        );
        let text_rows = u16::try_from(app.composer.line_count())
            .unwrap_or(6)
            .clamp(1, 6);
        let composer_height = text_rows.saturating_add(2);
        let reserved = gap_height
            .saturating_add(suggestion_height)
            .saturating_add(composer_height)
            .saturating_add(1);
        let primary_height = height.saturating_sub(reserved).max(1);
        let gutter = u16::from(width >= 4);
        Some(Self {
            x: gutter,
            y: 0,
            width: width.saturating_sub(gutter.saturating_mul(2)),
            height: primary_height,
        })
    }

    /// Geometry of the composer's editable text area (right of the `❯ ` gutter),
    /// in the same terminal-absolute frame as [`Self::from_app`]. `None` when the
    /// composer is off screen (approval owns the bottom) or there is no room.
    #[must_use]
    pub fn composer_from_app(app: &TuiApp) -> Option<Self> {
        let rect = crate::render::composer_text_area_rect(app)?;
        Some(Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        })
    }

    /// Whether a terminal-absolute cell falls inside this rect.
    #[must_use]
    pub fn contains(self, column: u16, row: u16) -> bool {
        column >= self.x
            && column < self.x.saturating_add(self.width)
            && row >= self.y
            && row < self.y.saturating_add(self.height)
    }

    /// Map terminal-absolute coords into content-local **display** coords.
    #[must_use]
    pub fn to_local(self, column: u16, row: u16) -> Option<(u16, u16)> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        if row < self.y || column < self.x {
            return Some((0, 0));
        }
        let local_col = column
            .saturating_sub(self.x)
            .min(self.width.saturating_sub(1));
        let local_row = row
            .saturating_sub(self.y)
            .min(self.height.saturating_sub(1));
        Some((local_col, local_row))
    }
}

/// Build every wrapped transcript row in document order, before viewport
/// scrolling. Absolute transcript selections extract from this stable source.
#[must_use]
pub fn full_transcript_plain_lines(app: &TuiApp) -> Vec<String> {
    let Some(geo) = ContentGeometry::from_app(app) else {
        return Vec::new();
    };
    crate::render::full_transcript_plain_lines_for_selection(app, geo.width)
}

/// Build the plain-text lines currently painted in the transcript content area.
/// Delegates to the same wrap → pad → scroll pipeline as [`crate::render`].
#[must_use]
pub fn visible_transcript_plain_lines(app: &TuiApp) -> Vec<String> {
    let Some(geo) = ContentGeometry::from_app(app) else {
        return Vec::new();
    };
    crate::render::visible_transcript_plain_lines_for_selection(app, geo.width, geo.height)
}

/// Absolute wrapped-row index of the first transcript row currently visible.
#[must_use]
pub fn transcript_viewport_start(app: &TuiApp, content: &ContentGeometry) -> usize {
    let total = crate::render::full_transcript_plain_lines_for_selection(app, content.width).len();
    crate::render::transcript_visible_start(
        total,
        usize::from(content.height),
        app.transcript.scroll_offset,
    )
}

/// The composer's editable lines exactly as painted (logical line == screen row;
/// wrapping is off in the composer). Empty when the composer is off screen.
#[must_use]
pub fn visible_composer_plain_lines(app: &TuiApp) -> Vec<String> {
    if crate::render::composer_text_area_rect(app).is_none() {
        return Vec::new();
    }
    app.composer.lines()
}

/// Extract the committed selection's text from whichever surface the gesture
/// began in. This is the entry point the app uses on copy: it dispatches on
/// [`SelectionState::region`] so a composer drag never pulls transcript text and
/// vice versa.
#[must_use]
pub fn extract_selection(app: &TuiApp, range: SelectionRange, region: SelectionRegion) -> String {
    let lines = match region {
        SelectionRegion::Transcript => full_transcript_plain_lines(app),
        SelectionRegion::Composer => visible_composer_plain_lines(app),
    };
    extract_range_from_lines(&lines, range)
}

/// Extract selected transcript text from an absolute wrapped-row range.
#[must_use]
pub fn extract_selected_text(app: &TuiApp, range: SelectionRange) -> String {
    let lines = full_transcript_plain_lines(app);
    extract_range_from_lines(&lines, range)
}

/// Shared display-column slicing over already-painted lines. Used by both the
/// transcript and composer extractors so their selection semantics stay identical.
#[must_use]
fn extract_range_from_lines(lines: &[String], range: SelectionRange) -> String {
    let n = range.normalized();
    if lines.is_empty() {
        return String::new();
    }
    let start_row = usize::from(n.start_row).min(lines.len().saturating_sub(1));
    let end_row = usize::from(n.end_row).min(lines.len().saturating_sub(1));
    let mut parts = Vec::new();
    for (row_idx, line) in lines.iter().enumerate().take(end_row + 1).skip(start_row) {
        let (from_col, to_col) = if start_row == end_row {
            let from = usize::from(n.start_col);
            let to = usize::from(n.end_col).max(from.saturating_add(1));
            (from, to)
        } else if row_idx == start_row {
            (usize::from(n.start_col), usize::MAX)
        } else if row_idx == end_row {
            (0, usize::from(n.end_col))
        } else {
            (0, usize::MAX)
        };
        let slice = slice_by_display_cols(line, from_col, to_col);
        // Drop background padding spaces so clipboard text matches what users
        // perceive as content (not the full strip fill).
        let slice = slice.trim_end().to_string();
        if !slice.is_empty() || start_row != end_row {
            parts.push(slice);
        }
    }
    parts.join("\n")
}

/// Slice `line` by display-column range `[from_col, to_col)`.
///
/// Wide glyphs (CJK, emoji) count as 2 columns so selection matches terminal
/// cursor geometry instead of UTF-8 char indices.
#[must_use]
pub fn slice_by_display_cols(line: &str, from_col: usize, to_col: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    for ch in line.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        let next = col.saturating_add(w);
        if next <= from_col {
            col = next;
            continue;
        }
        if col >= to_col {
            break;
        }
        // Include glyph if it overlaps the selected display range.
        out.push(ch);
        col = next;
        if col >= to_col {
            break;
        }
    }
    out
}

#[must_use]
pub fn line_display_width(line: &str) -> usize {
    UnicodeWidthStr::width(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_slice_handles_cjk_double_width() {
        // "你好世界" is 8 display columns (4×2). Selecting cols 0..4 is "你好".
        let line = "你好世界";
        assert_eq!(line_display_width(line), 8);
        assert_eq!(slice_by_display_cols(line, 0, 4), "你好");
        assert_eq!(slice_by_display_cols(line, 2, 6), "好世");
        assert_eq!(slice_by_display_cols(line, 0, 8), "你好世界");
    }

    #[test]
    fn display_slice_mixed_ascii_and_cjk() {
        // "ab中文cd" → widths 1 1 2 2 1 1 = 8
        let line = "ab中文cd";
        assert_eq!(slice_by_display_cols(line, 0, 2), "ab");
        assert_eq!(slice_by_display_cols(line, 2, 6), "中文");
        assert_eq!(slice_by_display_cols(line, 6, 8), "cd");
    }

    #[test]
    fn extract_matches_rendered_cjk_line() {
        use crate::{StatusBarState, TranscriptItem, TranscriptItemKind, TranscriptState, TuiApp};

        let mut app = TuiApp::new(StatusBarState::default());
        app.terminal_size.width = 40;
        app.terminal_size.height = 20;
        app.transcript = TranscriptState::default();
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::Model,
            "assistant",
            "你好世界ABCD",
            false,
        ));

        // Display: 你(0-1) 好(2-3) 世(4-5) 界(6-7) A(8) B(9) C(10) D(11)
        // Select display cols [2, 9) → "好世界A" (exclusive end past A)
        let lines = visible_transcript_plain_lines(&app);
        assert!(
            !lines.is_empty(),
            "expected painted transcript lines, got none"
        );
        let body_row = lines
            .iter()
            .position(|l| l.contains("你好世界"))
            .expect("body line with CJK text");
        let range = SelectionRange {
            start_col: 2,
            start_row: body_row as u16,
            end_col: 9,
            end_row: body_row as u16,
        };
        let text = extract_selected_text(&app, range);
        assert_eq!(text, "好世界A", "lines={lines:?} got={text:?}");
    }

    #[test]
    fn selection_survives_end_drag_without_clear() {
        let mut state = SelectionState::default();
        let geo = ContentGeometry {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };
        state.begin_drag(2, 1, SelectionRegion::Transcript, &geo, 0);
        state.update_drag(8, 1);
        assert!(state.end_drag(&geo, 0));
        assert!(state.committed.is_some());
        // Highlight must remain until next gesture.
        assert!(state.paint_transcript_range(&geo, 0).is_some());
    }

    #[test]
    fn transcript_drag_can_scroll_across_viewports_and_copy_full_range() {
        use crate::{
            StatusBarState, TranscriptItem, TranscriptItemKind, TranscriptState, TuiAction, TuiApp,
            TuiEffect,
        };

        let mut app = TuiApp::new(StatusBarState::default());
        app.terminal_size.width = 40;
        app.terminal_size.height = 12;
        app.markdown_enabled = false;
        app.transcript = TranscriptState::default();
        let body = (0..40)
            .map(|index| format!("line-{index:03} content"))
            .collect::<Vec<_>>()
            .join("\n");
        app.transcript.push(TranscriptItem::new(
            TranscriptItemKind::Model,
            "assistant",
            body,
            false,
        ));

        let geo = ContentGeometry::from_app(&app).expect("transcript geometry");
        app.reduce(TuiAction::SelectionDragStart {
            column: geo.x + 8,
            row: geo.y + geo.height - 1,
        });
        app.reduce(TuiAction::ScrollTranscriptPageUp);
        assert!(
            app.selection.drag_anchor.is_some(),
            "scrolling must keep the in-progress selection"
        );
        app.reduce(TuiAction::SelectionDragUpdate {
            column: geo.x + 4,
            row: geo.y,
        });
        let effect = app.reduce(TuiAction::SelectionDragEnd {
            column: geo.x + 4,
            row: geo.y,
        });

        let TuiEffect::CopyToClipboard { text } = effect else {
            panic!("cross-screen drag should copy on release");
        };
        assert!(
            text.lines().count() > usize::from(geo.height),
            "selection must include off-screen rows: {text:?}"
        );
        assert!(text.contains("line-028"), "earlier page missing: {text:?}");
        assert!(text.contains("line-039"), "anchor page missing: {text:?}");

        app.reduce(TuiAction::ScrollTranscriptBottom);
        let viewport_start = transcript_viewport_start(&app, &geo);
        assert!(
            app.selection
                .paint_transcript_range(&geo, viewport_start)
                .is_some(),
            "committed highlight must reappear when its rows are visible"
        );
    }
}
