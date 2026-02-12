use super::*;
use fret_core::TimerToken;
use fret_runtime::ModelCreatedDebugInfo;

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugFrameStats {
    pub frame_id: FrameId,
    /// Approximate capacity retained by per-frame scratch (“frame arena”) containers.
    ///
    /// This is a coarse signal intended for diagnostics/triage: it tracks reserved capacity
    /// (not current length) and intentionally underestimates hash table overhead.
    pub frame_arena_capacity_estimate_bytes: u64,
    /// Number of scratch containers that grew their capacity during the current frame.
    ///
    /// This is a proxy for allocator churn in hot paths that should ideally stabilize after
    /// warmup.
    pub frame_arena_grow_events: u32,
    /// Number of child-element vectors reused from the per-window pool during element build.
    ///
    /// This is a proxy for “element tree arena” progress: higher reuse implies less allocator churn
    /// while building the ephemeral declarative element tree.
    pub element_children_vec_pool_reuses: u32,
    /// Number of child-element vectors that had to be newly allocated during element build.
    pub element_children_vec_pool_misses: u32,
    /// Total time spent in event dispatch during the current frame.
    ///
    /// This includes pointer routing, capture/focus arbitration, and widget event hooks. It does
    /// not include layout/prepaint/paint, which are tracked separately.
    pub dispatch_time: Duration,
    /// Number of pointer/drag events dispatched during the current frame.
    pub dispatch_pointer_events: u32,
    /// Total wall time spent dispatching pointer/drag events during the current frame.
    pub dispatch_pointer_event_time: Duration,
    /// Number of timer events dispatched during the current frame.
    pub dispatch_timer_events: u32,
    /// Total wall time spent dispatching timer events during the current frame.
    pub dispatch_timer_event_time: Duration,
    /// Number of timer events that resolved to an explicit element target.
    pub dispatch_timer_targeted_events: u32,
    /// Wall time spent dispatching explicitly targeted timer events.
    pub dispatch_timer_targeted_time: Duration,
    /// Number of timer events that fell back to broadcast delivery (no element target).
    pub dispatch_timer_broadcast_events: u32,
    /// Wall time spent in broadcast timer delivery (including layer scanning + dispatch).
    pub dispatch_timer_broadcast_time: Duration,
    /// Total number of layers visited during broadcast timer delivery.
    pub dispatch_timer_broadcast_layers_visited: u32,
    /// Time spent rebuilding the visible-layers scratch list for broadcast timer delivery.
    pub dispatch_timer_broadcast_rebuild_visible_layers_time: Duration,
    /// Time spent iterating candidate layers and dispatching broadcast timers.
    pub dispatch_timer_broadcast_loop_time: Duration,
    /// Slowest single timer event routing time observed in the current frame.
    pub dispatch_timer_slowest_event_time: Duration,
    /// Token of the slowest timer event observed in the current frame (if any).
    pub dispatch_timer_slowest_token: Option<TimerToken>,
    /// Whether the slowest timer event used the broadcast fallback.
    pub dispatch_timer_slowest_was_broadcast: bool,
    /// Number of non-pointer, non-timer events dispatched during the current frame.
    pub dispatch_other_events: u32,
    /// Total wall time spent dispatching non-pointer, non-timer events during the current frame.
    pub dispatch_other_event_time: Duration,
    /// Time spent inside hit-testing during the current frame (subset of `dispatch_time`).
    pub hit_test_time: Duration,
    /// Number of events dispatched during the current frame.
    pub dispatch_events: u32,
    /// Number of hit-test queries executed during the current frame.
    pub hit_test_queries: u32,
    /// Count of bounds-tree queries attempted during hit testing.
    pub hit_test_bounds_tree_queries: u32,
    /// Bounds-tree queries that were disabled (e.g. env-gated, layer not indexed, or unsupported transforms).
    pub hit_test_bounds_tree_disabled: u32,
    /// Bounds-tree queries that missed (no containing leaf).
    pub hit_test_bounds_tree_misses: u32,
    /// Bounds-tree queries that returned a candidate leaf.
    pub hit_test_bounds_tree_hits: u32,
    /// Bounds-tree candidates rejected by `hit_test_node_self_only`, forcing a fallback traversal.
    pub hit_test_bounds_tree_candidate_rejected: u32,
    /// Total bounds-tree nodes visited across all queries in the current frame.
    pub hit_test_bounds_tree_nodes_visited: u32,
    /// Total bounds-tree nodes pushed to the search stack across all queries in the current frame.
    pub hit_test_bounds_tree_nodes_pushed: u32,
    /// Number of hit-test queries that reused the cached path (no bounds-tree query needed).
    pub hit_test_path_cache_hits: u32,
    /// Number of hit-test queries that fell back to bounds-tree or full traversal (cache miss).
    pub hit_test_path_cache_misses: u32,
    /// Total wall time spent attempting the cached-path hit-test fast path in the current frame.
    pub hit_test_cached_path_time: Duration,
    /// Total wall time spent querying the bounds-tree index in the current frame.
    pub hit_test_bounds_tree_query_time: Duration,
    /// Total wall time spent validating bounds-tree candidates (`hit_test_node_self_only`) in the current frame.
    pub hit_test_candidate_self_only_time: Duration,
    /// Total wall time spent in full traversal fallback hit-testing in the current frame.
    pub hit_test_fallback_traversal_time: Duration,
    /// Total wall time spent updating hover state from pointer hit-testing in the current frame.
    pub dispatch_hover_update_time: Duration,
    /// Total wall time spent applying scroll-handle binding invalidations during event dispatch.
    pub dispatch_scroll_handle_invalidation_time: Duration,
    /// Total wall time spent computing active input layers and enforcing modal barrier scope.
    pub dispatch_active_layers_time: Duration,
    /// Total wall time spent constructing and publishing the window input context snapshot.
    pub dispatch_input_context_time: Duration,
    /// Total wall time spent building the event chain during dispatch.
    pub dispatch_event_chain_build_time: Duration,
    /// Total wall time spent delivering capture-phase widget events during dispatch.
    pub dispatch_widget_capture_time: Duration,
    /// Total wall time spent delivering bubble-phase widget events during dispatch.
    pub dispatch_widget_bubble_time: Duration,
    /// Total wall time spent computing the cursor icon from the pointer hit during dispatch.
    pub dispatch_cursor_query_time: Duration,
    /// Total wall time spent dispatching pointer-move layer observers (both post-dispatch and when
    /// pointer dispatch is suppressed).
    pub dispatch_pointer_move_layer_observers_time: Duration,
    /// Total wall time spent dispatching the synthetic hover-move observer chain when the pointer crosses targets.
    pub dispatch_synth_hover_observer_time: Duration,
    /// Total wall time spent pushing cursor-icon effects during dispatch.
    pub dispatch_cursor_effect_time: Duration,
    /// Total wall time spent publishing post-dispatch window integration snapshots (input context,
    /// command availability) during dispatch.
    pub dispatch_post_dispatch_snapshot_time: Duration,
    pub layout_time: Duration,
    pub layout_collect_roots_time: Duration,
    pub layout_invalidate_scroll_handle_bindings_time: Duration,
    pub layout_expand_view_cache_invalidations_time: Duration,
    pub layout_request_build_roots_time: Duration,
    pub layout_pending_barrier_relayouts_time: Duration,
    pub layout_repair_view_cache_bounds_time: Duration,
    pub layout_contained_view_cache_roots_time: Duration,
    pub layout_collapse_layout_observations_time: Duration,
    pub layout_prepaint_after_layout_time: Duration,
    pub layout_skipped_engine_frame: bool,
    pub layout_roots_time: Duration,
    pub layout_barrier_relayouts_time: Duration,
    pub layout_view_cache_time: Duration,
    pub layout_semantics_refresh_time: Duration,
    pub layout_focus_repair_time: Duration,
    pub layout_deferred_cleanup_time: Duration,
    pub prepaint_time: Duration,
    pub paint_time: Duration,
    pub paint_record_visual_bounds_time: Duration,
    pub paint_record_visual_bounds_calls: u32,
    /// Total wall time spent computing paint-cache keys and paint-cache enablement checks.
    pub paint_cache_key_time: Duration,
    /// Total wall time spent checking paint-cache hit eligibility (excluding replay itself).
    pub paint_cache_hit_check_time: Duration,
    /// Total wall time spent executing `Widget::paint()` (including push/pop transforms).
    pub paint_widget_time: Duration,
    /// Total wall time spent recording paint observations (`observed_in_paint` + globals).
    pub paint_observation_record_time: Duration,
    /// Total wall time spent iterating element-runtime observed models inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_models_time: Duration,
    /// Total observed-model edges iterated inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_models_items: u32,
    /// Total wall time spent iterating element-runtime observed globals inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_globals_time: Duration,
    /// Total observed-global edges iterated inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_globals_items: u32,
    /// Total wall time spent resolving the element instance (`ElementInstance`) inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_instance_lookup_time: Duration,
    /// Number of `ElementInstance` lookups performed inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_instance_lookup_calls: u32,
    /// Total wall time spent preparing text blobs (`TextSystem::prepare`) during `Widget::paint`.
    pub paint_text_prepare_time: Duration,
    /// Number of text blob preparations performed during `Widget::paint`.
    pub paint_text_prepare_calls: u32,
    /// Count of text prepares where the cached blob was missing.
    pub paint_text_prepare_reason_blob_missing: u32,
    /// Count of text prepares triggered by a scale factor change.
    pub paint_text_prepare_reason_scale_changed: u32,
    /// Count of text prepares triggered by a plain-text change.
    pub paint_text_prepare_reason_text_changed: u32,
    /// Count of text prepares triggered by an attributed-text change.
    pub paint_text_prepare_reason_rich_changed: u32,
    /// Count of text prepares triggered by a text-style change.
    pub paint_text_prepare_reason_style_changed: u32,
    /// Count of text prepares triggered by a wrap-mode change.
    pub paint_text_prepare_reason_wrap_changed: u32,
    /// Count of text prepares triggered by an overflow-mode change.
    pub paint_text_prepare_reason_overflow_changed: u32,
    /// Count of text prepares triggered by a max-width change.
    pub paint_text_prepare_reason_width_changed: u32,
    /// Count of text prepares triggered by a font-stack-key change.
    pub paint_text_prepare_reason_font_stack_changed: u32,
    pub paint_input_context_time: Duration,
    pub paint_scroll_handle_invalidation_time: Duration,
    pub paint_collect_roots_time: Duration,
    pub paint_publish_text_input_snapshot_time: Duration,
    pub paint_collapse_observations_time: Duration,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    /// Paint-cache replay attempts that were allowed specifically by the
    /// `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` gate.
    pub paint_cache_hit_test_only_replay_allowed: u32,
    /// Hit-test-only replay attempts rejected because the previous cache key did not match.
    pub paint_cache_hit_test_only_replay_rejected_key_mismatch: u32,
    pub paint_cache_replay_time: Duration,
    pub paint_cache_bounds_translate_time: Duration,
    pub paint_cache_bounds_translated_nodes: u32,
    pub interaction_cache_hits: u32,
    pub interaction_cache_misses: u32,
    pub interaction_cache_replayed_records: u32,
    pub interaction_records: u32,
    /// Number of layout engine root solves performed during the current frame.
    pub layout_engine_solves: u64,
    /// Total time spent in layout engine solves during the current frame.
    pub layout_engine_solve_time: Duration,
    /// Total number of `layout_engine_child_local_rect` queries performed during the current frame.
    pub layout_engine_child_rect_queries: u64,
    /// Total wall time spent inside layout engine child-rect queries during the current frame.
    pub layout_engine_child_rect_time: Duration,
    /// Number of "widget-local" layout engine solves triggered as a fallback when a widget cannot
    /// consume already-solved engine child rects.
    ///
    /// The goal for v2 is to keep this at `0` for normal UI trees by ensuring explicit layout
    /// barriers (scroll/virtualization/splits/...) register viewport roots or explicitly solve
    /// their child roots.
    pub layout_engine_widget_fallback_solves: u64,
    pub layout_fast_path_taken: bool,
    pub layout_invalidations_count: u32,
    /// Unique nodes observed as invalidation roots for model changes during the current frame.
    pub model_change_invalidation_roots: u32,
    /// Count of changed models consumed for propagation during the current frame.
    pub model_change_models: u32,
    /// Total (model -> node) observation edges scanned during propagation.
    pub model_change_observation_edges: u32,
    /// Count of changed models with no observation edges.
    pub model_change_unobserved_models: u32,
    /// Unique nodes observed as invalidation roots for global changes during the current frame.
    pub global_change_invalidation_roots: u32,
    /// Count of changed globals consumed for propagation during the current frame.
    pub global_change_globals: u32,
    /// Total (global -> node) observation edges scanned during propagation.
    pub global_change_observation_edges: u32,
    /// Count of changed globals with no observation edges.
    pub global_change_unobserved_globals: u32,
    /// Total nodes visited across invalidation walks during the current frame.
    pub invalidation_walk_nodes: u32,
    /// Total invalidation walks performed during the current frame.
    pub invalidation_walk_calls: u32,
    /// Nodes visited across invalidation walks attributed to model changes.
    pub invalidation_walk_nodes_model_change: u32,
    /// Invalidation walks attributed to model changes.
    pub invalidation_walk_calls_model_change: u32,
    /// Nodes visited across invalidation walks attributed to global changes.
    pub invalidation_walk_nodes_global_change: u32,
    /// Invalidation walks attributed to global changes.
    pub invalidation_walk_calls_global_change: u32,
    /// Nodes visited across invalidation walks attributed to hover state changes.
    pub invalidation_walk_nodes_hover: u32,
    /// Invalidation walks attributed to hover state changes.
    pub invalidation_walk_calls_hover: u32,
    /// Nodes visited across invalidation walks attributed to focus changes.
    pub invalidation_walk_nodes_focus: u32,
    /// Invalidation walks attributed to focus changes.
    pub invalidation_walk_calls_focus: u32,
    /// Nodes visited across invalidation walks attributed to all other sources.
    pub invalidation_walk_nodes_other: u32,
    /// Invalidation walks attributed to all other sources.
    pub invalidation_walk_calls_other: u32,
    /// Count of hover target changes for `Pressable` instances during the current frame.
    pub hover_pressable_target_changes: u32,
    /// Count of hover target changes for `HoverRegion` instances during the current frame.
    pub hover_hover_region_target_changes: u32,
    /// Count of declarative instance changes that happened in a frame that also observed a hover
    /// target change.
    pub hover_declarative_instance_changes: u32,
    /// Count of declarative `HitTest` invalidations attributed to hover during the current frame.
    pub hover_declarative_hit_test_invalidations: u32,
    /// Count of declarative `Layout` invalidations attributed to hover during the current frame.
    pub hover_declarative_layout_invalidations: u32,
    /// Count of declarative `Paint` invalidations attributed to hover during the current frame.
    pub hover_declarative_paint_invalidations: u32,
    /// Whether view-cache mode is active for this frame.
    pub view_cache_active: bool,
    /// How many invalidation walks were truncated by a view-cache boundary.
    pub view_cache_invalidation_truncations: u32,
    /// How many "contained" view-cache roots were re-laid out during the final pass.
    pub view_cache_contained_relayouts: u32,
    /// How many view-cache roots were observed during the current frame.
    pub view_cache_roots_total: u32,
    /// How many view-cache roots were reused during the current frame.
    pub view_cache_roots_reused: u32,
    /// View-cache roots that were not reused because they were mounted for the first time.
    pub view_cache_roots_first_mount: u32,
    /// View-cache roots that were not reused because their backing `NodeId` was recreated.
    pub view_cache_roots_node_recreated: u32,
    /// View-cache roots that were not reused because the declarative cache key did not match.
    pub view_cache_roots_cache_key_mismatch: u32,
    /// View-cache roots that were not reused because they were not marked as reuse roots.
    ///
    /// This is an authoring-level signal: either view-cache was not enabled, or reuse was gated off
    /// by local state (e.g. `view_cache_needs_rerender` / layout invalidation) upstream.
    pub view_cache_roots_not_marked_reuse_root: u32,
    /// View-cache roots that were not reused because `view_cache_needs_rerender` was set.
    pub view_cache_roots_needs_rerender: u32,
    /// View-cache roots that were not reused because they had a layout invalidation.
    pub view_cache_roots_layout_invalidated: u32,
    /// View-cache roots recorded from retained/manual cache roots (non-declarative).
    pub view_cache_roots_manual: u32,
    /// How many times `set_children_barrier` was applied (structural changes without forcing
    /// ancestor relayout).
    pub set_children_barrier_writes: u32,
    /// How many barrier relayout roots were scheduled via `set_children_barrier` in this frame.
    pub barrier_relayouts_scheduled: u32,
    /// How many barrier relayout roots were actually laid out in this frame.
    pub barrier_relayouts_performed: u32,
    /// How many VirtualList visible-range checks were evaluated (used to request rerenders under
    /// view-cache reuse).
    pub virtual_list_visible_range_checks: u32,
    /// How many VirtualList visible-range checks requested a refresh (range delta outside the
    /// currently mounted span).
    pub virtual_list_visible_range_refreshes: u32,
    /// How many VirtualList window shifts were observed during the current frame.
    pub virtual_list_window_shifts_total: u32,
    /// How many VirtualList window shifts required a non-retained cache-root rerender.
    pub virtual_list_window_shifts_non_retained: u32,
    /// How many retained VirtualList hosts were reconciled (attach/detach without rerendering the
    /// parent view-cache root).
    pub retained_virtual_list_reconciles: u32,
    /// Total items attached across retained VirtualList reconciles (new keys mounted).
    pub retained_virtual_list_attached_items: u32,
    /// Total items detached across retained VirtualList reconciles (keys removed from children).
    pub retained_virtual_list_detached_items: u32,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugHoverDeclarativeInvalidationHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub hit_test: u32,
    pub layout: u32,
    pub paint: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct UiDebugHoverDeclarativeInvalidationCounts {
    pub(super) hit_test: u32,
    pub(super) layout: u32,
    pub(super) paint: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugModelChangeHotspot {
    pub model: ModelId,
    pub observation_edges: u32,
    pub changed: Option<fret_runtime::model::ModelChangedDebugInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugModelChangeUnobserved {
    pub model: ModelId,
    pub created: Option<ModelCreatedDebugInfo>,
    pub changed: Option<fret_runtime::model::ModelChangedDebugInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugGlobalChangeHotspot {
    pub global: TypeId,
    pub observation_edges: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugGlobalChangeUnobserved {
    pub global: TypeId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugInvalidationSource {
    ModelChange,
    GlobalChange,
    Notify,
    Hover,
    Focus,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugInvalidationDetail {
    Unknown,
    ModelObservation,
    GlobalObservation,
    NotifyCall,
    HoverEvent,
    FocusEvent,
    ScrollHandleHitTestOnly,
    ScrollHandleLayout,
    ScrollHandleWindowUpdate,
    ScrollDeferredProbe,
    ScrollHandleScrollToItemWindowUpdate,
    ScrollHandleViewportResizeWindowUpdate,
    ScrollHandleItemsRevisionWindowUpdate,
    ScrollHandlePrefetchWindowUpdate,
    FocusVisiblePolicy,
    InputModalityPolicy,
    AnimationFrameRequest,
}

impl UiDebugInvalidationDetail {
    pub fn from_source(source: UiDebugInvalidationSource) -> Self {
        match source {
            UiDebugInvalidationSource::ModelChange => Self::ModelObservation,
            UiDebugInvalidationSource::GlobalChange => Self::GlobalObservation,
            UiDebugInvalidationSource::Notify => Self::NotifyCall,
            UiDebugInvalidationSource::Hover => Self::HoverEvent,
            UiDebugInvalidationSource::Focus => Self::FocusEvent,
            UiDebugInvalidationSource::Other => Self::Unknown,
        }
    }

    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::ModelObservation => Some("model_observation"),
            Self::GlobalObservation => Some("global_observation"),
            Self::NotifyCall => Some("notify_call"),
            Self::HoverEvent => Some("hover_event"),
            Self::FocusEvent => Some("focus_event"),
            Self::ScrollHandleHitTestOnly => Some("scroll_handle_hit_test_only"),
            Self::ScrollHandleLayout => Some("scroll_handle_layout"),
            Self::ScrollHandleWindowUpdate => Some("scroll_handle_window_update"),
            Self::ScrollDeferredProbe => Some("scroll_deferred_probe"),
            Self::ScrollHandleScrollToItemWindowUpdate => {
                Some("scroll_handle_scroll_to_item_window_update")
            }
            Self::ScrollHandleViewportResizeWindowUpdate => {
                Some("scroll_handle_viewport_resize_window_update")
            }
            Self::ScrollHandleItemsRevisionWindowUpdate => {
                Some("scroll_handle_items_revision_window_update")
            }
            Self::ScrollHandlePrefetchWindowUpdate => Some("scroll_handle_prefetch_window_update"),
            Self::FocusVisiblePolicy => Some("focus_visible_policy"),
            Self::InputModalityPolicy => Some("input_modality_policy"),
            Self::AnimationFrameRequest => Some("animation_frame_request"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugDirtyView {
    pub view: ViewId,
    pub element: Option<GlobalElementId>,
    pub source: UiDebugInvalidationSource,
    pub detail: UiDebugInvalidationDetail,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugNotifyRequest {
    pub frame_id: FrameId,
    pub caller_node: NodeId,
    pub target_view: ViewId,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugInvalidationWalk {
    pub root: NodeId,
    pub root_element: Option<GlobalElementId>,
    pub inv: Invalidation,
    pub source: UiDebugInvalidationSource,
    pub detail: UiDebugInvalidationDetail,
    pub walked_nodes: u32,
    pub truncated_at: Option<NodeId>,
}

/// Controls whether an overlay layer prevents pointer interactions from reaching layers beneath it.
///
/// This is a *mechanism* only. Policy lives in ecosystem crates (e.g. `fret-ui-kit`), which decide
/// when to enable occlusion (Radix `disableOutsidePointerEvents` outcomes, editor interaction
/// arbitration, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerOcclusion {
    /// No occlusion; pointer events route normally via hit-testing across layers.
    #[default]
    None,
    /// Blocks pointer interaction (hover/move/down/up) for layers beneath the occluding layer.
    BlockMouse,
    /// Blocks pointer interaction for layers beneath the occluding layer, but allows scroll wheel
    /// to route to underlay scroll targets.
    BlockMouseExceptScroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiInputArbitrationSnapshot {
    pub modal_barrier_root: Option<NodeId>,
    pub focus_barrier_root: Option<NodeId>,
    pub pointer_occlusion: PointerOcclusion,
    pub pointer_occlusion_layer: Option<UiLayerId>,
    pub pointer_capture_active: bool,
    /// When all captured pointers belong to the same layer, this reports that layer.
    ///
    /// If captures span multiple layers (or a captured node cannot be mapped to a layer), this is
    /// `None` and `pointer_capture_multiple_layers=true`.
    pub pointer_capture_layer: Option<UiLayerId>,
    pub pointer_capture_multiple_layers: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayerInfo {
    pub id: UiLayerId,
    pub root: NodeId,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub pointer_occlusion: PointerOcclusion,
    pub wants_pointer_down_outside_events: bool,
    pub consume_pointer_down_outside_events: bool,
    pub pointer_down_outside_branches: Vec<NodeId>,
    pub wants_pointer_move_events: bool,
    pub wants_timer_events: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugHitTest {
    pub hit: Option<NodeId>,
    pub active_layer_roots: Vec<NodeId>,
    pub barrier_root: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiDebugTextConstraintsSnapshot {
    pub measured: Option<TextConstraints>,
    pub prepared: Option<TextConstraints>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugCacheRootStats {
    pub root: NodeId,
    pub element: Option<GlobalElementId>,
    pub reused: bool,
    pub contained_layout: bool,
    pub paint_replayed_ops: u32,
    pub reuse_reason: UiDebugCacheRootReuseReason,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugScrollHandleChangeKind {
    Layout,
    HitTestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugScrollAxis {
    X,
    Y,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugScrollNodeTelemetry {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub axis: UiDebugScrollAxis,
    pub offset: fret_core::Point,
    pub viewport: fret_core::Size,
    pub content: fret_core::Size,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugScrollbarTelemetry {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub axis: UiDebugScrollAxis,
    pub scroll_target: Option<GlobalElementId>,
    pub offset: fret_core::Point,
    pub viewport: fret_core::Size,
    pub content: fret_core::Size,
    pub track: Rect,
    pub thumb: Option<Rect>,
    pub hovered: bool,
    pub dragging: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugScrollHandleChange {
    pub handle_key: usize,
    pub kind: UiDebugScrollHandleChangeKind,
    pub revision: u64,
    pub prev_revision: Option<u64>,
    pub offset: fret_core::Point,
    pub prev_offset: Option<fret_core::Point>,
    pub viewport: fret_core::Size,
    pub prev_viewport: Option<fret_core::Size>,
    pub content: fret_core::Size,
    pub prev_content: Option<fret_core::Size>,
    pub offset_changed: bool,
    pub viewport_changed: bool,
    pub content_changed: bool,
    pub bound_elements: u32,
    pub bound_nodes_sample: Vec<NodeId>,
    pub upgraded_to_layout_bindings: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugSetChildrenWrite {
    pub parent: NodeId,
    pub frame_id: FrameId,
    pub old_len: u32,
    pub new_len: u32,
    pub old_elements_head: [Option<GlobalElementId>; 4],
    pub new_elements_head: [Option<GlobalElementId>; 4],
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugParentSeverWrite {
    pub child: NodeId,
    pub parent: NodeId,
    pub frame_id: FrameId,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugSetLayerVisibleWrite {
    pub layer: UiLayerId,
    pub frame_id: FrameId,
    pub prev_visible: Option<bool>,
    pub visible: bool,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugOverlayPolicyDecisionWrite {
    pub layer: UiLayerId,
    pub frame_id: FrameId,
    pub kind: &'static str,
    pub present: bool,
    pub interactive: bool,
    pub wants_timer_events: bool,
    pub reason: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugRemoveSubtreeFrameContext {
    pub parent_frame_children_len: Option<u32>,
    pub parent_frame_children_contains_root: Option<bool>,
    pub root_frame_instance_present: bool,
    pub root_frame_children_len: Option<u32>,
    /// Whether this subtree root is reachable from the window's liveness roots when considering
    /// the authoritative retained edges used for GC (ie. the union of `UiTree` and `WindowFrame`
    /// child edges when available).
    pub root_reachable_from_layer_roots: bool,
    pub root_reachable_from_view_cache_roots: Option<bool>,
    pub liveness_layer_roots_len: u32,
    pub view_cache_reuse_roots_len: u32,
    pub view_cache_reuse_root_nodes_len: u32,
    pub trigger_element: Option<GlobalElementId>,
    pub trigger_element_root: Option<GlobalElementId>,
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    pub trigger_element_listed_under_reuse_root: Option<GlobalElementId>,
    pub path_edge_len: u8,
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    pub path_edge_frame_contains_child: [u8; 16],
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugRemoveSubtreeOutcome {
    SkippedLayerRoot,
    RootMissing,
    Removed,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugRemoveSubtreeRecord {
    pub outcome: UiDebugRemoveSubtreeOutcome,
    pub frame_id: FrameId,
    pub root: NodeId,
    pub root_element: Option<GlobalElementId>,
    pub root_parent: Option<NodeId>,
    pub root_parent_element: Option<GlobalElementId>,
    pub root_root: Option<NodeId>,
    pub root_layer: Option<UiLayerId>,
    pub root_layer_visible: Option<bool>,
    pub reachable_from_layer_roots: bool,
    pub reachable_from_view_cache_roots: Option<bool>,
    pub unreachable_from_liveness_roots: bool,
    pub liveness_layer_roots_len: Option<u32>,
    pub view_cache_reuse_roots_len: Option<u32>,
    pub view_cache_reuse_root_nodes_len: Option<u32>,
    pub trigger_element: Option<GlobalElementId>,
    pub trigger_element_root: Option<GlobalElementId>,
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    pub trigger_element_listed_under_reuse_root: Option<GlobalElementId>,
    pub root_children_len: u32,
    pub root_parent_children_len: Option<u32>,
    pub root_parent_children_contains_root: Option<bool>,
    pub root_parent_frame_children_len: Option<u32>,
    pub root_parent_frame_children_contains_root: Option<bool>,
    pub root_frame_instance_present: Option<bool>,
    pub root_frame_children_len: Option<u32>,
    pub root_path_len: u8,
    pub root_path: [u64; 16],
    pub root_path_truncated: bool,
    pub root_path_edge_len: u8,
    /// For each `root_path` edge (`child -> parent`), whether `UiTree` currently has the
    /// corresponding `parent.children` edge:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing node entry)
    pub root_path_edge_ui_contains_child: [u8; 16],
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    pub root_path_edge_frame_contains_child: [u8; 16],
    pub removed_nodes: u32,
    pub removed_head_len: u8,
    pub removed_head: [u64; 16],
    pub removed_tail_len: u8,
    pub removed_tail: [u64; 16],
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Default)]
pub struct UiDebugLayoutEngineMeasureHotspot {
    pub node: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub cache_hits: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub top_children: Vec<UiDebugLayoutEngineMeasureChildHotspot>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiDebugLayoutEngineMeasureChildHotspot {
    pub child: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutEngineSolve {
    pub root: NodeId,
    pub solve_time: Duration,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    pub measure_time: Duration,
    pub top_measures: Vec<UiDebugLayoutEngineMeasureHotspot>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugWidgetMeasureHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugPaintWidgetHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
    pub inclusive_scene_ops_delta: u32,
    pub exclusive_scene_ops_delta: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugPaintTextPrepareHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: &'static str,
    pub text_len: u32,
    pub constraints: TextConstraints,
    pub reasons_mask: u16,
    pub prepare_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DebugLayoutStackFrame {
    pub(super) child_inclusive_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DebugWidgetMeasureStackFrame {
    pub(super) child_inclusive_time: Duration,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DebugPaintStackFrame {
    pub(super) child_inclusive_time: Duration,
    pub(super) child_inclusive_scene_ops_delta: u32,
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
pub(super) struct DebugViewCacheRootRecord {
    pub(super) root: NodeId,
    pub(super) reused: bool,
    pub(super) contained_layout: bool,
    pub(super) reuse_reason: UiDebugCacheRootReuseReason,
}
