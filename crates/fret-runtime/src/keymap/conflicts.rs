use crate::CommandId;
use std::collections::{HashMap, HashSet};

use super::{
    Keymap, KeymapBindingSignature, KeymapConflict, KeymapConflictEntry, KeymapConflictKind,
};

impl Keymap {
    /// Returns keymap conflicts, defined as multiple bindings sharing the same
    /// `(platform, when, sequence)` tuple (ADR 0021 section 7).
    ///
    /// This is intended for diagnostics and future UI reporting.
    pub fn conflicts(&self) -> Vec<KeymapConflict> {
        let mut by_sig: HashMap<KeymapBindingSignature, Vec<KeymapConflictEntry>> = HashMap::new();

        for (index, b) in self.bindings.iter().enumerate() {
            let sig = KeymapBindingSignature {
                platform: b.platform,
                sequence: b.sequence.clone(),
                when: b.when.clone(),
            };
            by_sig.entry(sig).or_default().push(KeymapConflictEntry {
                index,
                command: b.command.clone(),
            });
        }

        let mut out: Vec<KeymapConflict> = Vec::new();
        for (signature, mut entries) in by_sig {
            if entries.len() <= 1 {
                continue;
            }

            entries.sort_by_key(|e| e.index);

            let mut commands: HashSet<Option<&CommandId>> = HashSet::new();
            let mut any_unbind = false;
            for e in &entries {
                any_unbind |= e.command.is_none();
                commands.insert(e.command.as_ref());
            }

            let kind = if commands.len() <= 1 {
                KeymapConflictKind::Redundant
            } else if any_unbind {
                KeymapConflictKind::Unbind
            } else {
                KeymapConflictKind::Override
            };

            out.push(KeymapConflict {
                signature,
                kind,
                entries,
            });
        }

        out.sort_by(|a, b| {
            let ka = a.entries.last().map(|e| e.index).unwrap_or_default();
            let kb = b.entries.last().map(|e| e.index).unwrap_or_default();
            ka.cmp(&kb)
        });
        out
    }
}
