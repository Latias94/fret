use crate::{CommandId, InputContext, KeyChord, Platform, WhenExpr};
use fret_core::{KeyCode, Modifiers};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub chord: KeyChord,
    pub when: Option<WhenExpr>,
}

#[derive(Debug, Default, Clone)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Keymap {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeymapError> {
        let parsed: KeymapFileAny =
            serde_json::from_slice(bytes).map_err(|source| KeymapError::ParseFailed { source })?;
        Self::from_any(parsed)
    }

    pub fn from_v1(file: KeymapFileV1) -> Result<Self, KeymapError> {
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
                Some(
                    WhenExpr::parse(when)
                        .map_err(|e| KeymapError::WhenParseFailed { index, error: e })?,
                )
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

    fn from_any(file: KeymapFileAny) -> Result<Self, KeymapError> {
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
                        Some(
                            WhenExpr::parse(when)
                                .map_err(|e| KeymapError::WhenParseFailed { index, error: e })?,
                        )
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
                        Some(
                            WhenExpr::parse(when)
                                .map_err(|e| KeymapError::WhenParseFailed { index, error: e })?,
                        )
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

    pub fn extend(&mut self, other: Keymap) {
        self.bindings.extend(other.bindings);
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
}

#[derive(Debug, Default)]
pub struct KeymapService {
    pub keymap: Keymap,
}

fn parse_keys(index: usize, keys: KeySpecV1) -> Result<KeyChord, KeymapError> {
    let key = KeyCode::from_token(&keys.key).ok_or_else(|| KeymapError::UnknownKeyToken {
        index,
        token: keys.key.clone(),
    })?;

    let mut mods = Modifiers::default();
    for m in keys.mods {
        match m.as_str() {
            "shift" => mods.shift = true,
            "ctrl" => mods.ctrl = true,
            "alt" => mods.alt = true,
            "altgr" => mods.alt_gr = true,
            "meta" => mods.meta = true,
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
