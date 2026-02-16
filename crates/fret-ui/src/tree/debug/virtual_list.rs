use super::super::*;

#[derive(Debug, Clone, Copy)]
pub enum UiDebugVirtualListWindowSource {
    Layout,
    Prepaint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugVirtualListWindowShiftKind {
    None,
    Prefetch,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugVirtualListWindowShiftReason {
    ScrollOffset,
    ViewportResize,
    ItemsRevision,
    ScrollToItem,
    InputsChange,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugVirtualListWindowShiftApplyMode {
    RetainedReconcile,
    NonRetainedRerender,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugVirtualListWindow {
    pub source: UiDebugVirtualListWindowSource,
    pub node: NodeId,
    pub element: GlobalElementId,
    pub axis: fret_core::Axis,
    pub is_probe_layout: bool,
    pub items_len: usize,
    pub items_revision: u64,
    pub prev_items_revision: u64,
    pub measure_mode: crate::element::VirtualListMeasureMode,
    pub overscan: usize,
    pub estimate_row_height: Px,
    pub gap: Px,
    pub scroll_margin: Px,
    pub viewport: Px,
    pub prev_viewport: Px,
    pub offset: Px,
    pub prev_offset: Px,
    pub content_extent: Px,
    pub policy_key: u64,
    pub inputs_key: u64,
    pub window_range: Option<crate::virtual_list::VirtualRange>,
    pub prev_window_range: Option<crate::virtual_list::VirtualRange>,
    pub render_window_range: Option<crate::virtual_list::VirtualRange>,
    pub deferred_scroll_to_item: bool,
    pub deferred_scroll_consumed: bool,
    pub window_mismatch: bool,
    pub window_shift_kind: UiDebugVirtualListWindowShiftKind,
    pub window_shift_reason: Option<UiDebugVirtualListWindowShiftReason>,
    pub window_shift_apply_mode: Option<UiDebugVirtualListWindowShiftApplyMode>,
    pub window_shift_invalidation_detail: Option<UiDebugInvalidationDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugRetainedVirtualListReconcileKind {
    Prefetch,
    Escape,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugRetainedVirtualListReconcile {
    pub node: NodeId,
    pub element: GlobalElementId,
    pub reconcile_kind: UiDebugRetainedVirtualListReconcileKind,
    /// Wall-clock time spent reconciling this retained host (including mounting/unmounting items).
    pub reconcile_time_us: u32,
    pub prev_items: u32,
    pub next_items: u32,
    pub preserved_items: u32,
    pub attached_items: u32,
    pub detached_items: u32,
    /// Keep-alive bucket size before this reconcile (after loading element-local state).
    pub keep_alive_pool_len_before: u32,
    /// Number of items that were re-attached from the retained keep-alive bucket instead of being
    /// mounted from scratch.
    pub reused_from_keep_alive_items: u32,
    /// Number of detached items that were retained in the keep-alive bucket after the reconcile.
    pub kept_alive_items: u32,
    /// Number of items evicted from the keep-alive bucket due to budget.
    pub evicted_keep_alive_items: u32,
    /// Keep-alive bucket size after this reconcile (after applying detach/evict updates).
    pub keep_alive_pool_len_after: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugPrepaintActionKind {
    Invalidate,
    RequestRedraw,
    RequestAnimationFrame,
    VirtualListWindowShift,
    ChartSamplingWindowShift,
    NodeGraphCullWindowShift,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugPrepaintAction {
    pub node: NodeId,
    pub target: Option<NodeId>,
    pub kind: UiDebugPrepaintActionKind,
    pub invalidation: Option<Invalidation>,
    pub element: Option<GlobalElementId>,
    pub virtual_list_window_shift_kind: Option<UiDebugVirtualListWindowShiftKind>,
    pub virtual_list_window_shift_reason: Option<UiDebugVirtualListWindowShiftReason>,
    pub chart_sampling_window_key: Option<u64>,
    pub node_graph_cull_window_key: Option<u64>,
    pub frame_id: FrameId,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugVirtualListWindowShiftSample {
    pub frame_id: FrameId,
    pub source: UiDebugVirtualListWindowSource,
    pub node: NodeId,
    pub element: GlobalElementId,
    pub window_shift_kind: UiDebugVirtualListWindowShiftKind,
    pub window_shift_reason: UiDebugVirtualListWindowShiftReason,
    pub window_shift_apply_mode: UiDebugVirtualListWindowShiftApplyMode,
    pub window_shift_invalidation_detail: Option<UiDebugInvalidationDetail>,
    pub prev_window_range: Option<crate::virtual_list::VirtualRange>,
    pub window_range: Option<crate::virtual_list::VirtualRange>,
    pub render_window_range: Option<crate::virtual_list::VirtualRange>,
}
