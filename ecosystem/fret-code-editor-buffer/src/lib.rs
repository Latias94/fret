use std::ops::Range;

use ropey::Rope;
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

/// A URI-like document identity for workspace shells.
///
/// This is intentionally a thin wrapper; normalization and scheme decisions are left to the
/// workspace layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocUri(String);

impl DocUri {
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEdit {
    pub edit: Edit,
    pub inverse: Edit,
    pub delta: BufferDelta,
}

/// A committed, invertible text transaction expressed as UTF-8 byte-index edits.
///
/// The transaction is self-contained: it includes the inverse edits computed from the buffer
/// state at apply time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextBufferTx {
    pub edits: Vec<Edit>,
    pub inverse_edits: Vec<Edit>,
}

impl TextBufferTx {
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub fn invert(&self) -> Self {
        Self {
            edits: self.inverse_edits.iter().rev().cloned().collect(),
            inverse_edits: self.edits.iter().rev().cloned().collect(),
        }
    }
}

/// Builder for a multi-edit transaction.
///
/// This type does not hold references into the buffer; callers apply edits via
/// `TextBuffer::apply_in_transaction`.
#[derive(Debug, Clone, Default)]
pub struct TextBufferTransaction {
    edits: Vec<Edit>,
    inverse_edits: Vec<Edit>,
}

impl TextBufferTransaction {
    /// Convert a live transaction snapshot into a committed, invertible transaction.
    ///
    /// This is equivalent to `snapshot()`, but uses the "commit" vocabulary used by
    /// undo/history layers.
    pub fn snapshot_tx(&self) -> TextBufferTx {
        self.snapshot()
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub fn clear(&mut self) {
        self.edits.clear();
        self.inverse_edits.clear();
    }

    pub fn snapshot(&self) -> TextBufferTx {
        TextBufferTx {
            edits: self.edits.clone(),
            inverse_edits: self.inverse_edits.clone(),
        }
    }

    /// Commit the transaction, consuming the builder and producing an invertible transaction.
    ///
    /// This is equivalent to `into_tx()`, but uses the "commit" vocabulary used by
    /// undo/history layers.
    pub fn commit(self) -> TextBufferTx {
        self.into_tx()
    }

    pub fn into_tx(self) -> TextBufferTx {
        TextBufferTx {
            edits: self.edits,
            inverse_edits: self.inverse_edits,
        }
    }
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
    uri: Option<DocUri>,
    revision: Revision,
    rope: Rope,
}

impl TextBuffer {
    pub fn new(doc: DocId, text: String) -> Result<Self, EditError> {
        Self::new_with_uri(doc, None, text)
    }

    pub fn new_with_uri(doc: DocId, uri: Option<DocUri>, text: String) -> Result<Self, EditError> {
        if !text.is_char_boundary(text.len()) {
            return Err(EditError::NotCharBoundary);
        }

        let buf = Self {
            doc,
            uri,
            revision: Revision(0),
            rope: Rope::from_str(&text),
        };
        Ok(buf)
    }

    pub fn doc(&self) -> DocId {
        self.doc
    }

    pub fn uri(&self) -> Option<&DocUri> {
        self.uri.as_ref()
    }

