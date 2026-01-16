use std::collections::HashMap;

use fret_core::AppWindowId;
use fret_runtime::{CommandId, InputContext, KeyChord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingShortcutContinuation {
    pub next: KeyChord,
    pub command: Option<CommandId>,
    pub has_continuation: bool,
}

#[derive(Debug, Clone)]
struct PendingShortcutOverlayEntry {
    pub input_ctx: InputContext,
    pub sequence: Vec<KeyChord>,
    pub continuations: Vec<PendingShortcutContinuation>,
}

#[derive(Debug, Default)]
pub struct PendingShortcutOverlayState {
    by_window: HashMap<AppWindowId, PendingShortcutOverlayEntry>,
}

impl PendingShortcutOverlayState {
    pub fn snapshot_for_window(
        &self,
        window: AppWindowId,
    ) -> Option<(&InputContext, &[KeyChord], &[PendingShortcutContinuation])> {
        self.by_window.get(&window).map(|entry| {
            (
                &entry.input_ctx,
                entry.sequence.as_slice(),
                entry.continuations.as_slice(),
            )
        })
    }

    pub fn set_sequence(
        &mut self,
        window: AppWindowId,
        input_ctx: InputContext,
        sequence: Vec<KeyChord>,
        continuations: Vec<PendingShortcutContinuation>,
    ) {
        if sequence.is_empty() {
            self.by_window.remove(&window);
        } else {
            self.by_window.insert(
                window,
                PendingShortcutOverlayEntry {
                    input_ctx,
                    sequence,
                    continuations,
                },
            );
        }
    }
}
