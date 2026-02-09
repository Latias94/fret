//! View-layer building blocks for the code editor ecosystem.
//!
//! v1 is intentionally minimal: "display rows" are logical lines split by `\n` and columns are
//! counted as Unicode scalar values (not graphemes, not rendered cells).
//!
//! See ADR 0200 for the normative buffer/view/surface split and v1 rollout constraints.

use fret_code_editor_buffer::TextBuffer;
use fret_runtime::TextBoundaryMode;
use fret_text_nav as text_nav;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

mod folds;
mod inlays;

pub use folds::{
    FoldSpan, FoldSpanError, apply_fold_spans, folded_byte_to_col, folded_col_count,
    folded_col_to_byte, validate_fold_spans,
};
pub use inlays::{InlaySpan, InlaySpanError, apply_inlay_spans, validate_inlay_spans};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlinePreedit {
    /// Anchor byte index in the underlying buffer.
    pub anchor: usize,
    /// Inline preedit text to be composed into the display stream.
    pub text: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayRowFragment {
    Buffer { range: Range<usize> },
    Placeholder { text: Arc<str>, maps_to: usize },
    Inlay { text: Arc<str>, maps_to: usize },
    Preedit { text: Arc<str>, maps_to: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayRowFragmentsError {
    UnsupportedWithWrap,
    InvalidFoldSpans(FoldSpanError),
    InvalidInlaySpans(InlaySpanError),
    MissingLineText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayPoint {
    pub row: usize,
    pub col: usize,
}

impl DisplayPoint {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// A minimal display mapping for v1 editor surfaces.
///
/// Today this only supports an optional "wrap after N Unicode scalar columns" mode. This is not a
/// substitute for pixel-accurate wrapping, but it provides a stable contract surface for:
///
/// - caret movement (byte ↔ display point),
/// - selection geometry,
/// - future display-map expansion (wrap/fold/inlays).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayMap {
    wrap_cols: Option<usize>,
    line_to_first_row: Vec<usize>,
    row_to_line: Vec<usize>,
    row_start_col: Vec<usize>,
    line_folds: Vec<Arc<[FoldSpan]>>,
    line_inlays: Vec<Arc<[InlaySpan]>>,
    preedit: Option<InlinePreedit>,
}

impl DisplayMap {
    /// Build a display map from the current buffer state.
    ///
    /// `wrap_cols` counts Unicode scalar values within a logical line (newline excluded).
    /// When `None`, display rows match logical lines.
    pub fn new(buf: &TextBuffer, wrap_cols: Option<usize>) -> Self {
        let empty_folds: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        let empty_inlays: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        Self::new_with_decorations(buf, wrap_cols, &empty_folds, &empty_inlays)
    }

    /// Build a display map from the current buffer state, including view-layer fold/inlay
    /// decorations (ADR 0200).
    ///
    /// v1 notes:
    /// - Folds and inlays participate in wrapped row-breaking.
    /// - Inlays whose insertion point lands inside a folded span are ignored.
    /// - Fold placeholders and inlay text are treated as atomic units for wrapping (no split).
    pub fn new_with_decorations(
        buf: &TextBuffer,
        wrap_cols: Option<usize>,
        folds_by_line: &HashMap<usize, Arc<[FoldSpan]>>,
        inlays_by_line: &HashMap<usize, Arc<[InlaySpan]>>,
    ) -> Self {
        Self::new_with_decorations_and_preedit(buf, wrap_cols, folds_by_line, inlays_by_line, None)
    }

    /// Build a display map from the current buffer state, including view-layer fold/inlay
    /// decorations and an optional inline preedit insertion (ADR 0203).
    ///
    /// Preedit is treated as an atomic insertion fragment and participates in wrapped row-breaking.
    pub fn new_with_decorations_and_preedit(
        buf: &TextBuffer,
        wrap_cols: Option<usize>,
        folds_by_line: &HashMap<usize, Arc<[FoldSpan]>>,
        inlays_by_line: &HashMap<usize, Arc<[InlaySpan]>>,
        preedit: Option<InlinePreedit>,
    ) -> Self {
        let wrap_cols = wrap_cols.filter(|v| *v > 0);

        let line_count = buf.line_count().max(1);

        let mut line_folds: Vec<Arc<[FoldSpan]>> = Vec::with_capacity(line_count);
        let mut line_inlays: Vec<Arc<[InlaySpan]>> = Vec::with_capacity(line_count);
        for line in 0..line_count {
            line_folds.push(
                folds_by_line
                    .get(&line)
                    .cloned()
                    .unwrap_or_else(|| Arc::<[FoldSpan]>::from([])),
            );
            line_inlays.push(
                inlays_by_line
                    .get(&line)
                    .cloned()
                    .unwrap_or_else(|| Arc::<[InlaySpan]>::from([])),
            );
        }

        if wrap_cols.is_none() {
            let mut line_to_first_row = Vec::with_capacity(line_count);
            let mut row_to_line = Vec::with_capacity(line_count);
            let mut row_start_col = Vec::with_capacity(line_count);
            for line in 0..line_count {
                line_to_first_row.push(line);
                row_to_line.push(line);
                row_start_col.push(0);
            }

            return Self {
                wrap_cols,
                line_to_first_row,
                row_to_line,
                row_start_col,
                line_folds,
                line_inlays,
                preedit,
            };
        }

        let mut line_to_first_row = Vec::with_capacity(line_count);
        let mut row_to_line = Vec::new();
        let mut row_start_col = Vec::new();

        let wrap = wrap_cols.unwrap_or(usize::MAX).max(1);

        for line in 0..line_count {
            line_to_first_row.push(row_to_line.len());

            let folds = line_folds.get(line).map(|v| v.as_ref()).unwrap_or(&[]);
            let inlays = line_inlays.get(line).map(|v| v.as_ref()).unwrap_or(&[]);

            let line_has_preedit = preedit.as_ref().is_some_and(|p| {
                !p.text.is_empty() && buf.line_index_at_byte(p.anchor.min(buf.len_bytes())) == line
            });

            if folds.is_empty() && inlays.is_empty() && !line_has_preedit {
                let cols = buf.line_char_count(line);
                let rows_for_line = ((cols.max(1) + wrap - 1) / wrap).max(1);
                for row_in_line in 0..rows_for_line {
                    row_to_line.push(line);
                    row_start_col.push(row_in_line * wrap);
                }
                continue;
            }

            let line_text_owned = buf.line_text(line).unwrap_or_default();
            let line_text = line_text_owned.as_str();
            let folds = validate_fold_spans(line_text, folds)
                .is_ok()
                .then_some(folds)
                .unwrap_or(&[]);
            let inlays = validate_inlay_spans(line_text, inlays)
                .is_ok()
                .then_some(inlays)
                .unwrap_or(&[]);

            let preedit_for_line =
                inline_preedit_for_line(buf, line, line_text, folds, preedit.as_ref());
            let starts =
                compute_wrapped_row_start_cols(line_text, folds, inlays, preedit_for_line, wrap);
            for start in starts {
                row_to_line.push(line);
                row_start_col.push(start);
            }
        }

        if row_to_line.is_empty() {
            row_to_line.push(0);
            row_start_col.push(0);
        }

        Self {
            wrap_cols,
            line_to_first_row,
            row_to_line,
            row_start_col,
            line_folds,
            line_inlays,
            preedit,
        }
    }

    pub fn row_count(&self) -> usize {
        self.row_to_line.len().max(1)
    }

    pub fn wrap_cols(&self) -> Option<usize> {
        self.wrap_cols
    }

    /// Return the first display-row index for a logical line.
    ///
    /// When wrapping is disabled, this is equal to `line`.
    pub fn line_first_display_row(&self, line: usize) -> usize {
        if self.line_to_first_row.is_empty() {
            return 0;
        }
        let line = line.min(self.line_to_first_row.len().saturating_sub(1));
        *self.line_to_first_row.get(line).unwrap_or(&0)
    }

    /// Return the display-row range that corresponds to a single logical line.
    ///
    /// When wrapping is disabled, this is always `line..(line + 1)` (clamped to the display-row
    /// count).
    pub fn line_display_row_range(&self, line: usize) -> Range<usize> {
        if self.line_to_first_row.is_empty() || self.row_to_line.is_empty() {
            return 0..0;
        }
        let line = line.min(self.line_to_first_row.len().saturating_sub(1));
        let start = self.line_first_display_row(line);
        let end = self
            .line_to_first_row
            .get(line + 1)
            .copied()
            .unwrap_or_else(|| self.row_to_line.len());
        start..end.max(start)
    }

    pub fn display_row_line(&self, display_row: usize) -> usize {
        if self.row_to_line.is_empty() {
            return 0;
        }
        let row = display_row.min(self.row_to_line.len().saturating_sub(1));
        self.row_to_line[row]
    }

    pub fn display_row_byte_range(&self, buf: &TextBuffer, display_row: usize) -> Range<usize> {
        if self.row_to_line.is_empty() {
            return 0..0;
        }

        let row = display_row.min(self.row_to_line.len().saturating_sub(1));
        let line = self.row_to_line[row];
        let Some(line_range) = buf.line_byte_range(line) else {
            return buf.len_bytes()..buf.len_bytes();
        };

        let start = self.display_point_to_byte(buf, DisplayPoint::new(row, 0));
        let start = start.min(line_range.end).max(line_range.start);

        let end = match self.wrap_cols {
            None => line_range.end,
            Some(_) => {
                if row.saturating_add(1) < self.row_to_line.len()
                    && self.row_to_line[row + 1] == line
                {
                    let next = self.display_point_to_byte(buf, DisplayPoint::new(row + 1, 0));
                    next.min(line_range.end).max(start)
                } else {
                    line_range.end.max(start)
                }
            }
        };

        start..end
    }

    /// Return display-row text fragments for unwrapped (1 row == 1 logical line) mapping.
    ///
    /// This is a view-layer contract surface for fold/placeholder expansion (ADR 0200). It is
    /// intentionally restricted to the unwrapped baseline until we define the combined wrap+fold
    /// semantics (avoid partially materializing placeholders across wrapped rows).
    pub fn display_row_fragments_unwrapped(
        &self,
        buf: &TextBuffer,
        display_row: usize,
        folds: &[FoldSpan],
    ) -> Result<Vec<DisplayRowFragment>, DisplayRowFragmentsError> {
        if self.wrap_cols.is_some() {
            return Err(DisplayRowFragmentsError::UnsupportedWithWrap);
        }

        if self.row_to_line.is_empty() {
            return Ok(Vec::new());
        }

        let row = display_row.min(self.row_to_line.len().saturating_sub(1));
        let line = self.row_to_line[row];
        let Some(line_range) = buf.line_byte_range(line) else {
            return Ok(Vec::new());
        };

        let Some(line_text) = buf.line_text(line) else {
            return Err(DisplayRowFragmentsError::MissingLineText);
        };
        validate_fold_spans(&line_text, folds)
            .map_err(DisplayRowFragmentsError::InvalidFoldSpans)?;

        let mut out = Vec::<DisplayRowFragment>::new();
        let mut cursor = line_range.start;
        for span in folds {
            let start = line_range
                .start
                .saturating_add(span.range.start)
                .min(line_range.end);
            let end = line_range
                .start
                .saturating_add(span.range.end)
                .min(line_range.end);
            if cursor < start {
                out.push(DisplayRowFragment::Buffer {
                    range: cursor..start,
                });
            }
            out.push(DisplayRowFragment::Placeholder {
                text: Arc::clone(&span.placeholder),
                maps_to: start,
            });
            cursor = end.max(start);
        }
        if cursor < line_range.end {
            out.push(DisplayRowFragment::Buffer {
                range: cursor..line_range.end,
            });
        }
        Ok(out)
    }

    /// Return display-row text fragments for unwrapped mapping, including fold placeholders and
    /// injected inlay text (ADR 0200).
    ///
    /// This is intentionally restricted to the unwrapped baseline until we define the combined
    /// wrap+fold/inlay semantics.
    pub fn display_row_fragments_unwrapped_with_inlays(
        &self,
        buf: &TextBuffer,
        display_row: usize,
        folds: &[FoldSpan],
        inlays: &[InlaySpan],
    ) -> Result<Vec<DisplayRowFragment>, DisplayRowFragmentsError> {
        if self.wrap_cols.is_some() {
            return Err(DisplayRowFragmentsError::UnsupportedWithWrap);
        }

        if self.row_to_line.is_empty() {
            return Ok(Vec::new());
        }

        let row = display_row.min(self.row_to_line.len().saturating_sub(1));
        let line = self.row_to_line[row];
        let Some(line_range) = buf.line_byte_range(line) else {
            return Ok(Vec::new());
        };

        let Some(line_text) = buf.line_text(line) else {
            return Err(DisplayRowFragmentsError::MissingLineText);
        };
        validate_fold_spans(&line_text, folds)
            .map_err(DisplayRowFragmentsError::InvalidFoldSpans)?;
        validate_inlay_spans(&line_text, inlays)
            .map_err(DisplayRowFragmentsError::InvalidInlaySpans)?;

        let mut out = Vec::<DisplayRowFragment>::new();

        let mut fold_idx = 0usize;
        let mut inlay_idx = 0usize;
        let mut cursor = 0usize;

        while cursor < line_text.len() || fold_idx < folds.len() || inlay_idx < inlays.len() {
            let next_fold = folds.get(fold_idx).map(|s| s.range.start);
            let next_inlay = inlays.get(inlay_idx).map(|s| s.byte);
            let next = match (next_fold, next_inlay) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
            .unwrap_or(line_text.len());

            if cursor < next {
                out.push(DisplayRowFragment::Buffer {
                    range: line_range.start.saturating_add(cursor)
                        ..line_range.start.saturating_add(next),
                });
                cursor = next;
                continue;
            }

            if let Some(inlay) = inlays.get(inlay_idx)
                && inlay.byte == cursor
            {
                out.push(DisplayRowFragment::Inlay {
                    text: Arc::clone(&inlay.text),
                    maps_to: line_range.start.saturating_add(inlay.byte),
                });
                inlay_idx = inlay_idx.saturating_add(1);
                continue;
            }

            if let Some(fold) = folds.get(fold_idx)
                && fold.range.start == cursor
            {
                let start = cursor;
                let end = fold.range.end.min(line_text.len()).max(start);
                out.push(DisplayRowFragment::Placeholder {
                    text: Arc::clone(&fold.placeholder),
                    maps_to: line_range.start.saturating_add(start),
                });
                cursor = end;
                fold_idx = fold_idx.saturating_add(1);
                continue;
            }

            // If we reach here, we failed to make progress (should be unreachable after validation).
            break;
        }

        Ok(out)
    }

    /// Map a UTF-8 byte index in the buffer to a wrapped display coordinate.
    pub fn byte_to_display_point(&self, buf: &TextBuffer, byte: usize) -> DisplayPoint {
        let pt = byte_to_display_point(buf, byte);
        if self.line_to_first_row.is_empty() {
            return DisplayPoint::new(0, 0);
        }

        let line = pt.row.min(self.line_to_first_row.len().saturating_sub(1));
        let line_first = self.line_first_display_row(line);
        let folds_empty = self
            .line_folds
            .get(line)
            .is_some_and(|v| v.as_ref().is_empty());
        let inlays_empty = self
            .line_inlays
            .get(line)
            .is_some_and(|v| v.as_ref().is_empty());
        let has_preedit = self.preedit.as_ref().is_some_and(|p| {
            !p.text.is_empty() && buf.line_index_at_byte(p.anchor.min(buf.len_bytes())) == line
        });

        match self.wrap_cols {
            None => {
                if folds_empty && inlays_empty && !has_preedit {
                    return pt;
                }

                let line_start = buf.line_start(line).unwrap_or(0);
                let line_text_owned = buf.line_text(line).unwrap_or_default();
                let line_text = line_text_owned.as_str();
                let byte = byte.min(buf.len_bytes());
                let line_local = byte.saturating_sub(line_start).min(line_text.len());

                let folds = self.line_folds.get(line).map(|v| v.as_ref()).unwrap_or(&[]);
                let inlays = self
                    .line_inlays
                    .get(line)
                    .map(|v| v.as_ref())
                    .unwrap_or(&[]);
                let folds = validate_fold_spans(line_text, folds)
                    .is_ok()
                    .then_some(folds)
                    .unwrap_or(&[]);
                let inlays = validate_inlay_spans(line_text, inlays)
                    .is_ok()
                    .then_some(inlays)
                    .unwrap_or(&[]);

                let preedit_for_line =
                    inline_preedit_for_line(buf, line, line_text, folds, self.preedit.as_ref());
                let col_in_line = decorated_byte_to_col_with_preedit(
                    line_text,
                    folds,
                    inlays,
                    preedit_for_line,
                    line_local,
                );
                DisplayPoint::new(line, col_in_line)
            }
            Some(wrap) => {
                let line_last_excl = self
                    .line_to_first_row
                    .get(line + 1)
                    .copied()
                    .unwrap_or_else(|| self.row_to_line.len());
                let rows_for_line = line_last_excl.saturating_sub(line_first).max(1);

                if folds_empty && inlays_empty && !has_preedit {
                    let row_in_line = (pt.col / wrap).min(rows_for_line.saturating_sub(1));
                    let col_in_row = pt.col.saturating_sub(row_in_line * wrap);
                    return DisplayPoint::new(line_first + row_in_line, col_in_row);
                }

                let line_start = buf.line_start(line).unwrap_or(0);
                let line_text_owned = buf.line_text(line).unwrap_or_default();
                let line_text = line_text_owned.as_str();
                let byte = byte.min(buf.len_bytes());
                let line_local = byte.saturating_sub(line_start).min(line_text.len());

                let folds = self.line_folds.get(line).map(|v| v.as_ref()).unwrap_or(&[]);
                let inlays = self
                    .line_inlays
                    .get(line)
                    .map(|v| v.as_ref())
                    .unwrap_or(&[]);
                let folds = validate_fold_spans(line_text, folds)
                    .is_ok()
                    .then_some(folds)
                    .unwrap_or(&[]);
                let inlays = validate_inlay_spans(line_text, inlays)
                    .is_ok()
                    .then_some(inlays)
                    .unwrap_or(&[]);

                let preedit_for_line =
                    inline_preedit_for_line(buf, line, line_text, folds, self.preedit.as_ref());
                let col_in_line = decorated_byte_to_col_with_preedit(
                    line_text,
                    folds,
                    inlays,
                    preedit_for_line,
                    line_local,
                );
                let rows = self.line_display_row_range(line);
                let row_in_line =
                    find_row_for_line_col(&self.row_start_col[rows.clone()], col_in_line)
                        .min(rows_for_line.saturating_sub(1));
                let display_row = rows.start.saturating_add(row_in_line);
                let row_start_col = *self.row_start_col.get(display_row).unwrap_or(&0);
                DisplayPoint::new(display_row, col_in_line.saturating_sub(row_start_col))
            }
        }
    }

    /// Map a wrapped display coordinate to a UTF-8 byte index in the buffer.
    ///
    /// If the point is out of bounds, this clamps to the nearest representable position.
    pub fn display_point_to_byte(&self, buf: &TextBuffer, mut pt: DisplayPoint) -> usize {
        if self.row_to_line.is_empty() {
            return 0;
        }
        pt.row = pt.row.min(self.row_to_line.len().saturating_sub(1));
        let line = self.row_to_line[pt.row];
        let folds_empty = self
            .line_folds
            .get(line)
            .is_some_and(|v| v.as_ref().is_empty());
        let inlays_empty = self
            .line_inlays
            .get(line)
            .is_some_and(|v| v.as_ref().is_empty());
        let has_preedit = self.preedit.as_ref().is_some_and(|p| {
            !p.text.is_empty() && buf.line_index_at_byte(p.anchor.min(buf.len_bytes())) == line
        });

        if self.wrap_cols.is_none() && folds_empty && inlays_empty && !has_preedit {
            return display_point_to_byte(buf, DisplayPoint::new(line, pt.col));
        }

        let row_start_col = *self.row_start_col.get(pt.row).unwrap_or(&0);
        let col_in_line = row_start_col.saturating_add(pt.col);

        if folds_empty && inlays_empty && !has_preedit {
            return buf.byte_at_line_col(line, col_in_line);
        }

        let line_start = buf.line_start(line).unwrap_or(0);
        let line_text_owned = buf.line_text(line).unwrap_or_default();
        let line_text = line_text_owned.as_str();

        let folds = self.line_folds.get(line).map(|v| v.as_ref()).unwrap_or(&[]);
        let inlays = self
            .line_inlays
            .get(line)
            .map(|v| v.as_ref())
            .unwrap_or(&[]);
        let folds = validate_fold_spans(line_text, folds)
            .is_ok()
            .then_some(folds)
            .unwrap_or(&[]);
        let inlays = validate_inlay_spans(line_text, inlays)
            .is_ok()
            .then_some(inlays)
            .unwrap_or(&[]);

        let preedit_for_line =
            inline_preedit_for_line(buf, line, line_text, folds, self.preedit.as_ref());
        let byte_in_line = decorated_col_to_byte_with_preedit(
            line_text,
            folds,
            inlays,
            preedit_for_line,
            col_in_line,
        );
        line_start.saturating_add(byte_in_line).min(buf.len_bytes())
    }
}

fn find_row_for_line_col(row_start_cols: &[usize], col_in_line: usize) -> usize {
    // Upper bound, then step back.
    let mut lo = 0usize;
    let mut hi = row_start_cols.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if row_start_cols[mid] <= col_in_line {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo.saturating_sub(1)
}

fn compute_wrapped_row_start_cols(
    line_text: &str,
    folds: &[FoldSpan],
    inlays: &[InlaySpan],
    preedit: Option<(usize, usize)>,
    wrap_cols: usize,
) -> Vec<usize> {
    let wrap_cols = wrap_cols.max(1);
    let mut starts = vec![0usize];

    let mut col = 0usize;
    let mut in_row = 0usize;

    let mut cursor = 0usize;
    let mut fold_idx = 0usize;
    let mut inlay_idx = 0usize;
    let mut preedit = preedit.filter(|(_, cols)| *cols > 0);

    while cursor < line_text.len()
        || fold_idx < folds.len()
        || inlay_idx < inlays.len()
        || preedit.is_some()
    {
        if let Some((anchor, token_cols)) = preedit
            && anchor == cursor
        {
            // Atomic: never split preedit text across wrapped rows (ADR 0203).
            let remaining = wrap_cols.saturating_sub(in_row);
            if in_row > 0 && token_cols > remaining {
                starts.push(col);
                in_row = 0;
            }

            col = col.saturating_add(token_cols);
            if token_cols >= wrap_cols {
                // Overflow consumes the row; next token starts a fresh row.
                starts.push(col);
                in_row = 0;
            } else {
                in_row = in_row.saturating_add(token_cols);
                if in_row == wrap_cols {
                    starts.push(col);
                    in_row = 0;
                }
            }

            preedit = None;
            continue;
        }

        while inlay_idx < inlays.len() && inlays[inlay_idx].byte < cursor {
            // Inlays inside a folded span are skipped by advancing past the fold jump.
            inlay_idx = inlay_idx.saturating_add(1);
        }

        let next_fold = folds.get(fold_idx).map(|s| s.range.start);
        let next_inlay = inlays.get(inlay_idx).map(|s| s.byte);
        let next = match (next_fold, next_inlay) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => line_text.len(),
        }
        .min(line_text.len());

        if cursor < next {
            // Buffer text is splittable at scalar boundaries.
            let segment = &line_text[cursor..next];
            let mut rem = segment.chars().count();
            while rem > 0 {
                let remaining = wrap_cols.saturating_sub(in_row);
                if remaining == 0 {
                    starts.push(col);
                    in_row = 0;
                    continue;
                }
                let take = remaining.min(rem);
                rem = rem.saturating_sub(take);
                col = col.saturating_add(take);
                in_row = in_row.saturating_add(take);
                if in_row == wrap_cols && rem > 0 {
                    starts.push(col);
                    in_row = 0;
                }
            }
            cursor = next;
            continue;
        }

        if let Some(inlay) = inlays.get(inlay_idx)
            && inlay.byte == cursor
        {
            // Atomic: never split inlay text across wrapped rows.
            let token_cols = inlay.text.chars().count();
            let remaining = wrap_cols.saturating_sub(in_row);
            if in_row > 0 && token_cols > remaining {
                starts.push(col);
                in_row = 0;
            }

            col = col.saturating_add(token_cols);
            if token_cols >= wrap_cols {
                // Overflow consumes the row; next token starts a fresh row.
                starts.push(col);
                in_row = 0;
            } else {
                in_row = in_row.saturating_add(token_cols);
                if in_row == wrap_cols {
                    starts.push(col);
                    in_row = 0;
                }
            }

            inlay_idx = inlay_idx.saturating_add(1);
            continue;
        }

        if let Some(fold) = folds.get(fold_idx)
            && fold.range.start == cursor
        {
            // Atomic: never split fold placeholders across wrapped rows.
            let token_cols = fold.placeholder.chars().count();
            let remaining = wrap_cols.saturating_sub(in_row);
            if in_row > 0 && token_cols > remaining {
                starts.push(col);
                in_row = 0;
            }

            col = col.saturating_add(token_cols);
            if token_cols >= wrap_cols {
                starts.push(col);
                in_row = 0;
            } else {
                in_row = in_row.saturating_add(token_cols);
                if in_row == wrap_cols {
                    starts.push(col);
                    in_row = 0;
                }
            }

            let start = cursor;
            let end = fold.range.end.min(line_text.len()).max(start);
            cursor = end;
            fold_idx = fold_idx.saturating_add(1);
            continue;
        }

        break;
    }

    // Remove the trailing empty row start if we ended exactly at a row boundary.
    if starts.len() > 1 && starts.last().is_some_and(|v| *v == col) {
        starts.pop();
    }
    if starts.is_empty() {
        starts.push(0);
    }
    starts
}

fn decorated_byte_to_col(
    line_text: &str,
    folds: &[FoldSpan],
    inlays: &[InlaySpan],
    byte: usize,
) -> usize {
    let byte = clamp_to_char_boundary(line_text, byte.min(line_text.len()));

    let mut col = 0usize;
    let mut cursor = 0usize;
    let mut fold_idx = 0usize;
    let mut inlay_idx = 0usize;

    while cursor <= line_text.len() {
        while inlay_idx < inlays.len() && inlays[inlay_idx].byte < cursor {
            inlay_idx = inlay_idx.saturating_add(1);
        }

        let next_fold = folds.get(fold_idx).map(|s| s.range.start);
        let next_inlay = inlays.get(inlay_idx).map(|s| s.byte);
        let next = match (next_fold, next_inlay) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => line_text.len(),
        }
        .min(line_text.len());

        if byte < next {
            return col.saturating_add(line_text[cursor..byte].chars().count());
        }

        if cursor < next {
            col = col.saturating_add(line_text[cursor..next].chars().count());
            cursor = next;
        }

        if let Some(inlay) = inlays.get(inlay_idx)
            && inlay.byte == cursor
        {
            if byte == cursor {
                return col;
            }
            col = col.saturating_add(inlay.text.chars().count());
            inlay_idx = inlay_idx.saturating_add(1);
            continue;
        }

        if let Some(fold) = folds.get(fold_idx)
            && fold.range.start == cursor
        {
            let start = cursor;
            let end = fold.range.end.min(line_text.len()).max(start);
            if byte < end {
                return col;
            }
            col = col.saturating_add(fold.placeholder.chars().count());
            cursor = end;
            fold_idx = fold_idx.saturating_add(1);
            continue;
        }

        if cursor >= line_text.len() {
            break;
        }

        cursor = (cursor + 1).min(line_text.len());
    }

    col
}

fn inline_preedit_for_line(
    buf: &TextBuffer,
    line: usize,
    line_text: &str,
    folds: &[FoldSpan],
    preedit: Option<&InlinePreedit>,
) -> Option<(usize, usize)> {
    let preedit = preedit?;
    if preedit.text.is_empty() {
        return None;
    }

    let anchor = preedit.anchor.min(buf.len_bytes());
    if buf.line_index_at_byte(anchor) != line {
        return None;
    }

    let line_start = buf.line_start(line).unwrap_or(0);
    let mut local = anchor.saturating_sub(line_start).min(line_text.len());
    local = clamp_to_char_boundary(line_text, local);

    // If the anchor lands inside a folded span, clamp to the fold start (ADR 0203).
    for fold in folds {
        let start = fold.range.start.min(line_text.len());
        let end = fold.range.end.min(line_text.len()).max(start);
        if start < local && local < end {
            local = start;
            break;
        }
    }

    let cols = preedit.text.chars().count();
    Some((local, cols))
}

fn decorated_byte_to_col_with_preedit(
    line_text: &str,
    folds: &[FoldSpan],
    inlays: &[InlaySpan],
    preedit: Option<(usize, usize)>,
    byte: usize,
) -> usize {
    let Some((anchor_local, preedit_cols)) = preedit else {
        return decorated_byte_to_col(line_text, folds, inlays, byte);
    };
    if preedit_cols == 0 {
        return decorated_byte_to_col(line_text, folds, inlays, byte);
    }

    let insert_col = decorated_byte_to_col(line_text, folds, inlays, anchor_local);
    let base_col = decorated_byte_to_col(line_text, folds, inlays, byte);
    if base_col > insert_col {
        base_col.saturating_add(preedit_cols)
    } else {
        base_col
    }
}

fn byte_offset_for_col(slice: &str, col: usize) -> usize {
    if col == 0 {
        return 0;
    }
    let mut remaining = col;
    for (i, _) in slice.char_indices() {
        if remaining == 0 {
            return i;
        }
        remaining = remaining.saturating_sub(1);
    }
    slice.len()
}

fn decorated_col_to_byte(
    line_text: &str,
    folds: &[FoldSpan],
    inlays: &[InlaySpan],
    col: usize,
) -> usize {
    let mut cursor = 0usize;
    let mut cursor_col = 0usize;
    let mut fold_idx = 0usize;
    let mut inlay_idx = 0usize;

    while cursor <= line_text.len() {
        while inlay_idx < inlays.len() && inlays[inlay_idx].byte < cursor {
            inlay_idx = inlay_idx.saturating_add(1);
        }

        let next_fold = folds.get(fold_idx).map(|s| s.range.start);
        let next_inlay = inlays.get(inlay_idx).map(|s| s.byte);
        let next = match (next_fold, next_inlay) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => line_text.len(),
        }
        .min(line_text.len());

        if cursor < next {
            let segment = &line_text[cursor..next];
            let seg_cols = segment.chars().count();
            if col < cursor_col.saturating_add(seg_cols) {
                let local_col = col.saturating_sub(cursor_col);
                let offset = byte_offset_for_col(segment, local_col);
                return cursor.saturating_add(offset).min(line_text.len());
            }
            cursor_col = cursor_col.saturating_add(seg_cols);
            cursor = next;
        }

        if let Some(inlay) = inlays.get(inlay_idx)
            && inlay.byte == cursor
        {
            let inlay_cols = inlay.text.chars().count();
            if col < cursor_col.saturating_add(inlay_cols) {
                return cursor;
            }
            cursor_col = cursor_col.saturating_add(inlay_cols);
            inlay_idx = inlay_idx.saturating_add(1);
            continue;
        }

        if let Some(fold) = folds.get(fold_idx)
            && fold.range.start == cursor
        {
            let placeholder_cols = fold.placeholder.chars().count();
            if col < cursor_col.saturating_add(placeholder_cols) {
                return cursor;
            }
            cursor_col = cursor_col.saturating_add(placeholder_cols);
            cursor = fold.range.end.min(line_text.len()).max(fold.range.start);
            fold_idx = fold_idx.saturating_add(1);
            continue;
        }

        break;
    }

    line_text.len()
}

fn decorated_col_to_byte_with_preedit(
    line_text: &str,
    folds: &[FoldSpan],
    inlays: &[InlaySpan],
    preedit: Option<(usize, usize)>,
    col: usize,
) -> usize {
    let Some((anchor_local, preedit_cols)) = preedit else {
        return decorated_col_to_byte(line_text, folds, inlays, col);
    };
    if preedit_cols == 0 {
        return decorated_col_to_byte(line_text, folds, inlays, col);
    }

    let insert_col = decorated_byte_to_col(line_text, folds, inlays, anchor_local);
    if col < insert_col {
        return decorated_col_to_byte(line_text, folds, inlays, col);
    }

    let after_insert = insert_col.saturating_add(preedit_cols);
    if col >= after_insert {
        return decorated_col_to_byte(line_text, folds, inlays, col.saturating_sub(preedit_cols));
    }

    // Inside the injected preedit fragment: snap to its anchor.
    decorated_col_to_byte(line_text, folds, inlays, insert_col)
}

/// Map a UTF-8 byte index in the buffer to a `(row, col)` display coordinate.
pub fn byte_to_display_point(buf: &TextBuffer, mut byte: usize) -> DisplayPoint {
    byte = byte.min(buf.len_bytes());
    let (row, col) = buf.line_col_at_byte(byte);
    DisplayPoint { row, col }
}

/// Map a `(row, col)` display coordinate to a UTF-8 byte index in the buffer.
///
/// If `col` is out of bounds for the row, this clamps to the row end (excluding the trailing
/// newline).
pub fn display_point_to_byte(buf: &TextBuffer, mut pt: DisplayPoint) -> usize {
    let line_count = buf.line_count().max(1);
    if line_count == 0 {
        return 0;
    }
    pt.row = pt.row.min(line_count.saturating_sub(1));
    buf.byte_at_line_col(pt.row, pt.col)
}

pub fn clamp_to_char_boundary(text: &str, idx: usize) -> usize {
    text_nav::clamp_to_char_boundary(text, idx)
}

pub fn prev_char_boundary(text: &str, idx: usize) -> usize {
    text_nav::prev_char_boundary(text, idx)
}

pub fn next_char_boundary(text: &str, idx: usize) -> usize {
    text_nav::next_char_boundary(text, idx)
}

pub fn select_word_range(text: &str, idx: usize, mode: TextBoundaryMode) -> (usize, usize) {
    text_nav::select_word_range(text, idx, mode)
}

pub fn select_line_range(text: &str, idx: usize) -> (usize, usize) {
    text_nav::select_line_range(text, idx)
}

pub fn move_word_left(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    text_nav::move_word_left(text, idx, mode)
}

pub fn move_word_right(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    text_nav::move_word_right(text, idx, mode)
}

/// Select a word range in a `TextBuffer` using v1 line-local semantics.
///
/// v1 operates on a single logical line slice (newline excluded). Crossing line boundaries is
/// handled by the caller (e.g. double-click on a newline maps to the nearest line).
pub fn select_word_range_in_buffer(
    buf: &TextBuffer,
    idx: usize,
    mode: TextBoundaryMode,
) -> (usize, usize) {
    if buf.is_empty() {
        return (0, 0);
    }
    let idx = buf.clamp_to_char_boundary_left(idx.min(buf.len_bytes()));
    let line = buf.line_index_at_byte(idx);
    let line_start = buf.line_start(line).unwrap_or(0);
    let line_text = buf.line_text(line).unwrap_or_default();
    let local = idx.saturating_sub(line_start).min(line_text.len());
    let (a, b) = select_word_range(&line_text, local, mode);
    (line_start.saturating_add(a), line_start.saturating_add(b))
}

pub fn move_word_left_in_buffer(buf: &TextBuffer, idx: usize, mode: TextBoundaryMode) -> usize {
    if buf.is_empty() {
        return 0;
    }
    let idx = buf.clamp_to_char_boundary_left(idx.min(buf.len_bytes()));
    let line = buf.line_index_at_byte(idx);
    let line_start = buf.line_start(line).unwrap_or(0);
    let line_text = buf.line_text(line).unwrap_or_default();
    let local = idx.saturating_sub(line_start).min(line_text.len());

    if local == 0 && line > 0 {
        let prev_line = line - 1;
        let prev_start = buf.line_start(prev_line).unwrap_or(0);
        let prev_text = buf.line_text(prev_line).unwrap_or_default();
        let prev_local = prev_text.len();
        let new_local = move_word_left(&prev_text, prev_local, mode);
        return prev_start.saturating_add(new_local);
    }

    line_start.saturating_add(move_word_left(&line_text, local, mode))
}

pub fn move_word_right_in_buffer(buf: &TextBuffer, idx: usize, mode: TextBoundaryMode) -> usize {
    if buf.is_empty() {
        return 0;
    }
    let idx = buf.clamp_to_char_boundary_left(idx.min(buf.len_bytes()));
    let line = buf.line_index_at_byte(idx);
    let line_start = buf.line_start(line).unwrap_or(0);
    let line_text = buf.line_text(line).unwrap_or_default();
    let local = idx.saturating_sub(line_start).min(line_text.len());

    if local >= line_text.len() && line.saturating_add(1) < buf.line_count() {
        let next_line = line + 1;
        let next_start = buf.line_start(next_line).unwrap_or(buf.len_bytes());
        let next_text = buf.line_text(next_line).unwrap_or_default();
        let new_local = move_word_right(&next_text, 0, mode);
        return next_start.saturating_add(new_local);
    }

    line_start.saturating_add(move_word_right(&line_text, local, mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::{DocId, TextBuffer};
    use fret_runtime::TextBoundaryMode;

    #[test]
    fn byte_to_display_point_counts_unicode_scalars() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "a😃b\nc".to_string()).unwrap();

        assert_eq!(byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(byte_to_display_point(&buf, 1), DisplayPoint::new(0, 1));
        assert_eq!(
            byte_to_display_point(&buf, 1 + "😃".len()),
            DisplayPoint::new(0, 2)
        );
        assert_eq!(
            byte_to_display_point(&buf, 1 + "😃".len() + 1),
            DisplayPoint::new(0, 3)
        );
        assert_eq!(
            byte_to_display_point(&buf, buf.text_string().find('\n').unwrap()),
            DisplayPoint::new(0, 3)
        );
        assert_eq!(
            byte_to_display_point(&buf, buf.text_string().find('\n').unwrap() + 1),
            DisplayPoint::new(1, 0)
        );
    }

    #[test]
    fn display_point_to_byte_clamps_to_line_end() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "ab\nc".to_string()).unwrap();

        assert_eq!(display_point_to_byte(&buf, DisplayPoint::new(0, 0)), 0);
        assert_eq!(display_point_to_byte(&buf, DisplayPoint::new(0, 1)), 1);
        assert_eq!(display_point_to_byte(&buf, DisplayPoint::new(0, 2)), 2);
        assert_eq!(display_point_to_byte(&buf, DisplayPoint::new(0, 99)), 2);
        assert_eq!(
            display_point_to_byte(&buf, DisplayPoint::new(1, 1)),
            buf.len_bytes()
        );
    }

    #[test]
    fn prev_char_boundary_handles_multibyte_chars() {
        let text = "a😃b";
        let after_emoji = 1 + "😃".len();
        assert_eq!(prev_char_boundary(text, after_emoji), 1);
    }

    #[test]
    fn select_line_range_includes_trailing_newline() {
        assert_eq!(select_line_range("hello\nworld", 0), (0, 6));
        assert_eq!(select_line_range("hello\nworld", 5), (0, 6));
        assert_eq!(select_line_range("hello\nworld", 6), (6, 11));
    }

    #[test]
    fn select_word_range_prefers_previous_when_on_whitespace() {
        assert_eq!(
            select_word_range("hello world", 5, TextBoundaryMode::UnicodeWord),
            (0, 5)
        );
        assert_eq!(
            select_word_range("hello world", 5, TextBoundaryMode::Identifier),
            (0, 5)
        );
    }

    #[test]
    fn move_word_right_distinguishes_unicode_word_and_identifier_for_apostrophe() {
        let text = "can't";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::UnicodeWord),
            text.len(),
            "UnicodeWord should treat \"can't\" as a single word"
        );
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::Identifier),
            3,
            "Identifier should split \"can't\" around the apostrophe"
        );
    }

    #[test]
    fn select_word_range_identifier_uses_xid_continue() {
        assert_eq!(
            select_word_range("αβγ δ", 1, TextBoundaryMode::Identifier),
            (0, "αβγ".len())
        );
        assert_eq!(
            select_word_range("a_b c", 1, TextBoundaryMode::Identifier),
            (0, "a_b".len())
        );
    }

    #[test]
    fn select_word_range_unicode_word_handles_cjk_runs() {
        let text = "世界 hello";
        assert_eq!(
            select_word_range(text, 0, TextBoundaryMode::UnicodeWord),
            (0, "世".len())
        );
        assert_eq!(
            select_word_range(text, "世".len(), TextBoundaryMode::UnicodeWord),
            ("世".len(), "世界".len())
        );
    }

    #[test]
    fn select_word_range_unicode_word_falls_back_to_single_char_on_emoji() {
        let text = "hi😀there";
        let emoji_start = "hi".len();
        let emoji_end = emoji_start + "😀".len();
        assert_eq!(
            select_word_range(text, emoji_start, TextBoundaryMode::UnicodeWord),
            (emoji_start, emoji_end)
        );
    }

    #[test]
    fn select_word_range_identifier_includes_digits_and_underscores() {
        let text = "foo123_bar baz";
        assert_eq!(
            select_word_range(text, 2, TextBoundaryMode::Identifier),
            (0, "foo123_bar".len())
        );
        assert_eq!(
            select_word_range(text, "foo".len() + 1, TextBoundaryMode::Identifier),
            (0, "foo123_bar".len())
        );
    }

    #[test]
    fn select_word_range_selects_whitespace_runs_when_not_preferring_previous_word() {
        assert_eq!(
            select_word_range("  hello", 1, TextBoundaryMode::UnicodeWord),
            (0, 2)
        );
        assert_eq!(
            select_word_range("  hello", 1, TextBoundaryMode::Identifier),
            (0, 2)
        );
    }

    #[test]
    fn move_word_right_skips_whitespace_and_moves_to_word_end() {
        let text = "hello   world";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::UnicodeWord),
            "hello".len()
        );
        assert_eq!(
            move_word_right(text, "hello".len(), TextBoundaryMode::UnicodeWord),
            text.len()
        );
    }

    #[test]
    fn move_word_identifier_respects_token_boundaries() {
        let text = "foo_bar baz";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::Identifier),
            "foo_bar".len()
        );
        assert_eq!(
            move_word_right(text, "foo_bar".len(), TextBoundaryMode::Identifier),
            text.len()
        );
        assert_eq!(
            move_word_left(text, text.len(), TextBoundaryMode::Identifier),
            "foo_bar ".len()
        );
        assert_eq!(
            move_word_left(text, "foo_bar ".len(), TextBoundaryMode::Identifier),
            0
        );
    }

    #[test]
    fn move_word_unicode_word_left_moves_to_previous_word_start() {
        let text = "hello   world";
        assert_eq!(
            move_word_left(text, text.len(), TextBoundaryMode::UnicodeWord),
            "hello   ".len()
        );
        assert_eq!(
            move_word_left(text, "hello   ".len(), TextBoundaryMode::UnicodeWord),
            0
        );
    }

    #[test]
    fn move_word_identifier_treats_punctuation_as_delimiter() {
        let text = "foo.bar";
        assert_eq!(
            move_word_right(text, 0, TextBoundaryMode::Identifier),
            "foo".len()
        );
        assert_eq!(
            move_word_right(text, "foo".len(), TextBoundaryMode::Identifier),
            text.len()
        );
        assert_eq!(
            move_word_left(text, text.len(), TextBoundaryMode::Identifier),
            "foo.".len()
        );
    }

    #[test]
    fn select_word_range_falls_back_to_single_char_on_punctuation() {
        let text = "foo.bar";
        let dot = "foo".len();
        assert_eq!(
            select_word_range(text, dot, TextBoundaryMode::UnicodeWord),
            (0, text.len())
        );
        assert_eq!(
            select_word_range(text, dot, TextBoundaryMode::Identifier),
            (dot, dot + 1)
        );
    }
}

