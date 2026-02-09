use crate::{CommandId, InputContext, KeyChord};
use std::collections::HashSet;

use super::{Binding, Keymap, KeymapContinuation, SequenceMatch};

impl Keymap {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push_binding(&mut self, binding: Binding) {
        self.bindings.push(binding);
    }

    /// Last-wins resolution. If a later binding matches and its `command` is `None`, the key is
    /// explicitly unbound and resolution stops.
    pub fn resolve(&self, ctx: &InputContext, chord: KeyChord) -> Option<CommandId> {
        for b in self.bindings.iter().rev() {
            if b.sequence.as_slice() != [chord] {
                continue;
            }
            if !b.platform.matches(ctx.platform) {
                continue;
            }
            if let Some(expr) = b.when.as_ref()
                && !expr.eval(ctx)
            {
                continue;
            }
            return b.command.clone();
        }
        None
    }

    /// Sequence matching helper used by pending multi-stroke bindings (ADR 0043).
    pub fn match_sequence(&self, ctx: &InputContext, sequence: &[KeyChord]) -> SequenceMatch {
        let mut exact: Option<Option<CommandId>> = None;
        let mut has_continuation = false;

        // Track full sequences we've already evaluated to preserve last-wins semantics for
        // continuations and exact matches under the current context.
        let mut seen: HashSet<Vec<KeyChord>> = HashSet::new();

        for b in self.bindings.iter().rev() {
            if !b.platform.matches(ctx.platform) {
                continue;
            }
            if let Some(expr) = b.when.as_ref()
                && !expr.eval(ctx)
            {
                continue;
            }

            if b.sequence.len() < sequence.len() {
                continue;
            }
            if b.sequence.get(0..sequence.len()) != Some(sequence) {
                continue;
            }

            if !seen.insert(b.sequence.clone()) {
                continue;
            }

            if b.sequence.len() == sequence.len() {
                if exact.is_none() {
                    exact = Some(b.command.clone());
                }
            } else if b.command.is_some() {
                has_continuation = true;
            }

            if exact.is_some() && has_continuation {
                break;
            }
        }

        SequenceMatch {
            exact,
            has_continuation,
        }
    }

    /// Lists the valid "next" keystrokes that can follow the provided prefix under the given
    /// input context.
    ///
    /// This is intended for UI hint overlays (e.g. a leader-key popup): it enumerates candidate
    /// next chords from the configured bindings, then uses `match_sequence` to filter down to
    /// chords that either execute a command or have further continuations.
    pub fn continuations(
        &self,
        ctx: &InputContext,
        prefix: &[KeyChord],
    ) -> Vec<KeymapContinuation> {
        if prefix.is_empty() {
            return Vec::new();
        }

        let mut candidates: Vec<KeyChord> = Vec::new();
        let mut seen: HashSet<KeyChord> = HashSet::new();

        for b in self.bindings.iter().rev() {
            if !b.platform.matches(ctx.platform) {
                continue;
            }
            if let Some(expr) = b.when.as_ref()
                && !expr.eval(ctx)
            {
                continue;
            }
            if b.sequence.len() <= prefix.len() {
                continue;
            }
            if b.sequence.get(0..prefix.len()) != Some(prefix) {
                continue;
            }

            let next = b.sequence[prefix.len()];
            if seen.insert(next) {
                candidates.push(next);
            }
        }

        let mut out: Vec<KeymapContinuation> = Vec::new();
        for next in candidates {
            let mut seq: Vec<KeyChord> = Vec::with_capacity(prefix.len() + 1);
            seq.extend_from_slice(prefix);
            seq.push(next);

            let matched = self.match_sequence(ctx, &seq);
            let exact_command = matched.exact.clone().flatten();
            if exact_command.is_some() || matched.has_continuation {
                out.push(KeymapContinuation { next, matched });
            }
        }

        out
    }

    pub fn extend(&mut self, other: Keymap) {
        self.bindings.extend(other.bindings);
    }
}
