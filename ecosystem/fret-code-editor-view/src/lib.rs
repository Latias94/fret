//! View-layer building blocks for the code editor ecosystem.
//!
//! v1 is intentionally minimal: "display rows" are logical lines split by `\n` and columns are
//! counted as Unicode scalar values (not graphemes, not rendered cells).

use fret_code_editor_buffer::TextBuffer;
use fret_runtime::TextBoundaryMode;
use unicode_segmentation::UnicodeSegmentation;

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
        let mut line_to_first_row = Vec::with_capacity(line_count);
        let mut row_to_line = Vec::new();
        let mut row_start_col = Vec::new();

        for line in 0..line_count {
            line_to_first_row.push(row_to_line.len());

            let cols = buf.line_text(line).unwrap_or("").chars().count();
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
        let Some(range) = buf.line_byte_range(line) else {
            return buf.len_bytes();
        };
        let start = range.start.min(buf.len_bytes());
        let end = range.end.min(buf.len_bytes());
        if start >= end {
            return start;
        }

        let Some(line_text) = buf.text().get(start..end) else {
            return start;
        };

        let row_start_col = self.row_start_col[pt.row];
        let col_in_line = row_start_col.saturating_add(pt.col);

        let mut remaining = col_in_line;
        let mut offset = start;
        for ch in line_text.chars() {
            if remaining == 0 {
                break;
            }
            offset = offset.saturating_add(ch.len_utf8());
            remaining -= 1;
        }

        // Clamp to the logical line end (excluding the trailing newline).
        offset.min(end).min(display_point_to_byte(
            buf,
            DisplayPoint::new(line, usize::MAX),
        ))
    }
}

/// Map a UTF-8 byte index in the buffer to a `(row, col)` display coordinate.
pub fn byte_to_display_point(buf: &TextBuffer, mut byte: usize) -> DisplayPoint {
    byte = byte.min(buf.len_bytes());
    while byte > 0 && !buf.text().is_char_boundary(byte) {
        byte = byte.saturating_sub(1);
    }

    let row = buf.line_index_at_byte(byte);
    let row_start = buf.line_start(row).unwrap_or(0).min(buf.len_bytes());
    let byte = byte.max(row_start).min(buf.len_bytes());
    let col = buf
        .text()
        .get(row_start..byte)
        .map(|s| s.chars().count())
        .unwrap_or(0);

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

    let Some(range) = buf.line_byte_range(pt.row) else {
        return buf.len_bytes();
    };
    let start = range.start.min(buf.len_bytes());
    let end = range.end.min(buf.len_bytes());
    if start >= end {
        return start;
    }

    let Some(line) = buf.text().get(start..end) else {
        return start;
    };

    let mut col = pt.col;
    let mut offset = start;
    for ch in line.chars() {
        if col == 0 {
            break;
        }
        offset = offset.saturating_add(ch.len_utf8());
        col -= 1;
    }
    offset.min(end)
}

pub fn clamp_to_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    if text.is_char_boundary(idx) {
        return idx;
    }
    let mut i = idx;
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

pub fn prev_char_boundary(text: &str, idx: usize) -> usize {
    let idx = clamp_to_char_boundary(text, idx);
    if idx == 0 {
        return 0;
    }

    // Avoid scanning from the start (which is O(n)). Back up to the previous UTF-8 char boundary.
    let mut i = idx.saturating_sub(1);
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

pub fn next_char_boundary(text: &str, idx: usize) -> usize {
    let idx = clamp_to_char_boundary(text, idx);
    if idx >= text.len() {
        return text.len();
    }
    let ch = text[idx..].chars().next().unwrap_or('\0');
    idx.saturating_add(ch.len_utf8()).min(text.len())
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_continue(ch)
}

fn char_at(text: &str, idx: usize) -> Option<char> {
    let idx = clamp_to_char_boundary(text, idx);
    text.get(idx..)?.chars().next()
}

fn is_unicode_word_char(text: &str, idx: usize) -> bool {
    let idx = clamp_to_char_boundary(text, idx);
    text.unicode_word_indices()
        .any(|(start, word)| (start..start + word.len()).contains(&idx))
}

fn unicode_word_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
    let idx = clamp_to_char_boundary(text, idx);
    for (start, word) in text.unicode_word_indices() {
        let end = start + word.len();
        if (start..end).contains(&idx) {
            return Some((start, end));
        }
    }
    None
}

