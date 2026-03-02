use std::collections::HashMap;

use fret_core::{AppWindowId, FrameId};

use crate::{CommandId, TickId};

/// Best-effort classification of where a command dispatch originated.
///
/// This is diagnostics-only metadata intended to improve explainability in `fretboard diag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDispatchSourceKindV1 {
    Pointer,
    Keyboard,
    Shortcut,
    Programmatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDispatchSourceV1 {
    pub kind: CommandDispatchSourceKindV1,
    /// `GlobalElementId.0` (from `crates/fret-ui`) when available.
    pub element: Option<u64>,
}

impl CommandDispatchSourceV1 {
    pub fn programmatic() -> Self {
        Self {
            kind: CommandDispatchSourceKindV1::Programmatic,
            element: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDispatchDecisionV1 {
    pub seq: u64,
    pub frame_id: FrameId,
    pub tick_id: TickId,
    pub window: AppWindowId,
    pub command: CommandId,
    pub source: CommandDispatchSourceV1,
    pub handled: bool,
    /// `GlobalElementId.0` (from `crates/fret-ui`) for the first widget that handled the command.
    pub handled_by_element: Option<u64>,
    pub stopped: bool,
    pub started_from_focus: bool,
    pub used_default_root_fallback: bool,
}

#[derive(Default)]
pub struct WindowCommandDispatchDiagnosticsStore {
    next_seq: u64,
    per_window: HashMap<AppWindowId, Vec<CommandDispatchDecisionV1>>,
}

impl WindowCommandDispatchDiagnosticsStore {
    const MAX_ENTRIES_PER_WINDOW: usize = 128;

    pub fn record(&mut self, mut decision: CommandDispatchDecisionV1) {
        decision.seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);

        let entries = self.per_window.entry(decision.window).or_default();
        entries.push(decision);
        if entries.len() > Self::MAX_ENTRIES_PER_WINDOW {
            let extra = entries.len().saturating_sub(Self::MAX_ENTRIES_PER_WINDOW);
            entries.drain(0..extra);
        }
    }

    pub fn decisions_for_frame(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
        max_entries: usize,
    ) -> Vec<CommandDispatchDecisionV1> {
        let Some(entries) = self.per_window.get(&window) else {
            return Vec::new();
        };
        entries
            .iter()
            .rev()
            .filter(|e| e.frame_id == frame_id)
            .take(max_entries)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn snapshot_since(
        &self,
        window: AppWindowId,
        since_seq: u64,
        max_entries: usize,
    ) -> Vec<CommandDispatchDecisionV1> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingCommandDispatchSourceV1 {
    tick_id: TickId,
    window: AppWindowId,
    command: CommandId,
    source: CommandDispatchSourceV1,
}

/// Frame/tick-local source metadata for the next `Effect::Command` dispatch.
///
/// This is a diagnostics-only escape hatch so pointer-triggered dispatch (which is encoded via an
/// `Effect::Command`) can still be explained as “element X dispatched command Y”.
#[derive(Default)]
pub struct WindowPendingCommandDispatchSourceService {
    per_window: HashMap<AppWindowId, Vec<PendingCommandDispatchSourceV1>>,
}

impl WindowPendingCommandDispatchSourceService {
    const MAX_PENDING_PER_WINDOW: usize = 32;

    pub fn record(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        command: CommandId,
        source: CommandDispatchSourceV1,
    ) {
        let pending = PendingCommandDispatchSourceV1 {
            tick_id,
            window,
            command,
            source,
        };
        let entries = self.per_window.entry(window).or_default();
        entries.push(pending);
        if entries.len() > Self::MAX_PENDING_PER_WINDOW {
            let extra = entries.len().saturating_sub(Self::MAX_PENDING_PER_WINDOW);
            entries.drain(0..extra);
        }
    }

    pub fn consume(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        command: &CommandId,
    ) -> Option<CommandDispatchSourceV1> {
        let entries = self.per_window.get_mut(&window)?;

        // Drop stale pending entries that were recorded on previous ticks. We intentionally keep
        // this strict: pending context is only meant to bridge a single synchronous effect flush.
        entries.retain(|e| e.tick_id == tick_id);

        let pos = entries
            .iter()
            .rposition(|e| &e.command == command && e.window == window)?;
        Some(entries.remove(pos).source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_source_expires_across_ticks() {
        let mut svc = WindowPendingCommandDispatchSourceService::default();
        let window = AppWindowId::default();
        let cmd = CommandId::from("test.cmd");

        svc.record(
            window,
            TickId(10),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
            },
        );

        assert_eq!(
            svc.consume(window, TickId(10), &cmd),
            Some(CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
            })
        );

        svc.record(
            window,
            TickId(10),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
            },
        );

        assert_eq!(svc.consume(window, TickId(11), &cmd), None);
    }
}