    pub fn set_uri(&mut self, uri: Option<DocUri>) {
        self.uri = uri;
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// Best-effort string snapshot for debugging/tests.
    ///
    /// Editor surfaces should prefer slicing APIs instead of materializing the whole document.
    pub fn text_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn is_char_boundary(&self, idx: usize) -> bool {
        if idx == 0 || idx == self.len_bytes() {
            return true;
        }
        if idx > self.len_bytes() {
            return false;
        }
        let (chunk, chunk_start, _, _) = self.rope.chunk_at_byte(idx);
        let rel = idx.saturating_sub(chunk_start);
        chunk.is_char_boundary(rel)
    }

    pub fn slice_to_string(&self, range: Range<usize>) -> Option<String> {
        if range.start > range.end {
            return None;
        }
        if range.end > self.len_bytes() {
            return None;
        }
        if !self.is_char_boundary(range.start) || !self.is_char_boundary(range.end) {
            return None;
        }
        let start_char = self.rope.byte_to_char(range.start);
        let end_char = self.rope.byte_to_char(range.end);
        Some(self.rope.slice(start_char..end_char).to_string())
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn line_start(&self, line: usize) -> Option<usize> {
        if line >= self.line_count() {
            return None;
        }
        Some(self.rope.line_to_byte(line))
    }

    pub fn line_byte_range_including_newline(&self, line: usize) -> Option<Range<usize>> {
        let start = self.line_start(line)?;
        let end = if line.saturating_add(1) < self.line_count() {
            self.rope.line_to_byte(line + 1)
        } else {
            self.len_bytes()
        };
        Some(start..end.min(self.len_bytes()))
    }

    pub fn line_byte_range(&self, line: usize) -> Option<Range<usize>> {
        let range = self.line_byte_range_including_newline(line)?;
        let mut end = range.end;
        if end > range.start && self.byte_at(end - 1) == Some(b'\n') {
            end = end.saturating_sub(1);
        }
        Some(range.start..end.min(self.len_bytes()))
    }

    pub fn line_text(&self, line: usize) -> Option<String> {
        let range = self.line_byte_range(line)?;
        self.slice_to_string(range)
    }

    pub fn line_char_count(&self, line: usize) -> usize {
        let Some(range) = self.line_byte_range_including_newline(line) else {
            return 0;
        };
        let mut end = range.end.min(self.len_bytes());
        if end > range.start && self.byte_at(end - 1) == Some(b'\n') {
            end = end.saturating_sub(1);
        }
        if !self.is_char_boundary(range.start) || !self.is_char_boundary(end) {
            return 0;
        }
        let start_char = self.rope.byte_to_char(range.start);
        let end_char = self.rope.byte_to_char(end);
        end_char.saturating_sub(start_char)
    }

    pub fn line_index_at_byte(&self, idx: usize) -> usize {
        let idx = self.clamp_to_char_boundary_left(idx.min(self.len_bytes()));
        self.rope
            .byte_to_line(idx)
            .min(self.line_count().saturating_sub(1))
    }

    pub fn line_col_at_byte(&self, idx: usize) -> (usize, usize) {
        let idx = self.clamp_to_char_boundary_left(idx.min(self.len_bytes()));
        let line = self.line_index_at_byte(idx);
        let line_start_char = self.rope.line_to_char(line);
        let char_idx = self.rope.byte_to_char(idx);
        (line, char_idx.saturating_sub(line_start_char))
    }

    pub fn byte_at_line_col(&self, line: usize, col: usize) -> usize {
        let line = line.min(self.line_count().saturating_sub(1));
        let start_char = self.rope.line_to_char(line);
        let line_slice = self.rope.line(line);
        let mut len_chars = line_slice.len_chars();
        if len_chars > 0 && line_slice.char(len_chars - 1) == '\n' {
            len_chars = len_chars.saturating_sub(1);
        }
        let col = col.min(len_chars);
        self.rope.char_to_byte(start_char.saturating_add(col))
    }

    pub fn prev_char_boundary(&self, idx: usize) -> usize {
        let idx = self.clamp_to_char_boundary_left(idx.min(self.len_bytes()));
        let char_idx = self.rope.byte_to_char(idx);
        if char_idx == 0 {
            0
        } else {
            self.rope.char_to_byte(char_idx - 1)
        }
    }

    pub fn next_char_boundary(&self, idx: usize) -> usize {
        let idx = self.clamp_to_char_boundary_left(idx.min(self.len_bytes()));
        let char_idx = self.rope.byte_to_char(idx);
        let next = char_idx.saturating_add(1).min(self.rope.len_chars());
        self.rope.char_to_byte(next)
    }

    pub fn apply(&mut self, edit: Edit) -> Result<BufferDelta, EditError> {
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

        self.validate_index(start)?;
        self.validate_index(end)?;

        let old_start_line = self.line_index_at_byte(start);
        let old_end_line = self.line_index_at_byte(end);
        let old_count = old_end_line
            .saturating_sub(old_start_line)
            .saturating_add(1);

        let start_char = self.rope.byte_to_char(start);
        let end_char = self.rope.byte_to_char(end);
        if start_char < end_char {
            self.rope.remove(start_char..end_char);
        }
        if !insert.is_empty() {
            self.rope.insert(start_char, &insert);
        }
        self.revision = Revision(self.revision.0.saturating_add(1));

        let new_end = start.saturating_add(insert.len()).min(self.len_bytes());
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

    pub fn apply_tx(&mut self, tx: &TextBufferTx) -> Result<(), EditError> {
        for edit in &tx.edits {
            self.apply(edit.clone())?;
        }
        Ok(())
    }

    /// Begin a new text transaction (buffer-level begin/update/commit/cancel vocabulary).
    pub fn transaction_begin(&self) -> TextBufferTransaction {
        TextBufferTransaction::default()
    }

    /// Update an in-flight transaction by applying an edit and recording its inverse.
    pub fn transaction_update(
        &mut self,
        tx: &mut TextBufferTransaction,
        edit: Edit,
    ) -> Result<BufferDelta, EditError> {
        self.apply_in_transaction(tx, edit)
    }

    /// Commit a transaction builder into an invertible transaction object.
    pub fn transaction_commit(&self, tx: TextBufferTransaction) -> TextBufferTx {
        tx.into_tx()
    }

    /// Cancel an in-flight transaction by rolling back the recorded inverse edits.
    pub fn transaction_cancel(&mut self, tx: &TextBufferTransaction) -> Result<(), EditError> {
        self.rollback_transaction(tx)
    }

    pub fn apply_with_inverse(&mut self, edit: Edit) -> Result<AppliedEdit, EditError> {
        let inverse = match &edit {
            Edit::Insert { at, text } => {
                self.validate_index(*at)?;
                Edit::Delete {
                    range: (*at)..at.saturating_add(text.len()),
                }
            }
            Edit::Delete { range } => {
                self.validate_range(range)?;
                let removed = self
                    .slice_to_string(range.clone())
                    .ok_or(EditError::RangeEndOutOfBounds)?;
                Edit::Insert {
                    at: range.start,
                    text: removed,
                }
            }
            Edit::Replace { range, text } => {
                self.validate_range(range)?;
                let removed = self
                    .slice_to_string(range.clone())
                    .ok_or(EditError::RangeEndOutOfBounds)?;
                Edit::Replace {
                    range: range.start..range.start.saturating_add(text.len()),
                    text: removed,
                }
            }
        };

        let delta = self.apply(edit.clone())?;
        Ok(AppliedEdit {
            edit,
            inverse,
            delta,
        })
    }

    pub fn apply_in_transaction(
        &mut self,
        tx: &mut TextBufferTransaction,
        edit: Edit,
    ) -> Result<BufferDelta, EditError> {
        let applied = self.apply_with_inverse(edit)?;
        let delta = applied.delta;
        tx.edits.push(applied.edit);
        tx.inverse_edits.push(applied.inverse);
        Ok(delta)
    }

    pub fn rollback_transaction(&mut self, tx: &TextBufferTransaction) -> Result<(), EditError> {
        for edit in tx.inverse_edits.iter().rev() {
            self.apply(edit.clone())?;
        }
        Ok(())
    }

    fn validate_index(&self, idx: usize) -> Result<(), EditError> {
        if idx > self.len_bytes() {
            return Err(EditError::IndexOutOfBounds);
        }
        if !self.is_char_boundary(idx) {
            return Err(EditError::NotCharBoundary);
        }
        Ok(())
    }

    fn validate_range(&self, range: &Range<usize>) -> Result<(), EditError> {
        if range.start > range.end {
            return Err(EditError::RangeStartAfterEnd);
        }
        if range.end > self.len_bytes() {
            return Err(EditError::RangeEndOutOfBounds);
        }
        if !self.is_char_boundary(range.start) || !self.is_char_boundary(range.end) {
            return Err(EditError::NotCharBoundary);
        }
        Ok(())
    }

    fn byte_at(&self, idx: usize) -> Option<u8> {
        if idx >= self.len_bytes() {
            return None;
        }
        let (chunk, chunk_start, _, _) = self.rope.chunk_at_byte(idx);
        let rel = idx.saturating_sub(chunk_start);
        chunk.as_bytes().get(rel).copied()
    }

    pub fn clamp_to_char_boundary_left(&self, mut idx: usize) -> usize {
        idx = idx.min(self.len_bytes());
        while idx > 0 && !self.is_char_boundary(idx) {
            idx = idx.saturating_sub(1);
        }
        idx
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

        assert_eq!(buf.text_string(), "hello, \nworld");
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

        assert_eq!(buf.text_string(), "a\nXX\nc");
        assert_eq!(delta.lines.start, 0);
        assert!(delta.lines.old_count >= 2);
        assert!(delta.lines.new_count >= 2);
    }

    #[test]
    fn delete_newline_merges_lines_and_updates_index() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "a\nb".to_string()).unwrap();
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_start(0), Some(0));
        assert_eq!(buf.line_start(1), Some(2));

        buf.apply(Edit::Delete { range: 1..2 }).unwrap();

        assert_eq!(buf.text_string(), "ab");
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.line_start(0), Some(0));
        assert_eq!(buf.line_start(1), None);
    }

