#[cfg(feature = "syntax")]
use super::*;
#[cfg(feature = "syntax")]
use std::collections::{HashSet, VecDeque};
#[cfg(feature = "syntax")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::editor) struct SyntaxSpan {
    /// Range within the row text (UTF-8 byte indices).
    pub(in crate::editor) range: Range<usize>,
    pub(in crate::editor) highlight: &'static str,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::editor) struct SyntaxPrefetchKey {
    pub(in crate::editor) doc: DocId,
    pub(in crate::editor) rev: fret_code_editor_buffer::Revision,
    pub(in crate::editor) language: Arc<str>,
    pub(in crate::editor) chunk_start: usize,
    pub(in crate::editor) chunk_end: usize,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
pub(in crate::editor) struct SyntaxPrefetchChunk {
    pub(in crate::editor) key: SyntaxPrefetchKey,
    pub(in crate::editor) rows: Arc<[(usize, Arc<[SyntaxSpan]>)]>,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Default)]
pub(in crate::editor) struct SyntaxPrefetchRuntimeState {
    pub(in crate::editor) pending: HashSet<SyntaxPrefetchKey>,
    pub(in crate::editor) ready: VecDeque<SyntaxPrefetchChunk>,
    last_visible_start: Option<usize>,
}

#[cfg(feature = "syntax")]
#[derive(Clone)]
pub(in crate::editor) struct SyntaxPrefetchRuntime {
    pub(in crate::editor) shared: Arc<Mutex<SyntaxPrefetchRuntimeState>>,
    pub(in crate::editor) dispatcher: DispatcherHandle,
}

#[cfg(feature = "syntax")]
impl std::fmt::Debug for SyntaxPrefetchRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxPrefetchRuntime")
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "syntax")]
impl SyntaxPrefetchRuntime {
    pub(in crate::editor) fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            shared: Arc::new(Mutex::new(SyntaxPrefetchRuntimeState::default())),
            dispatcher,
        }
    }

    pub(in crate::editor) fn clear(&self) {
        let mut state = self.lock_state();
        state.pending.clear();
        state.ready.clear();
        state.last_visible_start = None;
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, SyntaxPrefetchRuntimeState> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::editor) fn note_visible_start(&self, visible_start: usize) -> i8 {
        let mut state = self.lock_state();
        let direction = match state.last_visible_start {
            Some(prev) if visible_start < prev => -1,
            Some(prev) if visible_start > prev => 1,
            _ => 1,
        };
        state.last_visible_start = Some(visible_start);
        direction
    }

    pub(in crate::editor) fn drain_ready(&self) -> Vec<SyntaxPrefetchChunk> {
        let mut state = self.lock_state();
        state.ready.drain(..).collect()
    }

    pub(in crate::editor) fn try_mark_pending(&self, key: SyntaxPrefetchKey) -> bool {
        const MAX_PENDING: usize = 12;

        let mut state = self.lock_state();
        if state.pending.contains(&key) || state.ready.iter().any(|chunk| chunk.key == key) {
            return false;
        }
        if state.pending.len() >= MAX_PENDING {
            return false;
        }
        state.pending.insert(key)
    }
}
