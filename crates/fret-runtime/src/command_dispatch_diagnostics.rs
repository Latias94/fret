use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, FrameId};

use crate::{CommandId, CommandScope, TickId};

/// Best-effort provenance for where a command dispatch originated.
///
/// The UI runtime may use a still-live `element` to seed command routing and to validate that an
/// owner-first hook originated inside the active modal input scope. `kind` and `test_id` retain
/// their diagnostics role. Missing, stale, or ambiguous element identity must fail closed for
/// modal provenance and fall back to normal focus/root routing for widget dispatch.
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
    /// This keeps UI-triggered `Effect::Command` dispatch explainable without requiring callers
    /// to correlate element IDs with a semantics snapshot.
    pub test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDispatchOutcomeV1 {
    /// Canonical typed action identity, when known.
    ///
    /// This identifies the domain action represented by the outcome. It is not dispatch-source
    /// provenance; [`CommandDispatchSourceV1`] carries that separately. The routed `command` may
    /// be an alias or shell command while this field retains the canonical typed identity.
    pub action_id: Option<CommandId>,
    /// Domain-owned target identity, such as `pane-a/doc-a` for workspace commands.
    pub target: Option<Arc<str>>,
    pub applied: bool,
    pub blocked_dirty_close: bool,
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
    pub outcome: Option<CommandDispatchOutcomeV1>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingCommandDispatchOutcomeV1 {
    tick_id: TickId,
    window: AppWindowId,
    command: CommandId,
    outcome: CommandDispatchOutcomeV1,
}

#[derive(Default)]
pub struct WindowPendingCommandDispatchOutcomeService {
    per_window: HashMap<AppWindowId, Vec<PendingCommandDispatchOutcomeV1>>,
}

impl WindowPendingCommandDispatchOutcomeService {
    const MAX_PENDING_PER_WINDOW: usize = 32;
    const PENDING_OUTCOME_TTL_TICKS: u64 = 64;

    pub fn record(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        command: CommandId,
        outcome: CommandDispatchOutcomeV1,
    ) {
        let entries = self.per_window.entry(window).or_default();
        entries.push(PendingCommandDispatchOutcomeV1 {
            tick_id,
            window,
            command,
            outcome,
        });
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
    ) -> Option<CommandDispatchOutcomeV1> {
        let entries = self.per_window.get_mut(&window)?;
        let min_tick = TickId(tick_id.0.saturating_sub(Self::PENDING_OUTCOME_TTL_TICKS));
        entries.retain(|entry| entry.tick_id.0 >= min_tick.0 && entry.tick_id.0 <= tick_id.0);
        let position = entries
            .iter()
            .position(|entry| &entry.command == command && entry.window == window)?;
        Some(entries.remove(position).outcome)
    }
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
    entry_id: u64,
    tick_id: TickId,
    window: AppWindowId,
    command: CommandId,
    source: CommandDispatchSourceV1,
}

/// Opaque identity for one source reinserted with
/// [`WindowPendingCommandDispatchSourceService::restore_next`].
///
/// A dispatcher that tentatively restores a source can use this ticket to remove only that entry
/// if a downstream route did not consume it. Later occurrences of the same command remain intact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingCommandDispatchSourceTicket {
    window: AppWindowId,
    entry_id: u64,
}

