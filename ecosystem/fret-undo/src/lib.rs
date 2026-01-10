//! App-owned undo/redo infrastructure.
//!
//! This crate intentionally lives in `ecosystem/`:
//! - The UI runtime must not own a global undo stack (ADR 0136).
//! - Editors still benefit from a reusable history implementation with explicit transaction
//!   boundaries and coalescing semantics (ADR 0024).
//!
//! The core idea matches common editor patterns (Unity/Unreal/Godot):
//! - The app records committed transactions in a history stack.
//! - Undo/redo is performed by applying transactions via app-provided closures.

use std::sync::Arc;

/// Recommended command id for document/window-level undo.
pub const CMD_EDIT_UNDO: &str = "edit.undo";
/// Recommended command id for document/window-level redo.
pub const CMD_EDIT_REDO: &str = "edit.redo";

/// Coalescing key for continuous edits (dragging, scrubbing).
///
/// This is intentionally app-defined and data-first; it should use stable identities (ADR 0024).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoalesceKey(pub Arc<str>);

impl From<&'static str> for CoalesceKey {
    fn from(value: &'static str) -> Self {
        Self(Arc::from(value))
    }
}

impl From<String> for CoalesceKey {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl From<Arc<str>> for CoalesceKey {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

/// One committed undoable transaction in history.
#[derive(Debug, Clone)]
pub struct UndoRecord<T> {
    pub label: Option<Arc<str>>,
    pub coalesce_key: Option<CoalesceKey>,
    pub tx: T,
}

impl<T> UndoRecord<T> {
    pub fn new(tx: T) -> Self {
        Self {
            label: None,
            coalesce_key: None,
            tx,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn coalesce_key(mut self, key: impl Into<CoalesceKey>) -> Self {
        self.coalesce_key = Some(key.into());
        self
    }
}

/// A linear undo/redo history for app-defined transactions.
///
/// This type does not prescribe how transactions are created; callers are expected to use
/// explicit begin/update/commit/cancel boundaries (ADR 0024) and record only committed edits.
#[derive(Debug, Clone)]
pub struct UndoHistory<T> {
    undo: Vec<UndoRecord<T>>,
    redo: Vec<UndoRecord<T>>,
    limit: usize,
}

impl<T> Default for UndoHistory<T> {
    fn default() -> Self {
        Self::with_limit(128)
    }
}

impl<T> UndoHistory<T> {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
        self.truncate_to_limit();
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn peek_undo(&self) -> Option<&UndoRecord<T>> {
        self.undo.last()
    }

    pub fn peek_redo(&self) -> Option<&UndoRecord<T>> {
        self.redo.last()
    }

    /// Records a committed transaction, clearing redo history.
    pub fn record(&mut self, record: UndoRecord<T>) {
        self.undo.push(record);
        self.redo.clear();
        self.truncate_to_limit();
    }

    /// Records a committed transaction, coalescing into the previous undo entry when possible.
    ///
    /// This is intended for cases where a UI emits multiple committed edits that should appear as
    /// one history entry (e.g. repeated small nudges, slider commits).
    ///
    /// Coalescing rules:
    /// - only coalesce when the incoming record has a `coalesce_key`,
    /// - only coalesce when redo history is empty (linear history),
    /// - only coalesce when the last undo entry has the same `coalesce_key`,
    /// - "last wins": the previous transaction is replaced by the new one.
    pub fn record_or_coalesce(&mut self, record: UndoRecord<T>) {
        let Some(key) = record.coalesce_key.clone() else {
            self.record(record);
            return;
        };
        if !self.redo.is_empty() {
            self.record(record);
            return;
        }
        if let Some(prev) = self.undo.last_mut() {
            if prev.coalesce_key == Some(key) {
                *prev = record;
                return;
            }
        }
        self.record(record);
    }

    /// Undoes the latest committed transaction.
    ///
    /// `apply_undo` must apply the inverse of `record.tx` and return a transaction that should be
    /// pushed onto the redo stack.
    pub fn undo<E>(
        &mut self,
        mut apply_undo: impl FnMut(&UndoRecord<T>) -> Result<T, E>,
    ) -> Result<Option<UndoRecord<T>>, E> {
        let Some(record) = self.undo.pop() else {
            return Ok(None);
        };

        let redo_tx = apply_undo(&record)?;
        self.redo.push(UndoRecord {
            label: record.label.clone(),
            coalesce_key: record.coalesce_key.clone(),
            tx: redo_tx,
        });

        Ok(Some(record))
    }

    /// Redoes the latest undone transaction.
    ///
    /// `apply_redo` must apply `record.tx` and return a transaction that should be pushed onto the
    /// undo stack.
    pub fn redo<E>(
        &mut self,
        mut apply_redo: impl FnMut(&UndoRecord<T>) -> Result<T, E>,
    ) -> Result<Option<UndoRecord<T>>, E> {
        let Some(record) = self.redo.pop() else {
            return Ok(None);
        };

        let undo_tx = apply_redo(&record)?;
        self.undo.push(UndoRecord {
            label: record.label.clone(),
            coalesce_key: record.coalesce_key.clone(),
            tx: undo_tx,
        });
        self.truncate_to_limit();

        Ok(Some(record))
    }

    fn truncate_to_limit(&mut self) {
        if self.undo.len() > self.limit {
            let excess = self.undo.len() - self.limit;
            self.undo.drain(0..excess);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_clears_redo() {
        let mut h = UndoHistory::<u32>::with_limit(8);
        h.record(UndoRecord::new(1));
        let _ = h
            .undo(|rec| Ok::<u32, ()>(rec.tx))
            .expect("undo should not error");
        assert!(h.can_redo());
        h.record(UndoRecord::new(2));
        assert!(!h.can_redo());
    }

    #[test]
    fn record_or_coalesce_replaces_last() {
        let mut h = UndoHistory::<u32>::with_limit(8);
        h.record_or_coalesce(UndoRecord::new(1).coalesce_key("k"));
        h.record_or_coalesce(UndoRecord::new(2).coalesce_key("k"));
        assert_eq!(h.undo.len(), 1);
        assert_eq!(h.peek_undo().unwrap().tx, 2);
    }
}
