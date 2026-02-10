use std::collections::HashMap;

use fret_core::{AppWindowId, FrameId, KeyCode, Modifiers};

use crate::CommandId;

/// Diagnostics-only trace entries that explain how keydown shortcuts were routed.
///
/// This store is intended to support structured explainability in `fretboard diag` without
/// relying on ad-hoc logs or screenshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutRoutingPhase {
    PreDispatch,
    PostDispatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutRoutingOutcome {
    /// Shortcut matching was skipped because the key was reserved for IME while composing.
    ReservedForIme,
    /// Shortcut matching was deferred, and the widget path consumed the event.
    ConsumedByWidget,
    /// A command was matched and dispatched via an `Effect::Command`.
    CommandDispatched,
    /// A command matched but was disabled (so the event fell through to normal dispatch).
    CommandDisabled,
    /// A key chord started or continued a multi-keystroke shortcut sequence.
    SequenceContinuation,
    /// A shortcut sequence failed to match and the captured keystrokes were replayed.
    SequenceReplay,
    /// No shortcut matched this chord.
    NoMatch,
    /// Shortcut matching was unavailable (no keymap service).
    NoKeymap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutRoutingDecision {
    pub seq: u64,
    pub frame_id: FrameId,
    pub phase: ShortcutRoutingPhase,
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub deferred: bool,
    pub focus_is_text_input: bool,
    pub ime_composing: bool,
    pub pending_sequence_len: u32,
    pub outcome: ShortcutRoutingOutcome,
    pub command: Option<CommandId>,
    pub command_enabled: Option<bool>,
}

#[derive(Default)]
pub struct WindowShortcutRoutingDiagnosticsStore {
    next_seq: u64,
    per_window: HashMap<AppWindowId, Vec<ShortcutRoutingDecision>>,
}

impl WindowShortcutRoutingDiagnosticsStore {
    const MAX_ENTRIES_PER_WINDOW: usize = 128;

    pub fn record(&mut self, window: AppWindowId, mut decision: ShortcutRoutingDecision) {
        decision.seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        let entries = self.per_window.entry(window).or_default();
        entries.push(decision);
        if entries.len() > Self::MAX_ENTRIES_PER_WINDOW {
            let extra = entries.len().saturating_sub(Self::MAX_ENTRIES_PER_WINDOW);
            entries.drain(0..extra);
        }
    }

    pub fn snapshot_since(
        &self,
        window: AppWindowId,
        since_seq: u64,
        max_entries: usize,
    ) -> Vec<ShortcutRoutingDecision> {
        let Some(entries) = self.per_window.get(&window) else {
            return Vec::new();
        };
        entries
            .iter()
            .filter(|e| e.seq >= since_seq)
            .take(max_entries)
            .cloned()
            .collect()
    }
}
