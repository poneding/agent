//! In-app transcript text selection.
//!
//! Mouse capture steals native terminal drag-select. This module restores a
//! usable select+copy path inside the app:
//!
//! - **Click** (no drag) expands/collapses thinking/tools via hit-test.
//! - **Drag** paints a selection highlight and, on release, copies the selected
//!   plain text while **keeping** the highlight until the next gesture.
//! - Coordinates use **display columns** (Unicode width), not UTF-8 char counts,
//!   so CJK selection matches what the cursor paints.
//! - **Shift+click/drag** is left to the host terminal when supported.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::TuiApp;

/// Inclusive-start / exclusive-end range in the visible transcript content area
/// using **display columns** (origin at content top-left).
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionState {
    /// Anchor of an in-progress drag, in terminal-absolute coordinates.
    pub drag_anchor: Option<(u16, u16)>,
    /// Current pointer position while dragging (terminal-absolute).
    pub drag_current: Option<(u16, u16)>,
    /// Committed selection after mouse-up, in content-local display columns.
    /// Cleared on the next click / scroll / transcript mutation — **not** on
    /// successful copy, so the highlight remains visible after release.
    pub committed: Option<SelectionRange>,
    /// True once the pointer has moved from the anchor — distinguishes click
    /// from drag on mouse-up.
    pub moved: bool,
}

impl SelectionState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn begin_drag(&mut self, column: u16, row: u16) {
        self.drag_anchor = Some((column, row));
        self.drag_current = Some((column, row));
        self.moved = false;
        self.committed = None;
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
    /// copy), `false` if it was a click (caller should hit-test toggle).
    pub fn end_drag(&mut self, content: &ContentGeometry) -> bool {
        let Some(anchor) = self.drag_anchor.take() else {
            return false;
        };
        let current = self.drag_current.take().unwrap_or(anchor);
        let was_drag = self.moved && anchor != current;
        self.moved = false;
        if was_drag {
            if let (Some(a), Some(b)) = (
                content.to_local(anchor.0, anchor.1),
                content.to_local(current.0, current.1),
            ) {
                // Mouse coordinates are inclusive of the cell under the cursor.
                // Store an exclusive end column so extract/highlight match the
                // painted range (and include the final cell).
                let end_col = b.0.saturating_add(1).min(content.width);
                self.committed = Some(
                    SelectionRange {
                        start_col: a.0,
                        start_row: a.1,
                        end_col,
                        end_row: b.1,
                    }
                    .normalized(),
                );
            }
            true
        } else {
            self.committed = None;
            false
        }
    }

    /// Active range for painting: live drag if any, else committed.
    #[must_use]
    pub fn paint_range(&self, content: &ContentGeometry) -> Option<SelectionRange> {
        if let (Some(anchor), Some(current)) = (self.drag_anchor, self.drag_current) {
            if !self.moved {
                return None;
            }
            let a = content.to_local(anchor.0, anchor.1)?;
            let b = content.to_local(current.0, current.1)?;
            let end_col = b.0.saturating_add(1).min(content.width);
            return Some(
                SelectionRange {
                    start_col: a.0,
                    start_row: a.1,
                    end_col,
                    end_row: b.1,
                }
                .normalized(),
            );
        }
        self.committed
    }
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
        let activity_height: u16 = if app.turn_state().is_active() { 2 } else { 0 };
        let text_rows = u16::try_from(app.composer.line_count())
            .unwrap_or(6)
            .clamp(1, 6);
        let composer_height = text_rows.saturating_add(2);
        let reserved = activity_height
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

/// Build the plain-text lines currently painted in the transcript content area.
/// Delegates to the same wrap → pad → scroll pipeline as [`crate::render`].
#[must_use]
pub fn visible_transcript_plain_lines(app: &TuiApp) -> Vec<String> {
    let Some(geo) = ContentGeometry::from_app(app) else {
        return Vec::new();
    };
    crate::render::visible_transcript_plain_lines_for_selection(app, geo.width, geo.height)
}

/// Extract selected plain text for a content-local range using **display columns**.
#[must_use]
pub fn extract_selected_text(app: &TuiApp, range: SelectionRange) -> String {
    let lines = visible_transcript_plain_lines(app);
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
        state.begin_drag(2, 1);
        state.update_drag(8, 1);
        let geo = ContentGeometry {
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        };
        assert!(state.end_drag(&geo));
        assert!(state.committed.is_some());
        // Highlight must remain until next gesture.
        assert!(state.paint_range(&geo).is_some());
    }
}
