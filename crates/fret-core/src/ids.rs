use slotmap::{KeyData, new_key_type};

new_key_type! {
    pub struct AppWindowId;
    pub struct NodeId;
    pub struct DockNodeId;
    pub struct ImageId;
    pub struct SvgId;
    pub struct FontId;
    pub struct TextBlobId;
    pub struct PathId;
    pub struct RenderTargetId;
}

impl FontId {
    /// Built-in "system UI" font alias (implementation-defined).
    ///
    /// Note: this is a semantic alias, not a numeric font database id.
    pub fn ui() -> Self {
        Self::default()
    }

    /// Built-in serif font alias (implementation-defined).
    pub fn serif() -> Self {
        Self::from(KeyData::from_ffi(1))
    }

    /// Built-in monospace font alias (implementation-defined).
    pub fn monospace() -> Self {
        Self::from(KeyData::from_ffi(2))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TickId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalDropToken(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileDialogToken(pub u64);
