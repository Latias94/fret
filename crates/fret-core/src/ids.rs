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
    pub struct MaterialId;
}

/// Window-scoped view identifier used for "dirty view" tracking (GPUI-aligned).
///
/// v1: a view is defined at cache boundary granularity (a `ViewCache` root), so `ViewId` wraps the
/// runtime `NodeId` for that cache root.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(pub NodeId);

impl From<NodeId> for ViewId {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}

impl From<ViewId> for NodeId {
    fn from(value: ViewId) -> Self {
        value.0
    }
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
pub struct ShareSheetToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncomingOpenToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageUploadToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageUpdateToken(pub u64);

/// Stable pointer/contact identifier used for multi-pointer input routing.
///
/// `PointerId(0)` is reserved for the mouse pointer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointerId(pub u64);
