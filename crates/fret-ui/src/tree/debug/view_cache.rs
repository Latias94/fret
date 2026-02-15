use super::super::*;

#[derive(Debug, Clone, Copy)]
pub struct UiDebugCacheRootStats {
    pub root: NodeId,
    pub element: Option<GlobalElementId>,
    pub reused: bool,
    pub contained_layout: bool,
    pub paint_replayed_ops: u32,
    pub reuse_reason: UiDebugCacheRootReuseReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugCacheRootReuseReason {
    FirstMount,
    NodeRecreated,
    MarkedReuseRoot,
    NotMarkedReuseRoot,
    CacheKeyMismatch,
    NeedsRerender,
    LayoutInvalidated,
    ManualCacheRoot,
}

impl UiDebugCacheRootReuseReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstMount => "first_mount",
            Self::NodeRecreated => "node_recreated",
            Self::MarkedReuseRoot => "marked_reuse_root",
            Self::NotMarkedReuseRoot => "not_marked_reuse_root",
            Self::CacheKeyMismatch => "cache_key_mismatch",
            Self::NeedsRerender => "needs_rerender",
            Self::LayoutInvalidated => "layout_invalidated",
            Self::ManualCacheRoot => "manual_cache_root",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::tree) struct DebugViewCacheRootRecord {
    pub(in crate::tree) root: NodeId,
    pub(in crate::tree) reused: bool,
    pub(in crate::tree) contained_layout: bool,
    pub(in crate::tree) reuse_reason: UiDebugCacheRootReuseReason,
}
