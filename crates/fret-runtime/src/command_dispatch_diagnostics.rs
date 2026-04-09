use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, FrameId};

use crate::{CommandId, CommandScope, TickId};

/// Best-effort classification of where a command dispatch originated.
///
/// This is diagnostics-only metadata intended to improve explainability in `fretboard-dev diag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDispatchSourceKindV1 {
    Pointer,
    Keyboard,
    Shortcut,
    Programmatic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDispatchSourceV1 {
    pub kind: CommandDispatchSourceKindV1,
    /// `GlobalElementId.0` (from `crates/fret-ui`) when available.
    pub element: Option<u64>,
    /// Best-effort stable selector for explainability (typically a semantics `test_id`).
    ///
    /// This is diagnostics-only metadata intended to make pointer-triggered `Effect::Command`
    /// dispatch explainable without requiring callers to correlate element IDs with a semantics
    /// snapshot.
    pub test_id: Option<Arc<str>>,
}

impl CommandDispatchSourceV1 {
    pub fn programmatic() -> Self {
        Self {
            kind: CommandDispatchSourceKindV1::Programmatic,
            element: None,
            test_id: None,
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
    /// Best-effort handler scope classification for explainability (ADR 0307).
    ///
    /// Notes:
    /// - `Some(CommandScope::Widget)` means the command was handled by bubbling widget dispatch.
    /// - For driver-handled commands, this is typically `Some(CommandScope::Window)` or
    ///   `Some(CommandScope::App)`.
    /// - `None` means the command was not handled (or the scope could not be determined).
    pub handled_by_scope: Option<CommandScope>,
    /// Whether the command was handled by a runner/driver integration layer (not by a UI element).
    pub handled_by_driver: bool,
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
    const PENDING_SOURCE_TTL_TICKS: u64 = 64;

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

        // Drop stale pending entries.
        //
        // This metadata is best-effort and diagnostics-only: in practice, effect-driven command
        // dispatch can be handled on a later tick (e.g. when the platform/backend defers effect
        // flushing, or when a UI interaction schedules work for a subsequent frame).
        //
        // Keep a small TTL window so pointer/keyboard-triggered dispatch remains explainable in
        // `fretboard-dev diag` without changing the `Effect::Command` schema.
        let min_tick = TickId(tick_id.0.saturating_sub(Self::PENDING_SOURCE_TTL_TICKS));
        entries.retain(|e| e.tick_id.0 >= min_tick.0 && e.tick_id.0 <= tick_id.0);

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
                test_id: None,
            },
        );

        assert_eq!(
            svc.consume(window, TickId(10), &cmd),
            Some(CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
                test_id: None,
            })
        );

        svc.record(
            window,
            TickId(10),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
                test_id: None,
            },
        );

        assert_eq!(
            svc.consume(window, TickId(11), &cmd),
            Some(CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
                test_id: None,
            })
        );

        svc.record(
            window,
            TickId(10),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(42),
                test_id: None,
            },
        );

        assert_eq!(svc.consume(window, TickId(80), &cmd), None);
    }

    #[test]
    fn pending_source_prefers_most_recent_match() {
        let mut svc = WindowPendingCommandDispatchSourceService::default();
        let window = AppWindowId::default();
        let cmd = CommandId::from("test.cmd");

        svc.record(
            window,
            TickId(10),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(1),
                test_id: None,
            },
        );
        svc.record(
            window,
            TickId(12),
            cmd.clone(),
            CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(2),
                test_id: None,
            },
        );

        assert_eq!(
            svc.consume(window, TickId(20), &cmd),
            Some(CommandDispatchSourceV1 {
                kind: CommandDispatchSourceKindV1::Pointer,
                element: Some(2),
                test_id: None,
            })
        );
    }
}
