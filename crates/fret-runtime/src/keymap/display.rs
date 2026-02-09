use crate::{CommandId, InputContext, InputDispatchPhase, KeyChord};
use std::collections::{HashMap, HashSet};

use super::Keymap;

impl Keymap {
    /// Best-effort reverse lookup for UI display (command palette / menus).
    ///
    /// This applies the same platform + `when` matching rules as `resolve`, then finds any chord
    /// whose effective command equals `command` under the provided context.
    pub fn shortcut_for_command(
        &self,
        ctx: &InputContext,
        command: &CommandId,
    ) -> Option<KeyChord> {
        self.shortcut_for_command_sequence(ctx, command)
            .filter(|seq| seq.len() == 1)
            .and_then(|seq| seq.first().copied())
    }

    pub fn shortcut_for_command_sequence(
        &self,
        ctx: &InputContext,
        command: &CommandId,
    ) -> Option<Vec<KeyChord>> {
        let mut order: Vec<Vec<KeyChord>> = Vec::new();
        let mut seen: HashSet<Vec<KeyChord>> = HashSet::new();
        let mut effective: HashMap<Vec<KeyChord>, Option<CommandId>> = HashMap::new();

        for b in &self.bindings {
            if !b.platform.matches(ctx.platform) {
                continue;
            }
            if let Some(expr) = b.when.as_ref()
                && !expr.eval(ctx)
            {
                continue;
            }
            if seen.insert(b.sequence.clone()) {
                order.push(b.sequence.clone());
            }
            effective.insert(b.sequence.clone(), b.command.clone());
        }

        order.into_iter().find(|seq| {
            effective
                .get(seq)
                .is_some_and(|c| c.as_ref() == Some(command))
        })
    }

    /// Best-effort reverse lookup for UI display that is intentionally *stable* across focus
    /// changes.
    ///
    /// This is intended for menu bar / command palette shortcut labels, where displaying different
    /// shortcuts as focus moves is confusing. Instead of using the live focus state, we evaluate
    /// bindings against a small set of "default" contexts derived from `base`:
    ///
    /// - non-modal + not text input
    /// - non-modal + text input
    /// - modal + not text input
    /// - modal + text input
    ///
    /// Candidate sequences are ranked by:
    ///
    /// 1. first matching default context (earlier is preferred),
    /// 2. shorter sequences (single-chord preferred),
    /// 3. later-defined bindings (user/project overrides preferred).
    pub fn display_shortcut_for_command(
        &self,
        base: &InputContext,
        command: &CommandId,
    ) -> Option<KeyChord> {
        self.display_shortcut_for_command_sequence(base, command)
            .filter(|seq| seq.len() == 1)
            .and_then(|seq| seq.first().copied())
    }

    pub fn display_shortcut_for_command_sequence(
        &self,
        base: &InputContext,
        command: &CommandId,
    ) -> Option<Vec<KeyChord>> {
        #[derive(Debug)]
        struct Candidate {
            ctx_index: usize,
            seq_len: usize,
            binding_index: usize,
            seq: Vec<KeyChord>,
        }

        fn default_display_contexts(base: &InputContext) -> [InputContext; 4] {
            let mut c0 = base.clone();
            c0.dispatch_phase = InputDispatchPhase::Bubble;
            c0.ui_has_modal = false;
            c0.focus_is_text_input = false;

            let mut c1 = c0.clone();
            c1.focus_is_text_input = true;

            let mut c2 = c0.clone();
            c2.ui_has_modal = true;
            c2.focus_is_text_input = false;

            let mut c3 = c2.clone();
            c3.focus_is_text_input = true;

            [c0, c1, c2, c3]
        }

        fn effective_command_for_sequence<'a>(
            keymap: &'a Keymap,
            ctx: &InputContext,
            seq: &[KeyChord],
        ) -> Option<(Option<&'a CommandId>, usize)> {
            for (index, b) in keymap.bindings.iter().enumerate().rev() {
                if b.sequence.as_slice() != seq {
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
                return Some((b.command.as_ref(), index));
            }
            None
        }

        let contexts = default_display_contexts(base);

        let mut sequences: HashSet<Vec<KeyChord>> = HashSet::new();
        for b in &self.bindings {
            sequences.insert(b.sequence.clone());
        }

        let mut best: Option<Candidate> = None;
        for seq in sequences.into_iter() {
            for (ctx_index, ctx) in contexts.iter().enumerate() {
                let Some((Some(cmd), binding_index)) =
                    effective_command_for_sequence(self, ctx, &seq)
                else {
                    continue;
                };
                if cmd != command {
                    continue;
                }

                let cand = Candidate {
                    ctx_index,
                    seq_len: seq.len(),
                    binding_index,
                    seq,
                };

                best = match best {
                    None => Some(cand),
                    Some(prev) => {
                        let replace = (
                            cand.ctx_index,
                            cand.seq_len,
                            std::cmp::Reverse(cand.binding_index),
                        ) < (
                            prev.ctx_index,
                            prev.seq_len,
                            std::cmp::Reverse(prev.binding_index),
                        );
                        if replace { Some(cand) } else { Some(prev) }
                    }
                };

                break;
            }
        }

        best.map(|c| c.seq)
    }
}