fn identifier_range_at(text: &str, idx: usize) -> Option<(usize, usize)> {
    let idx = clamp_to_char_boundary(text, idx);
    let ch = char_at(text, idx)?;
    if !is_identifier_char(ch) {
        return None;
    }

    let mut start = idx;
    while start > 0 {
        let prev = prev_char_boundary(text, start);
        let prev_ch = char_at(text, prev).unwrap_or(' ');
        if !is_identifier_char(prev_ch) {
            break;
        }
        start = prev;
    }

    let mut end = next_char_boundary(text, idx);
    while end < text.len() {
        let next_ch = char_at(text, end).unwrap_or(' ');
        if !is_identifier_char(next_ch) {
            break;
        }
        end = next_char_boundary(text, end);
    }

    Some((start, end))
}

pub fn select_word_range(text: &str, idx: usize, mode: TextBoundaryMode) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let mut idx = clamp_to_char_boundary(text, idx).min(text.len());
    if idx >= text.len() {
        idx = prev_char_boundary(text, idx);
    }

    if char_at(text, idx).is_some_and(|c| c.is_whitespace()) && idx > 0 {
        let prev = prev_char_boundary(text, idx);
        let prev_is_word = match mode {
            TextBoundaryMode::UnicodeWord => is_unicode_word_char(text, prev),
            TextBoundaryMode::Identifier => char_at(text, prev).is_some_and(is_identifier_char),
        };
        if prev_is_word {
            idx = prev;
        }
    }

    let Some(ch) = char_at(text, idx) else {
        return (0, 0);
    };

    if ch.is_whitespace() {
        let mut start = idx;
        while start > 0 {
            let prev = prev_char_boundary(text, start);
            if char_at(text, prev).is_some_and(|c| c.is_whitespace()) {
                start = prev;
            } else {
                break;
            }
        }
        let mut end = next_char_boundary(text, idx);
        while end < text.len() {
            if char_at(text, end).is_some_and(|c| c.is_whitespace()) {
                end = next_char_boundary(text, end);
            } else {
                break;
            }
        }
        return (start, end);
    }

    match mode {
        TextBoundaryMode::UnicodeWord => {
            unicode_word_range_at(text, idx).unwrap_or((idx, next_char_boundary(text, idx)))
        }
        TextBoundaryMode::Identifier => {
            identifier_range_at(text, idx).unwrap_or((idx, next_char_boundary(text, idx)))
        }
    }
}

pub fn select_line_range(text: &str, idx: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }

    let idx = clamp_to_char_boundary(text, idx).min(text.len());
    let start = text[..idx]
        .rfind('\n')
        .map(|i| (i + 1).min(text.len()))
        .unwrap_or(0);
    let end = text[idx..]
        .find('\n')
        .map(|i| (idx + i + 1).min(text.len()))
        .unwrap_or(text.len());
    (start, end)
}

pub fn move_word_left(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    let mut i = prev_char_boundary(text, idx);
    while i > 0 {
        let prev = prev_char_boundary(text, i);
        let ch = text[prev..i].chars().next().unwrap_or(' ');
        if !ch.is_whitespace() {
            break;
        }
        i = prev;
    }

    if i == 0 {
        return 0;
    }

    match mode {
        TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, i)
            .map(|(start, _)| start)
            .unwrap_or(i),
        TextBoundaryMode::Identifier => identifier_range_at(text, i)
            .map(|(start, _)| start)
            .unwrap_or(i),
    }
}

pub fn move_word_right(text: &str, idx: usize, mode: TextBoundaryMode) -> usize {
    let mut i = next_char_boundary(text, idx);
    while i < text.len() {
        let next = next_char_boundary(text, i);
        let ch = text[i..next].chars().next().unwrap_or(' ');
        if !ch.is_whitespace() {
            break;
        }
        i = next;
    }

    if i >= text.len() {
        return text.len();
    }

    match mode {
        TextBoundaryMode::UnicodeWord => unicode_word_range_at(text, i)
            .map(|(_, end)| end)
            .unwrap_or(i),
        TextBoundaryMode::Identifier => identifier_range_at(text, i)
            .map(|(_, end)| end)
            .unwrap_or(i),
    }
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
            byte_to_display_point(&buf, buf.text().find('\n').unwrap()),
            DisplayPoint::new(0, 3)
        );
        assert_eq!(
            byte_to_display_point(&buf, buf.text().find('\n').unwrap() + 1),
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
}
