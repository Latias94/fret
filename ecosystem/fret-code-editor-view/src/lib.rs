//! View-layer building blocks for the code editor ecosystem.
//!
//! v1 is intentionally minimal: "display rows" are logical lines split by `\n` and columns are
//! counted as Unicode scalar values (not graphemes, not rendered cells).

use fret_code_editor_buffer::TextBuffer;

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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::{DocId, TextBuffer};

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
}
