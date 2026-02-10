use crate::{CommandId, KeyChord, Platform, WhenExpr};

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
    pub(super) bindings: Vec<Binding>,
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
