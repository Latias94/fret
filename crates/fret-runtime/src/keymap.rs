use crate::{CommandId, InputContext, InputDispatchPhase, KeyChord, Platform, WhenExpr};
use fret_core::{KeyCode, Modifiers};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformFilter {
    All,
    Macos,
    Windows,
    Linux,
    Web,
}

impl PlatformFilter {
    pub fn matches(self, platform: Platform) -> bool {
        match self {
            Self::All => true,
            Self::Macos => platform == Platform::Macos,
            Self::Windows => platform == Platform::Windows,
            Self::Linux => platform == Platform::Linux,
            Self::Web => platform == Platform::Web,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub platform: PlatformFilter,
    pub sequence: Vec<KeyChord>,
    pub when: Option<WhenExpr>,
    pub command: Option<CommandId>,
}

#[derive(Debug, Clone)]
pub struct DefaultKeybinding {
    pub platform: PlatformFilter,
    pub sequence: Vec<KeyChord>,
    pub when: Option<WhenExpr>,
}

impl DefaultKeybinding {
    pub fn single(platform: PlatformFilter, chord: KeyChord) -> Self {
        Self {
            platform,
            sequence: vec![chord],
            when: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeymapBindingSignature {
    pub platform: PlatformFilter,
    pub sequence: Vec<KeyChord>,
    pub when: Option<WhenExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeymapConflictKind {
    /// Multiple bindings exist, but they all resolve to the same command payload.
    Redundant,
    /// Later bindings override earlier ones (last-wins) with a different command payload.
    Override,
    /// At least one binding explicitly unbinds (`command: null`), shadowing earlier bindings.
    Unbind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapConflictEntry {
    pub index: usize,
    pub command: Option<CommandId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapConflict {
    pub signature: KeymapBindingSignature,
    pub kind: KeymapConflictKind,
    /// Oldest -> newest (so the effective winner is `entries.last()`).
    pub entries: Vec<KeymapConflictEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenValidationMode {
    Strict,
    Lenient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeymapLoadOptions {
    pub when_validation: WhenValidationMode,
}

impl Default for KeymapLoadOptions {
    fn default() -> Self {
        Self {
            when_validation: WhenValidationMode::Strict,
        }
    }
}

impl Keymap {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeymapError> {
        Self::from_bytes_with_options(bytes, KeymapLoadOptions::default())
    }

    pub fn from_bytes_with_options(
        bytes: &[u8],
        options: KeymapLoadOptions,
    ) -> Result<Self, KeymapError> {
        let parsed: KeymapFileAny =
            serde_json::from_slice(bytes).map_err(|source| KeymapError::ParseFailed { source })?;
        Self::from_any(parsed, options)
    }

    pub fn from_v1(file: KeymapFileV1) -> Result<Self, KeymapError> {
        Self::from_v1_with_options(file, KeymapLoadOptions::default())
    }

    pub fn from_v1_with_options(
        file: KeymapFileV1,
        options: KeymapLoadOptions,
    ) -> Result<Self, KeymapError> {
        if file.keymap_version != 1 {
            return Err(KeymapError::UnsupportedVersion(file.keymap_version));
        }

        let mut out = Keymap::empty();
        for (index, b) in file.bindings.into_iter().enumerate() {
            let platform = match b.platform.as_deref().unwrap_or("all") {
                "all" => PlatformFilter::All,
                "macos" => PlatformFilter::Macos,
                "windows" => PlatformFilter::Windows,
                "linux" => PlatformFilter::Linux,
                "web" => PlatformFilter::Web,
                other => {
                    return Err(KeymapError::UnknownPlatform {
                        index,
                        value: other.into(),
                    });
                }
            };

            let chord = parse_keys(index, b.keys)?;

            let when = if let Some(when) = b.when.as_deref() {
                Some(parse_when(index, when, options.when_validation)?)
            } else {
                None
            };

            let command = b.command.map(CommandId::new);

            out.push_binding(Binding {
                platform,
                sequence: vec![chord],
                when,
                command,
            });
        }

        Ok(out)
    }

    fn from_any(file: KeymapFileAny, options: KeymapLoadOptions) -> Result<Self, KeymapError> {
        match file.keymap_version {
            1 => {
                let mut out = Keymap::empty();
                for (index, b) in file.bindings.into_iter().enumerate() {
                    let platform = match b.platform.as_deref().unwrap_or("all") {
                        "all" => PlatformFilter::All,
                        "macos" => PlatformFilter::Macos,
                        "windows" => PlatformFilter::Windows,
                        "linux" => PlatformFilter::Linux,
                        "web" => PlatformFilter::Web,
                        other => {
                            return Err(KeymapError::UnknownPlatform {
                                index,
                                value: other.into(),
                            });
                        }
                    };

                    let KeysAny::Single(keys) = b.keys else {
                        return Err(KeymapError::UnsupportedVersion(1));
                    };

                    let chord = parse_keys(index, keys)?;

                    let when = if let Some(when) = b.when.as_deref() {
                        Some(parse_when(index, when, options.when_validation)?)
                    } else {
                        None
                    };

                    let command = b.command.map(CommandId::new);

                    out.push_binding(Binding {
                        platform,
                        sequence: vec![chord],
                        when,
                        command,
                    });
                }
                Ok(out)
            }
            2 => {
                let mut out = Keymap::empty();
                for (index, b) in file.bindings.into_iter().enumerate() {
                    let platform = match b.platform.as_deref().unwrap_or("all") {
                        "all" => PlatformFilter::All,
                        "macos" => PlatformFilter::Macos,
                        "windows" => PlatformFilter::Windows,
                        "linux" => PlatformFilter::Linux,
                        "web" => PlatformFilter::Web,
                        other => {
                            return Err(KeymapError::UnknownPlatform {
                                index,
                                value: other.into(),
                            });
                        }
                    };

                    let key_specs = match b.keys {
                        KeysAny::Single(keys) => vec![keys],
                        KeysAny::Sequence(seq) => seq,
                    };
                    if key_specs.is_empty() {
                        return Err(KeymapError::EmptyKeys { index });
                    }

                    let mut sequence: Vec<KeyChord> = Vec::with_capacity(key_specs.len());
                    for keys in key_specs {
                        sequence.push(parse_keys(index, keys)?);
                    }

                    let when = if let Some(when) = b.when.as_deref() {
                        Some(parse_when(index, when, options.when_validation)?)
                    } else {
                        None
                    };

                    let command = b.command.map(CommandId::new);

                    out.push_binding(Binding {
                        platform,
                        sequence,
                        when,
                        command,
                    });
                }
                Ok(out)
            }
            other => Err(KeymapError::UnsupportedVersion(other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SequenceMatch {
    /// `Some(Some(cmd))` if an exact binding exists and is bound, `Some(None)` if explicitly unbound,
    /// and `None` if no exact binding exists under the provided context.
    pub exact: Option<Option<CommandId>>,
    /// True if any longer binding exists that starts with the provided sequence under the same context.
    pub has_continuation: bool,
}

#[derive(Debug, Clone)]
pub struct KeymapContinuation {
    pub next: KeyChord,
    pub matched: SequenceMatch,
}

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

#[derive(Debug, Default)]
pub struct KeymapService {
    pub keymap: Keymap,
}

fn parse_keys(index: usize, keys: KeySpecV1) -> Result<KeyChord, KeymapError> {
    let key: KeyCode = keys.key.parse().map_err(|_| KeymapError::UnknownKeyToken {
        index,
        token: keys.key.clone(),
    })?;

    let mut mods = Modifiers::default();
    for m in keys.mods {
        let token = m.to_ascii_lowercase();
        match token.as_str() {
            "shift" => mods.shift = true,
            "ctrl" | "control" => mods.ctrl = true,
            "alt" | "option" => mods.alt = true,
            "altgr" | "alt_gr" | "altgraph" => mods.alt_gr = true,
            "meta" | "cmd" | "command" => mods.meta = true,
            other => {
                return Err(KeymapError::UnknownModifier {
                    index,
                    value: other.into(),
                });
            }
        }
    }
    Ok(KeyChord::new(key, mods))
}

fn parse_when(index: usize, when: &str, mode: WhenValidationMode) -> Result<WhenExpr, KeymapError> {
    let expr =
        WhenExpr::parse(when).map_err(|e| KeymapError::WhenParseFailed { index, error: e })?;
    if mode == WhenValidationMode::Strict {
        expr.validate()
            .map_err(|e| KeymapError::WhenValidationFailed {
                index,
                error: e.to_string(),
            })?;
    }
    Ok(expr)
}

#[derive(Debug, thiserror::Error)]
pub enum KeymapError {
    #[error("failed to read keymap file")]
    ReadFailed { source: std::io::Error },
    #[error("failed to parse keymap json")]
    ParseFailed { source: serde_json::Error },
    #[error("unsupported keymap_version {0}")]
    UnsupportedVersion(u32),
    #[error("unknown platform value at binding[{index}]: {value}")]
    UnknownPlatform { index: usize, value: String },
    #[error("unknown key token at binding[{index}]: {token}")]
    UnknownKeyToken { index: usize, token: String },
    #[error("unknown modifier at binding[{index}]: {value}")]
    UnknownModifier { index: usize, value: String },
    #[error("empty keys sequence at binding[{index}]")]
    EmptyKeys { index: usize },
    #[error("failed to parse when at binding[{index}]: {error}")]
    WhenParseFailed { index: usize, error: String },
    #[error("invalid when expression at binding[{index}]: {error}")]
    WhenValidationFailed { index: usize, error: String },
}

#[derive(Debug, Deserialize)]
pub struct KeymapFileV1 {
    pub keymap_version: u32,
    pub bindings: Vec<BindingV1>,
}

#[derive(Debug, Deserialize)]
pub struct BindingV1 {
    pub command: Option<String>,
    pub platform: Option<String>,
    pub when: Option<String>,
    pub keys: KeySpecV1,
}

#[derive(Debug, Deserialize)]
pub struct KeySpecV1 {
    pub mods: Vec<String>,
    pub key: String,
}

#[derive(Debug, Deserialize)]
struct KeymapFileAny {
    pub keymap_version: u32,
    pub bindings: Vec<BindingAny>,
}

#[derive(Debug, Deserialize)]
struct BindingAny {
    pub command: Option<String>,
    pub platform: Option<String>,
    pub when: Option<String>,
    pub keys: KeysAny,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum KeysAny {
    Single(KeySpecV1),
    Sequence(Vec<KeySpecV1>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{KeyCode, Modifiers};
    use std::sync::Arc;

    #[test]
    fn keymap_rejects_unknown_when_identifiers() {
        let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                {
                    "command": "test.command",
                    "keys": { "mods": [], "key": "KeyA" },
                    "when": "ui.multi_windo"
                }
            ]
        }"#;

        let err = Keymap::from_bytes(bytes).unwrap_err();
        assert!(matches!(
            err,
            KeymapError::WhenValidationFailed { index: 0, .. }
        ));
    }

    #[test]
    fn keymap_accepts_modifier_tokens_case_insensitive_and_aliases() {
        let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                { "command": "test.shift", "keys": { "mods": ["Shift"], "key": "Tab" } },
                { "command": "test.ctrl", "keys": { "mods": ["Control"], "key": "KeyA" } },
                { "command": "test.alt", "keys": { "mods": ["Option"], "key": "KeyB" } },
                { "command": "test.meta", "keys": { "mods": ["Command"], "key": "KeyC" } },
                { "command": "test.alt_gr", "keys": { "mods": ["Alt_Gr"], "key": "KeyD" } }
            ]
        }"#;

        Keymap::from_bytes(bytes).expect("keymap parses");
    }

    #[test]
    fn keymap_rejects_string_keys_used_as_boolean_when() {
        let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                {
                    "command": "test.command",
                    "keys": { "mods": [], "key": "KeyA" },
                    "when": "dnd.external_payload"
                }
            ]
        }"#;

        let err = Keymap::from_bytes(bytes).unwrap_err();
        assert!(matches!(
            err,
            KeymapError::WhenValidationFailed { index: 0, .. }
        ));
    }

    #[test]
    fn keymap_conflicts_detects_last_wins_overrides() {
        let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                { "command": "test.a", "keys": { "mods": ["ctrl"], "key": "KeyP" } },
                { "command": "test.b", "keys": { "mods": ["ctrl"], "key": "KeyP" } }
            ]
        }"#;

        let km = Keymap::from_bytes(bytes).unwrap();
        let conflicts = km.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, KeymapConflictKind::Override);
        assert_eq!(conflicts[0].entries.len(), 2);
        assert_eq!(
            conflicts[0].entries[0].command.as_ref().unwrap().as_str(),
            "test.a"
        );
        assert_eq!(
            conflicts[0].entries[1].command.as_ref().unwrap().as_str(),
            "test.b"
        );
    }

    #[test]
    fn keymap_continuations_list_valid_next_chords_and_filters_unbound() {
        let ctrl_k = KeyChord::new(
            KeyCode::KeyK,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let up = KeyChord::new(KeyCode::ArrowUp, Modifiers::default());
        let down_shift = KeyChord::new(
            KeyCode::ArrowDown,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );

        let mut km = Keymap::empty();
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_k, up],
            when: None,
            command: Some(CommandId::new(Arc::<str>::from("test.up"))),
        });
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_k, down_shift],
            when: None,
            command: Some(CommandId::new(Arc::<str>::from("test.down_shift"))),
        });

        let ctx = InputContext {
            platform: Platform::Windows,
            ..Default::default()
        };

        let out = km.continuations(&ctx, &[ctrl_k]);
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|c| {
            c.next == up
                && c.matched
                    .exact
                    .as_ref()
                    .is_some_and(|c| c.as_ref().is_some_and(|id| id.as_str() == "test.up"))
        }));
        assert!(out.iter().any(|c| c.next == down_shift
            && c.matched.exact.as_ref().is_some_and(|c| {
                c.as_ref()
                    .is_some_and(|id| id.as_str() == "test.down_shift")
            })));

        // Explicitly unbind one of the continuations: it should no longer be listed.
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_k, up],
            when: None,
            command: None,
        });

        let out = km.continuations(&ctx, &[ctrl_k]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].next, down_shift);
    }

    #[test]
    fn keymap_display_shortcut_prefers_non_modal_non_text_context() {
        let mut km = Keymap::empty();
        let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

        let ctrl_p = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let ctrl_e = KeyChord::new(
            KeyCode::KeyE,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );

        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_e],
            when: Some(WhenExpr::parse("focus.is_text_input").unwrap()),
            command: Some(cmd.clone()),
        });
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_p],
            when: None,
            command: Some(cmd.clone()),
        });

        let base = InputContext {
            platform: Platform::Windows,
            ui_has_modal: true,
            focus_is_text_input: true,
            ..Default::default()
        };

        let out = km
            .display_shortcut_for_command_sequence(&base, &cmd)
            .unwrap();
        assert_eq!(out, vec![ctrl_p]);
    }

    #[test]
    fn keymap_display_shortcut_falls_back_to_modal_context_when_needed() {
        let mut km = Keymap::empty();
        let cmd = CommandId::new(Arc::<str>::from("test.modal_only"));

        let esc = KeyChord::new(KeyCode::Escape, Modifiers::default());
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![esc],
            when: Some(WhenExpr::parse("ui.has_modal").unwrap()),
            command: Some(cmd.clone()),
        });

        let base = InputContext {
            platform: Platform::Windows,
            ui_has_modal: false,
            focus_is_text_input: false,
            ..Default::default()
        };

        let out = km
            .display_shortcut_for_command_sequence(&base, &cmd)
            .unwrap();
        assert_eq!(out, vec![esc]);
    }

    #[test]
    fn keymap_display_shortcut_prefers_later_overrides_for_the_same_command() {
        let mut km = Keymap::empty();
        let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

        let ctrl_p = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let ctrl_shift_p = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        );

        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_p],
            when: None,
            command: Some(cmd.clone()),
        });
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_shift_p],
            when: None,
            command: Some(cmd.clone()),
        });

        let base = InputContext {
            platform: Platform::Windows,
            ..Default::default()
        };

        let out = km
            .display_shortcut_for_command_sequence(&base, &cmd)
            .unwrap();
        assert_eq!(out, vec![ctrl_shift_p]);
    }

    #[test]
    fn keymap_display_shortcut_ignores_explicit_unbinds() {
        let mut km = Keymap::empty();
        let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

        let ctrl_p = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let ctrl_shift_p = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        );

        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_p],
            when: None,
            command: Some(cmd.clone()),
        });
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_p],
            when: None,
            command: None,
        });
        km.push_binding(Binding {
            platform: PlatformFilter::All,
            sequence: vec![ctrl_shift_p],
            when: None,
            command: Some(cmd.clone()),
        });

        let base = InputContext {
            platform: Platform::Windows,
            ..Default::default()
        };

        let out = km
            .display_shortcut_for_command_sequence(&base, &cmd)
            .unwrap();
        assert_eq!(out, vec![ctrl_shift_p]);
    }
}
