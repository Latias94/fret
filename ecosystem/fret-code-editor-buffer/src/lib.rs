use std::ops::Range;

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocId(Uuid);

impl DocId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DocId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Insert { at: usize, text: String },
    Delete { range: Range<usize> },
    Replace { range: Range<usize>, text: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineDelta {
    pub start: usize,
    pub old_count: usize,
    pub new_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferDelta {
    pub before: Revision,
    pub after: Revision,
    pub lines: LineDelta,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EditError {
    #[error("range start is greater than end")]
    RangeStartAfterEnd,
    #[error("range end is out of bounds")]
    RangeEndOutOfBounds,
    #[error("index is out of bounds")]
    IndexOutOfBounds,
    #[error("index is not a UTF-8 char boundary")]
    NotCharBoundary,
}

#[derive(Debug, Clone)]
pub struct TextBuffer {
    doc: DocId,
    revision: Revision,
    text: String,
    line_starts: Vec<usize>,
}

impl TextBuffer {
    pub fn new(doc: DocId, text: String) -> Result<Self, EditError> {
        if !text.is_char_boundary(text.len()) {
            return Err(EditError::NotCharBoundary);
        }

        let mut buf = Self {
            doc,
            revision: Revision(0),
            text,
            line_starts: Vec::new(),
        };
        buf.rebuild_line_index();
        Ok(buf)
    }

    pub fn doc(&self) -> DocId {
        self.doc
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len_bytes(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len().max(1)
    }

    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    pub fn line_byte_range_including_newline(&self, line: usize) -> Option<Range<usize>> {
        let start = self.line_start(line)?;
        let end = self
            .line_start(line.saturating_add(1))
            .unwrap_or_else(|| self.text.len());
        Some(start..end.min(self.text.len()))
    }

    pub fn line_byte_range(&self, line: usize) -> Option<Range<usize>> {
        let range = self.line_byte_range_including_newline(line)?;
        let end = range.end;
        let end = if end > range.start && self.text.as_bytes().get(end - 1) == Some(&b'\n') {
            end - 1
        } else {
            end
        };
        Some(range.start..end)
    }

    pub fn line_text(&self, line: usize) -> Option<&str> {
        let range = self.line_byte_range(line)?;
        self.text.get(range)
    }

    pub fn line_index_at_byte(&self, idx: usize) -> usize {
        let idx = idx.min(self.text.len());
        match self.line_starts.binary_search(&idx) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        }
    }

    pub fn apply(&mut self, edit: Edit) -> Result<BufferDelta, EditError> {
        let before_text = self.text.clone();
        let before_lines = self.line_starts.clone();
        let before_rev = self.revision;

        let (start, end, insert) = match edit {
            Edit::Insert { at, text } => {
                self.validate_index(at)?;
                (at, at, text)
            }
            Edit::Delete { range } => {
                self.validate_range(&range)?;
                (range.start, range.end, String::new())
            }
            Edit::Replace { range, text } => {
                self.validate_range(&range)?;
                (range.start, range.end, text)
            }
        };

        if !insert.is_char_boundary(insert.len()) {
            return Err(EditError::NotCharBoundary);
        }
        if !before_text.is_char_boundary(start) || !before_text.is_char_boundary(end) {
            return Err(EditError::NotCharBoundary);
        }

        self.text.replace_range(start..end, &insert);
        self.revision = Revision(self.revision.0.saturating_add(1));
        self.rebuild_line_index();

        let old_start_line = line_index_at_byte(&before_lines, &before_text, start);
        let old_end_line = line_index_at_byte(&before_lines, &before_text, end);
        let old_count = old_end_line
            .saturating_sub(old_start_line)
            .saturating_add(1);

        let new_end = start.saturating_add(insert.len()).min(self.text.len());
        let new_end_line = self.line_index_at_byte(new_end);
        let new_count = new_end_line
            .saturating_sub(old_start_line)
            .saturating_add(1);

        Ok(BufferDelta {
            before: before_rev,
            after: self.revision,
            lines: LineDelta {
                start: old_start_line,
                old_count,
                new_count,
            },
        })
    }

    fn validate_index(&self, idx: usize) -> Result<(), EditError> {
        if idx > self.text.len() {
            return Err(EditError::IndexOutOfBounds);
        }
        if !self.text.is_char_boundary(idx) {
            return Err(EditError::NotCharBoundary);
        }
        Ok(())
    }

    fn validate_range(&self, range: &Range<usize>) -> Result<(), EditError> {
        if range.start > range.end {
            return Err(EditError::RangeStartAfterEnd);
        }
        if range.end > self.text.len() {
            return Err(EditError::RangeEndOutOfBounds);
        }
        if !self.text.is_char_boundary(range.start) || !self.text.is_char_boundary(range.end) {
            return Err(EditError::NotCharBoundary);
        }
        Ok(())
    }

    fn rebuild_line_index(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);
        for (idx, ch) in self.text.char_indices() {
            if ch == '\n' && idx + 1 <= self.text.len() {
                self.line_starts.push(idx + 1);
            }
        }
    }
}

fn line_index_at_byte(starts: &[usize], text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    match starts.binary_search(&idx) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) => i - 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_updates_text_and_revision() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "hello\nworld".to_string()).unwrap();
        assert_eq!(buf.revision(), Revision(0));

        let delta = buf
            .apply(Edit::Insert {
                at: 5,
                text: ", ".to_string(),
            })
            .unwrap();

        assert_eq!(buf.text(), "hello, \nworld");
        assert_eq!(delta.before, Revision(0));
        assert_eq!(delta.after, Revision(1));
        assert_eq!(delta.lines.start, 0);
    }

    #[test]
    fn replace_across_newline_tracks_line_delta() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "a\nb\nc".to_string()).unwrap();

        let delta = buf
            .apply(Edit::Replace {
                range: 1..4,
                text: "\nXX\n".to_string(),
            })
            .unwrap();

        assert_eq!(buf.text(), "a\nXX\nc");
        assert_eq!(delta.lines.start, 0);
        assert!(delta.lines.old_count >= 2);
        assert!(delta.lines.new_count >= 2);
    }

    #[test]
    fn rejects_non_char_boundary_index() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "😃".to_string()).unwrap();
        let err = buf
            .apply(Edit::Insert {
                at: 1,
                text: "x".to_string(),
            })
            .unwrap_err();
        assert_eq!(err, EditError::NotCharBoundary);
    }

    #[test]
    fn line_text_excludes_trailing_newline() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "a\nb\n".to_string()).unwrap();
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line_text(0), Some("a"));
        assert_eq!(buf.line_text(1), Some("b"));
        assert_eq!(buf.line_text(2), Some(""));
        assert_eq!(buf.line_byte_range_including_newline(1), Some(2..4));
        assert_eq!(buf.line_byte_range(1), Some(2..3));
    }
}