    #[test]
    fn insert_at_line_start_does_not_shift_line_start() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "a\nb".to_string()).unwrap();
        assert_eq!(buf.line_start(1), Some(2));

        buf.apply(Edit::Insert {
            at: 2,
            text: "X".to_string(),
        })
        .unwrap();

        assert_eq!(buf.text_string(), "a\nXb");
        assert_eq!(buf.line_start(1), Some(2));
    }

    #[test]
    fn insert_newline_creates_new_line_start() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "ab".to_string()).unwrap();
        assert_eq!(buf.line_count(), 1);

        buf.apply(Edit::Insert {
            at: 1,
            text: "\n".to_string(),
        })
        .unwrap();

        assert_eq!(buf.text_string(), "a\nb");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_start(1), Some(2));
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
        assert_eq!(buf.line_text(0), Some("a".to_string()));
        assert_eq!(buf.line_text(1), Some("b".to_string()));
        assert_eq!(buf.line_text(2), Some(String::new()));
        assert_eq!(buf.line_byte_range_including_newline(1), Some(2..4));
        assert_eq!(buf.line_byte_range(1), Some(2..3));
    }

    #[test]
    fn transaction_invert_roundtrip() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "abc".to_string()).unwrap();
        let mut txn = TextBufferTransaction::default();

        let _ = buf
            .apply_in_transaction(
                &mut txn,
                Edit::Insert {
                    at: 3,
                    text: "d".to_string(),
                },
            )
            .unwrap();
        let _ = buf
            .apply_in_transaction(
                &mut txn,
                Edit::Insert {
                    at: 4,
                    text: "e".to_string(),
                },
            )
            .unwrap();

        assert_eq!(buf.text_string(), "abcde");

        let tx = txn.snapshot();
        assert_eq!(tx.edits.len(), 2);
        assert_eq!(tx.inverse_edits.len(), 2);

        let undo_tx = tx.invert();
        buf.apply_tx(&undo_tx).unwrap();
        assert_eq!(buf.text_string(), "abc");

        let redo_tx = undo_tx.invert();
        buf.apply_tx(&redo_tx).unwrap();
        assert_eq!(buf.text_string(), "abcde");
    }

    #[test]
    fn rollback_transaction_restores_text() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "hello".to_string()).unwrap();
        let mut txn = TextBufferTransaction::default();

        let _ = buf
            .apply_in_transaction(
                &mut txn,
                Edit::Insert {
                    at: 5,
                    text: " world".to_string(),
                },
            )
            .unwrap();
        let _ = buf
            .apply_in_transaction(
                &mut txn,
                Edit::Replace {
                    range: 0..5,
                    text: "hi".to_string(),
                },
            )
            .unwrap();
        assert_eq!(buf.text_string(), "hi world");

        buf.rollback_transaction(&txn).unwrap();
        assert_eq!(buf.text_string(), "hello");
    }

    #[test]
    fn transaction_hooks_vocabulary_roundtrip() {
        let doc = DocId::new();
        let mut buf = TextBuffer::new(doc, "hello".to_string()).unwrap();

        let mut txn = buf.transaction_begin();
        let _ = buf
            .transaction_update(
                &mut txn,
                Edit::Insert {
                    at: 5,
                    text: " world".to_string(),
                },
            )
            .unwrap();
        let _ = buf
            .transaction_update(
                &mut txn,
                Edit::Replace {
                    range: 0..5,
                    text: "hi".to_string(),
                },
            )
            .unwrap();

        assert_eq!(buf.text_string(), "hi world");
        let committed = buf.transaction_commit(txn.clone());

        buf.transaction_cancel(&txn).unwrap();
        assert_eq!(buf.text_string(), "hello");

        buf.apply_tx(&committed).unwrap();
        assert_eq!(buf.text_string(), "hi world");
    }

    #[test]
    fn buffer_stores_optional_uri() {
        let doc = DocId::new();
        let uri = DocUri::new("file:///tmp/hello.txt");
        let mut buf =
            TextBuffer::new_with_uri(doc, Some(uri.clone()), "hello".to_string()).unwrap();
        assert_eq!(buf.uri().map(DocUri::as_str), Some("file:///tmp/hello.txt"));

        buf.set_uri(None);
        assert_eq!(buf.uri(), None);

        buf.set_uri(Some(uri));
        assert_eq!(buf.uri().map(DocUri::as_str), Some("file:///tmp/hello.txt"));
    }
}