/// Frame/tick-local provenance for the next `Effect::Command` dispatch.
///
/// `Effect::Command` does not carry its origin directly, so UI actions record provenance here for
/// both live source-element routing and diagnostics. Consumers that tentatively inspect an entry
/// before delegating to the next routing layer must restore it with [`Self::restore_next`].
#[derive(Default)]
pub struct WindowPendingCommandDispatchSourceService {
    per_window: HashMap<AppWindowId, Vec<PendingCommandDispatchSourceV1>>,
    next_entry_id: u64,
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
        let entry_id = self.allocate_entry_id();
        let pending = PendingCommandDispatchSourceV1 {
            entry_id,
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

    /// Restore a source that was tentatively consumed by a dispatcher before handing the same
    /// command to the next routing layer.
    ///
    /// The restored entry must remain ahead of later same-command effects, so this inserts at the
    /// front instead of behaving like a newly recorded dispatch.
    pub fn restore_next(
        &mut self,
        window: AppWindowId,
        tick_id: TickId,
        command: CommandId,
        source: CommandDispatchSourceV1,
    ) -> PendingCommandDispatchSourceTicket {
        let entry_id = self.allocate_entry_id();
        let entries = self.per_window.entry(window).or_default();
        entries.insert(
            0,
            PendingCommandDispatchSourceV1 {
                entry_id,
                tick_id,
                window,
                command,
                source,
            },
        );
        entries.truncate(Self::MAX_PENDING_PER_WINDOW);
        PendingCommandDispatchSourceTicket { window, entry_id }
    }

    /// Remove the exact source previously returned by [`Self::restore_next`].
    ///
    /// Returns `false` when a downstream dispatcher already consumed the restored entry. Unlike
    /// [`Self::consume`], this never removes a later occurrence that happens to share a command ID.
    pub fn discard_restored(&mut self, ticket: PendingCommandDispatchSourceTicket) -> bool {
        let Some(entries) = self.per_window.get_mut(&ticket.window) else {
            return false;
        };
        let Some(pos) = entries
            .iter()
            .position(|entry| entry.entry_id == ticket.entry_id)
        else {
            return false;
        };
        entries.remove(pos);
        if entries.is_empty() {
            self.per_window.remove(&ticket.window);
        }
        true
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
            .position(|e| &e.command == command && e.window == window)?;
        Some(entries.remove(pos).source)
    }

    fn allocate_entry_id(&mut self) -> u64 {
        let entry_id = self.next_entry_id;
        self.next_entry_id = self.next_entry_id.wrapping_add(1);
        entry_id
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
    fn pending_source_preserves_fifo_order_for_repeated_command() {
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
                element: Some(1),
                test_id: None,
            })
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

    #[test]
    fn restored_source_stays_ahead_of_later_same_command_dispatches() {
        let mut svc = WindowPendingCommandDispatchSourceService::default();
        let window = AppWindowId::default();
        let command = CommandId::from("test.cmd");
        for element in [1, 2] {
            svc.record(
                window,
                TickId(10),
                command.clone(),
                CommandDispatchSourceV1 {
                    kind: CommandDispatchSourceKindV1::Pointer,
                    element: Some(element),
                    test_id: None,
                },
            );
        }
        let first = svc
            .consume(window, TickId(10), &command)
            .expect("first source");
        let ticket = svc.restore_next(window, TickId(10), command.clone(), first);

        assert_eq!(
            svc.consume(window, TickId(10), &command)
                .and_then(|source| source.element),
            Some(1)
        );
        assert!(!svc.discard_restored(ticket));
        assert_eq!(
            svc.consume(window, TickId(10), &command)
                .and_then(|source| source.element),
            Some(2)
        );
    }

    #[test]
    fn restored_ticket_discards_only_the_reinserted_occurrence() {
        let mut svc = WindowPendingCommandDispatchSourceService::default();
        let window = AppWindowId::default();
        let command = CommandId::from("test.cmd");
        for element in [1, 2] {
            svc.record(
                window,
                TickId(10),
                command.clone(),
                CommandDispatchSourceV1 {
                    kind: CommandDispatchSourceKindV1::Pointer,
                    element: Some(element),
                    test_id: None,
                },
            );
        }
        let first = svc
            .consume(window, TickId(10), &command)
            .expect("first source");
        let ticket = svc.restore_next(window, TickId(10), command.clone(), first);

        assert!(svc.discard_restored(ticket));
        assert_eq!(
            svc.consume(window, TickId(10), &command)
                .and_then(|source| source.element),
            Some(2)
        );
    }

    #[test]
    fn pending_outcome_preserves_fifo_order_for_repeated_command() {
        let mut svc = WindowPendingCommandDispatchOutcomeService::default();
        let window = AppWindowId::default();
        let command = CommandId::from("test.cmd");
        for target in ["first", "second"] {
            svc.record(
                window,
                TickId(10),
                command.clone(),
                CommandDispatchOutcomeV1 {
                    action_id: Some(command.clone()),
                    target: Some(Arc::from(target)),
                    applied: true,
                    blocked_dirty_close: false,
                },
            );
        }

        assert_eq!(
            svc.consume(window, TickId(10), &command)
                .and_then(|outcome| outcome.target),
            Some(Arc::from("first"))
        );
        assert_eq!(
            svc.consume(window, TickId(10), &command)
                .and_then(|outcome| outcome.target),
            Some(Arc::from("second"))
        );
    }
}
