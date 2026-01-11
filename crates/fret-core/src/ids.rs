use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct AppWindowId;
    pub struct NodeId;
    pub struct DockNodeId;
    pub struct ImageId;
    pub struct SvgId;
    pub struct TextBlobId;
    pub struct PathId;
    pub struct RenderTargetId;
}

/// Stable, portable font identifier used by the UI/runtime.
///
/// This is intentionally a semantic identifier (not a font database index) so that it remains
/// stable across runs and portable to wasm/sandboxed environments.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontId {
    /// Built-in "system UI" font alias (implementation-defined).
    #[default]
    Ui,
    /// Built-in serif font alias (implementation-defined).
    Serif,
    /// Built-in monospace font alias (implementation-defined).
    Monospace,
    /// A named font family resolved by the backend (best-effort).
    Family(String),
}

impl FontId {
    pub fn ui() -> Self {
        Self::Ui
    }

    pub fn serif() -> Self {
        Self::Serif
    }

    pub fn monospace() -> Self {
        Self::Monospace
    }

    pub fn family(name: impl Into<String>) -> Self {
        Self::Family(name.into())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalDropToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileDialogToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipboardToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageUploadToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageUpdateToken(pub u64);
