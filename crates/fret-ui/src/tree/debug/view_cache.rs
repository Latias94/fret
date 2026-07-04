use super::super::*;

#[derive(Debug, Clone, Copy)]
pub struct UiDebugCacheRootStats {
    pub root: NodeId,
    pub element: Option<GlobalElementId>,
    pub reused: bool,
    pub layout_dependency: &'static str,
    pub paint_replayed_ops: u32,
    pub reuse_reason: UiDebugCacheRootReuseReason,
}

#[derive(Debug, Clone)]
pub struct UiDebugBoundaryStats {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub element: Option<GlobalElementId>,
    pub topology_epoch: u64,
    pub kind: &'static str,
    pub source: &'static str,
    pub prepaint_owner: &'static str,
    pub hit_test_bounds_owner: &'static str,
    pub semantics_subtree_owner: &'static str,
    pub interaction_cache_owner: &'static str,
    pub paint_cache_owner: &'static str,
    pub scene_fragment_owner: &'static str,
    pub scene_fragment_slots: usize,
    pub scene_fragment_entries: usize,
    pub scene_fragment_chunks: usize,
    pub scene_fragment_fingerprint: u64,
    pub scene_fragment_used_entries: usize,
    pub scene_fragment_rejected_entries: usize,
    pub scene_fragment_reject_reason: Option<&'static str>,
    pub layout_dependency: &'static str,
    pub layout_definite: bool,
    pub layout_dirty: bool,
    pub layout_dirty_source: Option<UiDebugInvalidationSource>,
    pub layout_dirty_detail: Option<UiDebugInvalidationDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugCacheRootReuseReason {
    FirstMount,
    NodeRecreated,
    MarkedReuseRoot,
    /// View caching is disabled globally (`UiTree::view_cache_enabled=false`).
    ViewCacheDisabled,
    /// View caching is disabled for correctness during inspection/picking (`UiTree::inspection_active=true`).
    InspectionActive,
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
            Self::ViewCacheDisabled => "view_cache_disabled",
            Self::InspectionActive => "inspection_active",
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
    pub(in crate::tree) layout_dependency: &'static str,
    pub(in crate::tree) reuse_reason: UiDebugCacheRootReuseReason,
}
