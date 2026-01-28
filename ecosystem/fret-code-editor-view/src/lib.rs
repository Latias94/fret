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
    text[..idx]
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
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
        .map(|i| (idx + i).min(text.len()))
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
