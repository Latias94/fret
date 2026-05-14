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
pub(crate) struct UiDebugVirtualListWindowShiftClassificationInput {
    pub view_cache_active: bool,
    pub retained_host: bool,
    pub window_shift_kind: UiDebugVirtualListWindowShiftKind,
    pub deferred_scroll_to_item: bool,
    pub items_revision: u64,
    pub prev_items_revision: u64,
    pub viewport: Px,
    pub prev_viewport: Px,
    pub offset: Px,
    pub prev_offset: Px,
    pub visible_range: Option<crate::virtual_list::VirtualRange>,
    pub prev_window_range: Option<crate::virtual_list::VirtualRange>,
    pub render_window_range: Option<crate::virtual_list::VirtualRange>,
    pub window_range: Option<crate::virtual_list::VirtualRange>,
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

pub(crate) fn classify_virtual_list_window_shift(
    input: UiDebugVirtualListWindowShiftClassificationInput,
) -> (
    Option<UiDebugVirtualListWindowShiftReason>,
    Option<UiDebugVirtualListWindowShiftApplyMode>,
    Option<UiDebugInvalidationDetail>,
) {
    if input.window_shift_kind == UiDebugVirtualListWindowShiftKind::None {
        return (None, None, None);
    }

    let reason = if input.deferred_scroll_to_item {
        UiDebugVirtualListWindowShiftReason::ScrollToItem
    } else if (input.viewport.0 - input.prev_viewport.0).abs() > 0.01 {
        UiDebugVirtualListWindowShiftReason::ViewportResize
    } else if virtual_range_inputs_changed(input.render_window_range, input.window_range) {
        UiDebugVirtualListWindowShiftReason::InputsChange
    } else if input.items_revision != input.prev_items_revision {
        UiDebugVirtualListWindowShiftReason::ItemsRevision
    } else if (input.offset.0 - input.prev_offset.0).abs() > 0.01 {
        UiDebugVirtualListWindowShiftReason::ScrollOffset
    } else if rendered_window_no_longer_covers_visible(
        input.render_window_range,
        input.visible_range,
    ) {
        UiDebugVirtualListWindowShiftReason::ScrollOffset
    } else if virtual_range_inputs_changed(input.prev_window_range, input.window_range) {
        UiDebugVirtualListWindowShiftReason::InputsChange
    } else {
        UiDebugVirtualListWindowShiftReason::Unknown
    };

    let mode = if input.retained_host {
        UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile
    } else {
        UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender
    };

    let invalidation_detail = if input.view_cache_active && !input.retained_host {
        Some(match reason {
            UiDebugVirtualListWindowShiftReason::ScrollToItem => {
                UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate
            }
            UiDebugVirtualListWindowShiftReason::ViewportResize => {
                UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate
            }
            UiDebugVirtualListWindowShiftReason::ItemsRevision => {
                UiDebugInvalidationDetail::ScrollHandleItemsRevisionWindowUpdate
            }
            UiDebugVirtualListWindowShiftReason::InputsChange => {
                UiDebugInvalidationDetail::ScrollHandleInputsChangeWindowUpdate
            }
            _ => fallback_virtual_list_window_shift_detail(input.window_shift_kind),
        })
    } else {
        None
    };

    (Some(reason), Some(mode), invalidation_detail)
}

pub(crate) fn fallback_virtual_list_window_shift_detail(
    kind: UiDebugVirtualListWindowShiftKind,
) -> UiDebugInvalidationDetail {
    match kind {
        UiDebugVirtualListWindowShiftKind::None => UiDebugInvalidationDetail::Unknown,
        UiDebugVirtualListWindowShiftKind::Prefetch => {
            UiDebugInvalidationDetail::ScrollHandlePrefetchWindowUpdate
        }
        UiDebugVirtualListWindowShiftKind::Escape => {
            UiDebugInvalidationDetail::ScrollHandleWindowUpdate
        }
    }
}

fn virtual_range_inputs_changed(
    previous: Option<crate::virtual_list::VirtualRange>,
    current: Option<crate::virtual_list::VirtualRange>,
) -> bool {
    previous.map(|range| (range.count, range.overscan))
        != current.map(|range| (range.count, range.overscan))
}

fn rendered_window_no_longer_covers_visible(
    rendered: Option<crate::virtual_list::VirtualRange>,
    visible: Option<crate::virtual_list::VirtualRange>,
) -> bool {
    let (Some(rendered), Some(visible)) = (rendered, visible) else {
        return false;
    };
    let rendered_start = rendered.start_index.saturating_sub(rendered.overscan);
    let rendered_end =
        (rendered.end_index + rendered.overscan).min(rendered.count.saturating_sub(1));
    visible.start_index < rendered_start || visible.end_index > rendered_end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(
        start_index: usize,
        end_index: usize,
        overscan: usize,
        count: usize,
    ) -> crate::virtual_list::VirtualRange {
        crate::virtual_list::VirtualRange {
            start_index,
            end_index,
            overscan,
            count,
        }
    }

    #[test]
    fn virtual_list_window_shift_classification_prioritizes_structural_inputs() {
        let (reason, apply_mode, detail) =
            classify_virtual_list_window_shift(UiDebugVirtualListWindowShiftClassificationInput {
                view_cache_active: true,
                retained_host: false,
                window_shift_kind: UiDebugVirtualListWindowShiftKind::Escape,
                deferred_scroll_to_item: false,
                items_revision: 12,
                prev_items_revision: 11,
                viewport: Px(240.0),
                prev_viewport: Px(240.0),
                offset: Px(720.0),
                prev_offset: Px(720.0),
                visible_range: Some(range(72, 95, 0, 111)),
                prev_window_range: Some(range(72, 95, 10, 50_000)),
                render_window_range: Some(range(72, 95, 10, 50_000)),
                window_range: Some(range(72, 95, 10, 111)),
            });

        assert_eq!(
            reason,
            Some(UiDebugVirtualListWindowShiftReason::InputsChange)
        );
        assert_eq!(
            apply_mode,
            Some(UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender)
        );
        assert_eq!(
            detail,
            Some(UiDebugInvalidationDetail::ScrollHandleInputsChangeWindowUpdate)
        );
    }

    #[test]
    fn virtual_list_window_shift_classification_keeps_retained_detail_side_effect_free() {
        let (reason, apply_mode, detail) =
            classify_virtual_list_window_shift(UiDebugVirtualListWindowShiftClassificationInput {
                view_cache_active: true,
                retained_host: true,
                window_shift_kind: UiDebugVirtualListWindowShiftKind::Escape,
                deferred_scroll_to_item: false,
                items_revision: 12,
                prev_items_revision: 11,
                viewport: Px(240.0),
                prev_viewport: Px(240.0),
                offset: Px(720.0),
                prev_offset: Px(720.0),
                visible_range: Some(range(72, 95, 0, 111)),
                prev_window_range: Some(range(72, 95, 10, 50_000)),
                render_window_range: Some(range(72, 95, 10, 50_000)),
                window_range: Some(range(72, 95, 10, 111)),
            });

        assert_eq!(
            reason,
            Some(UiDebugVirtualListWindowShiftReason::InputsChange)
        );
        assert_eq!(
            apply_mode,
            Some(UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile)
        );
        assert_eq!(detail, None);
    }
}
