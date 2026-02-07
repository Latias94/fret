//! View-layer building blocks for the code editor ecosystem.
//!
//! v1 is intentionally minimal: "display rows" are logical lines split by `\n` and columns are
//! counted as Unicode scalar values (not graphemes, not rendered cells).

use fret_code_editor_buffer::TextBuffer;
use fret_runtime::TextBoundaryMode;
use fret_text_nav as text_nav;
use std::ops::Range;

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
}

impl DisplayMap {
    /// Build a display map from the current buffer state.
    ///
    /// `wrap_cols` counts Unicode scalar values within a logical line (newline excluded).
    /// When `None`, display rows match logical lines.
    pub fn new(buf: &TextBuffer, wrap_cols: Option<usize>) -> Self {
        let wrap_cols = wrap_cols.filter(|v| *v > 0);

        let line_count = buf.line_count().max(1);

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
            };
        }

        let mut line_to_first_row = Vec::with_capacity(line_count);
        let mut row_to_line = Vec::new();
        let mut row_start_col = Vec::new();

        for line in 0..line_count {
            line_to_first_row.push(row_to_line.len());

            let cols = buf.line_char_count(line);
            let rows_for_line = match wrap_cols {
                None => 1,
                Some(wrap) => ((cols.max(1) + wrap - 1) / wrap).max(1),
            };

            for row_in_line in 0..rows_for_line {
                row_to_line.push(line);
                row_start_col.push(row_in_line * wrap_cols.unwrap_or(usize::MAX));
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

    /// Map a UTF-8 byte index in the buffer to a wrapped display coordinate.
    pub fn byte_to_display_point(&self, buf: &TextBuffer, byte: usize) -> DisplayPoint {
        let pt = byte_to_display_point(buf, byte);
        let Some(wrap) = self.wrap_cols else {
            return pt;
        };

        let line = pt.row.min(self.line_to_first_row.len().saturating_sub(1));
        let line_first = *self.line_to_first_row.get(line).unwrap_or(&0);
        let line_last_excl = self
            .line_to_first_row
            .get(line + 1)
            .copied()
            .unwrap_or_else(|| self.row_to_line.len());
        let rows_for_line = line_last_excl.saturating_sub(line_first).max(1);

        let row_in_line = (pt.col / wrap).min(rows_for_line.saturating_sub(1));
        let col_in_row = pt.col.saturating_sub(row_in_line * wrap);
        DisplayPoint::new(line_first + row_in_line, col_in_row)
    }

    /// Map a wrapped display coordinate to a UTF-8 byte index in the buffer.
    ///
    /// If the point is out of bounds, this clamps to the nearest representable position.
    pub fn display_point_to_byte(&self, buf: &TextBuffer, mut pt: DisplayPoint) -> usize {
        let Some(_wrap) = self.wrap_cols else {
            return display_point_to_byte(buf, pt);
        };

        if self.row_to_line.is_empty() {
            return 0;
        }
        pt.row = pt.row.min(self.row_to_line.len().saturating_sub(1));
        let line = self.row_to_line[pt.row];
        let row_start_col = self.row_start_col[pt.row];
        let col_in_line = row_start_col.saturating_add(pt.col);
        buf.byte_at_line_col(line, col_in_line)
    }
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
}