#[cfg(test)]
mod display_map_tests {
    use super::*;
    use fret_code_editor_buffer::{DocId, TextBuffer};
    use std::collections::HashMap;

    fn fragments_to_string(buf: &TextBuffer, fragments: &[DisplayRowFragment]) -> String {
        let mut out = String::new();
        for f in fragments {
            match f {
                DisplayRowFragment::Buffer { range } => {
                    out.push_str(buf.slice_to_string(range.clone()).as_deref().unwrap_or(""));
                }
                DisplayRowFragment::Placeholder { text, .. } => {
                    out.push_str(text.as_ref());
                }
                DisplayRowFragment::Inlay { text, .. } => {
                    out.push_str(text.as_ref());
                }
                DisplayRowFragment::Preedit { text, .. } => {
                    out.push_str(text.as_ref());
                }
            }
        }
        out
    }

    #[test]
    fn display_map_without_wrap_matches_logical_lines() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "ab\nc".to_string()).unwrap();
        let map = DisplayMap::new(&buf, None);
        assert_eq!(map.row_count(), 2);

        assert_eq!(map.byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(map.byte_to_display_point(&buf, 2), DisplayPoint::new(0, 2));
        assert_eq!(map.byte_to_display_point(&buf, 3), DisplayPoint::new(1, 0));
        assert_eq!(
            map.display_point_to_byte(&buf, DisplayPoint::new(1, 1)),
            buf.len_bytes()
        );
    }

    #[test]
    fn inline_preedit_shifts_mapping_without_wrap() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();

        let folds: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        let inlays: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        let preedit = InlinePreedit {
            anchor: 2,
            text: Arc::<str>::from("XY"),
        };

        let map = DisplayMap::new_with_decorations_and_preedit(
            &buf,
            None,
            &folds,
            &inlays,
            Some(preedit),
        );

        assert_eq!(map.byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(map.byte_to_display_point(&buf, 2), DisplayPoint::new(0, 2));
        assert_eq!(map.byte_to_display_point(&buf, 3), DisplayPoint::new(0, 5));

        // Points inside the preedit map back to its anchor.
        assert_eq!(map.display_point_to_byte(&buf, DisplayPoint::new(0, 3)), 2);
        assert_eq!(map.display_point_to_byte(&buf, DisplayPoint::new(0, 4)), 2);
        assert_eq!(map.display_point_to_byte(&buf, DisplayPoint::new(0, 5)), 3);
    }

    #[test]
    fn inline_preedit_participates_in_wrapped_row_breaking() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();

        let folds: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        let inlays: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        let preedit = InlinePreedit {
            anchor: 2,
            text: Arc::<str>::from("XY"),
        };

        let map = DisplayMap::new_with_decorations_and_preedit(
            &buf,
            Some(4),
            &folds,
            &inlays,
            Some(preedit),
        );

        assert_eq!(map.row_count(), 2);
        assert_eq!(map.byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(map.byte_to_display_point(&buf, 3), DisplayPoint::new(1, 1));

        assert_eq!(map.display_row_byte_range(&buf, 0), 0..2);
        assert_eq!(map.display_row_byte_range(&buf, 1), 2..6);
    }

    #[test]
    fn display_map_wrap_cols_splits_rows() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcd\nef".to_string()).unwrap();
        let map = DisplayMap::new(&buf, Some(2));
        assert_eq!(map.row_count(), 3);

        // "abcd" is split into 2 display rows: "ab" and "cd".
        assert_eq!(map.byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(map.byte_to_display_point(&buf, 2), DisplayPoint::new(1, 0));
        assert_eq!(map.byte_to_display_point(&buf, 4), DisplayPoint::new(1, 2));

        // Second logical line "ef" stays a single row.
        assert_eq!(map.byte_to_display_point(&buf, 5), DisplayPoint::new(2, 0));
        assert_eq!(map.byte_to_display_point(&buf, 7), DisplayPoint::new(2, 2));
    }

    #[test]
    fn display_map_wrapped_roundtrips_char_boundaries() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "a馃槂bc".to_string()).unwrap();
        let map = DisplayMap::new(&buf, Some(2));

        let bytes = [
            0usize,
            1,
            1 + "馃槂".len(),
            1 + "馃槂".len() + 1,
            buf.len_bytes(),
        ];
        for byte in bytes {
            let pt = map.byte_to_display_point(&buf, byte);
            let back = map.display_point_to_byte(&buf, pt);
            assert_eq!(back, byte);
        }
    }

    #[test]
    fn display_row_byte_range_matches_logical_line_without_wrap() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "ab\nc".to_string()).unwrap();
        let map = DisplayMap::new(&buf, None);

        assert_eq!(map.display_row_byte_range(&buf, 0), 0..2);
        assert_eq!(map.display_row_byte_range(&buf, 1), 3..4);
    }

    #[test]
    fn display_row_fragments_unwrapped_replaces_fold_spans_with_placeholders() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef\n".to_string()).unwrap();
        let map = DisplayMap::new(&buf, None);

        let folds = vec![FoldSpan {
            range: 1..4,
            placeholder: std::sync::Arc::<str>::from("…"),
        }];
        let fragments = map
            .display_row_fragments_unwrapped(&buf, 0, &folds)
            .expect("fragments");
        assert_eq!(fragments_to_string(&buf, &fragments), "a…ef");
        assert_eq!(
            fragments,
            vec![
                DisplayRowFragment::Buffer { range: 0..1 },
                DisplayRowFragment::Placeholder {
                    text: std::sync::Arc::<str>::from("…"),
                    maps_to: 1,
                },
                DisplayRowFragment::Buffer { range: 4..6 },
            ]
        );
    }

    #[test]
    fn display_row_fragments_unwrapped_with_inlays_inserts_inlay_fragments() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef\n".to_string()).unwrap();
        let map = DisplayMap::new(&buf, None);

        let folds = vec![FoldSpan {
            range: 1..4,
            placeholder: std::sync::Arc::<str>::from("…"),
        }];
        let inlays = vec![InlaySpan {
            byte: 1,
            text: std::sync::Arc::<str>::from("<inlay>"),
        }];

        let fragments = map
            .display_row_fragments_unwrapped_with_inlays(&buf, 0, &folds, &inlays)
            .expect("fragments");
        assert_eq!(fragments_to_string(&buf, &fragments), "a<inlay>…ef");
        assert_eq!(
            fragments,
            vec![
                DisplayRowFragment::Buffer { range: 0..1 },
                DisplayRowFragment::Inlay {
                    text: std::sync::Arc::<str>::from("<inlay>"),
                    maps_to: 1,
                },
                DisplayRowFragment::Placeholder {
                    text: std::sync::Arc::<str>::from("…"),
                    maps_to: 1,
                },
                DisplayRowFragment::Buffer { range: 4..6 },
            ]
        );
    }

    #[test]
    fn display_row_fragments_unwrapped_rejects_wrap_mode() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();
        let map = DisplayMap::new(&buf, Some(2));
        let folds = vec![FoldSpan {
            range: 1..4,
            placeholder: std::sync::Arc::<str>::from("…"),
        }];
        assert_eq!(
            map.display_row_fragments_unwrapped(&buf, 0, &folds),
            Err(DisplayRowFragmentsError::UnsupportedWithWrap)
        );
    }

    #[test]
    fn display_row_byte_range_slices_wrapped_rows() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcd\nef".to_string()).unwrap();
        let map = DisplayMap::new(&buf, Some(2));

        assert_eq!(map.display_row_byte_range(&buf, 0), 0..2);
        assert_eq!(map.display_row_byte_range(&buf, 1), 2..4);
        assert_eq!(map.display_row_byte_range(&buf, 2), 5..7);
    }

    #[test]
    fn display_row_byte_range_handles_multibyte_chars() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "a😃b".to_string()).unwrap();
        let map = DisplayMap::new(&buf, Some(2));

        assert_eq!(map.display_row_byte_range(&buf, 0), 0..(1 + "😃".len()));
        assert_eq!(
            map.display_row_byte_range(&buf, 1),
            (1 + "😃".len())..buf.len_bytes()
        );
    }

    #[test]
    fn display_map_wrap_with_fold_placeholders_breaks_on_folded_text() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();

        let mut folds_by_line: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        folds_by_line.insert(
            0,
            Arc::from([FoldSpan {
                range: 1..4,
                placeholder: Arc::<str>::from("…"),
            }]),
        );
        let inlays_by_line: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();

        let map = DisplayMap::new_with_decorations(&buf, Some(2), &folds_by_line, &inlays_by_line);

        // Folded line "a…ef" is 4 columns, so it wraps into 2 rows.
        assert_eq!(map.row_count(), 2);
        assert_eq!(map.display_row_byte_range(&buf, 0), 0..4);
        assert_eq!(map.display_row_byte_range(&buf, 1), 4..6);

        // Bytes inside the folded range clamp to the fold start column.
        assert_eq!(map.byte_to_display_point(&buf, 0), DisplayPoint::new(0, 0));
        assert_eq!(map.byte_to_display_point(&buf, 1), DisplayPoint::new(0, 1));
        assert_eq!(map.byte_to_display_point(&buf, 2), DisplayPoint::new(0, 1));
        assert_eq!(map.byte_to_display_point(&buf, 3), DisplayPoint::new(0, 1));
        assert_eq!(map.byte_to_display_point(&buf, 4), DisplayPoint::new(1, 0));

        // Columns inside the placeholder map back to the fold start byte.
        assert_eq!(map.display_point_to_byte(&buf, DisplayPoint::new(0, 1)), 1);
        assert_eq!(map.display_point_to_byte(&buf, DisplayPoint::new(1, 0)), 4);
    }

    #[test]
    fn display_map_wrap_counts_inlays_in_row_breaks() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();

        let folds_by_line: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        let mut inlays_by_line: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        inlays_by_line.insert(
            0,
            Arc::from([InlaySpan {
                byte: 1,
                text: Arc::<str>::from("<inlay>"),
            }]),
        );

        let map = DisplayMap::new_with_decorations(&buf, Some(8), &folds_by_line, &inlays_by_line);

        // Line "a<inlay>bcdef" is 13 columns, so it wraps into 2 rows at 8 columns.
        assert_eq!(map.row_count(), 2);

        for byte in 0..=buf.len_bytes() {
            let pt = map.byte_to_display_point(&buf, byte);
            let back = map.display_point_to_byte(&buf, pt);
            assert_eq!(back, byte);
        }
    }
}
