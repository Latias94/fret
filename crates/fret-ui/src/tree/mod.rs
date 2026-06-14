use crate::{
    Theme, UiHost, declarative,
    elements::GlobalElementId,
    widget::{
        CommandAvailability, CommandCx, EventCx, Invalidation, LayoutCx, PaintCx,
        PlatformTextInputCx, SemanticsCx, Widget,
    },
};
use fret_core::time::{Duration, Instant};
use fret_core::{
    AppWindowId, Color, Corners, Event, KeyCode, NodeId, Point, PointerEvent, PointerId, Px, Rect,
    Scene, SceneOp, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size,
    TextConstraints, Transform2D, UiServices, ViewId,
};
use fret_runtime::{
    CommandId, Effect, FrameId, InputContext, InputDispatchPhase, KeyChord, KeymapService, ModelId,
    Platform, PlatformCapabilities, TickId,
};
use slotmap::{Key, SecondaryMap, SlotMap};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

mod bounds_tree;
mod commands;
mod debug;
mod dispatch;
mod dispatch_snapshot;
mod frame_arena;
mod hit_test;
mod invalidation_dedup;
mod layers;
mod layout;
mod measure;
mod node_storage;
mod observation;
mod paint;
mod paint_cache;
pub(crate) mod paint_style;
mod prepaint;
pub(crate) use prepaint::{PrepaintAfterLayoutInputs, VirtualListPrepaintWindowOutput};
mod profiling;
mod propagation_depth;
mod semantics;
mod shortcuts;
mod small_list;
mod ui_tree_accessors;
mod ui_tree_debug;
mod ui_tree_default;
mod ui_tree_focus;
mod ui_tree_input_snapshot;
mod ui_tree_invalidation;
mod ui_tree_invalidation_walk;
mod ui_tree_mutation;
mod ui_tree_outside_press;
mod ui_tree_scratch;
mod ui_tree_semantics;
mod ui_tree_subtree_layout_dirty;
mod ui_tree_text_input;
mod ui_tree_view_cache;
mod ui_tree_widget;
mod util;
mod view_boundary;
use debug::{
    DebugLayoutStackFrame, DebugPaintStackFrame, DebugViewCacheRootRecord,
    DebugWidgetMeasureStackFrame, UiDebugHoverDeclarativeInvalidationCounts,
    UiDebugLayoutDirtySource,
};
pub use debug::{
    PointerOcclusion, UiDebugBoundaryStats, UiDebugCacheRootReuseReason, UiDebugCacheRootStats,
    UiDebugCleanGeometrySolveSkipRejection, UiDebugCommandAvailabilityHotspot, UiDebugDirtyView,
    UiDebugFrameStats, UiDebugGlobalChangeHotspot, UiDebugGlobalChangeUnobserved, UiDebugHitTest,
    UiDebugHoverDeclarativeInvalidationHotspot, UiDebugInvalidationDetail,
    UiDebugInvalidationSource, UiDebugInvalidationWalk, UiDebugLayerInfo,
    UiDebugLayoutDirtyDescendant, UiDebugLayoutEngineMeasureChildHotspot,
    UiDebugLayoutEngineMeasureHotspot, UiDebugLayoutEngineSolve, UiDebugLayoutHotspot,
    UiDebugLayoutRequestBuildRoot, UiDebugModelChangeHotspot, UiDebugModelChangeUnobserved,
    UiDebugNotifyRequest, UiDebugPaintTextPrepareHotspot, UiDebugPaintWidgetHotspot,
    UiDebugPrepaintAction, UiDebugPrepaintActionKind, UiDebugRetainedVirtualListReconcile,
    UiDebugRetainedVirtualListReconcileKind, UiDebugScrollAxis, UiDebugScrollHandleChange,
    UiDebugScrollHandleChangeKind, UiDebugScrollLayoutKindProfile, UiDebugScrollLayoutPassKind,
    UiDebugScrollLayoutPhaseProfile, UiDebugScrollLayoutProfile, UiDebugScrollNodeTelemetry,
    UiDebugScrollOverflowObservationTelemetry, UiDebugScrollbarTelemetry,
    UiDebugTextConstraintsSnapshot, UiDebugVirtualListWindow,
    UiDebugVirtualListWindowShiftApplyMode, UiDebugVirtualListWindowShiftKind,
    UiDebugVirtualListWindowShiftReason, UiDebugVirtualListWindowShiftSample,
    UiDebugVirtualListWindowSource, UiDebugWidgetMeasureHotspot, UiInputArbitrationSnapshot,
};
pub(crate) use debug::{
    UiDebugVirtualListWindowShiftClassificationInput, classify_virtual_list_window_shift,
    fallback_virtual_list_window_shift_detail,
};
use frame_arena::FrameArenaScratch;
use invalidation_dedup::{InvalidationDedupTable, InvalidationVisited};
use measure::{DebugMeasureChildRecord, MeasureReentrancyDiagnostics, MeasureStackKey};
use observation::{GlobalObservationIndex, ObservationIndex, ObservationMask};
use profiling::{
    LayoutNodeProfileConfig, LayoutNodeProfileState, MeasureNodeProfileConfig,
    MeasureNodeProfileState, ScrollLayoutKindProfileScope,
};
use propagation_depth::PropagationDepthCacheEntry;
#[cfg(test)]
use util::event_allows_hit_test_path_cache_reuse;
use util::{
    TouchPointerDownOutsideCandidate, event_position, interactive_resize_stable_frames_required,
    pointer_type_supports_hover, rect_aabb_transformed, text_wrap_width_bucket_px,
    text_wrap_width_small_step_bucket_px, text_wrap_width_small_step_max_dw_px,
};

#[cfg(feature = "diagnostics")]
pub use debug::{
    UiDebugDispatchSnapshot, UiDebugDispatchSnapshotNode, UiDebugDispatchSnapshotParityReport,
    UiDebugOverlayPolicyDecisionWrite, UiDebugParentSeverWrite, UiDebugRemoveSubtreeFrameContext,
    UiDebugRemoveSubtreeOutcome, UiDebugRemoveSubtreeRecord, UiDebugSetChildrenWrite,
    UiDebugSetLayerVisibleWrite,
};

use layers::UiLayer;
pub use layers::{OverlayRootOptions, UiLayerId};
use node_storage::{
    ChildrenWritePolicy, HitTestPathCache, Node, NodeMeasureCache, NodeMeasureCacheKey,
    PrepaintHitTestCache, ViewCacheFlags,
};
pub use paint_cache::PaintCachePolicy;
use paint_cache::{PaintCacheEntry, PaintCacheKey, PaintCacheState};
use shortcuts::{
    KeydownShortcutParams, PendingShortcut, PointerDownOutsideOutcome, PointerDownOutsideParams,
};
use small_list::{SmallCopyList, SmallNodeList};
pub use view_boundary::BoundarySceneFragmentDebug;
use view_boundary::{PaintCacheEntryState, ViewBoundaryState};

pub(crate) use dispatch_snapshot::{UiDispatchSnapshot, UiDispatchSnapshotCacheEntry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NodePaintPassthrough {
    pub(crate) paint_children: bool,
    pub(crate) clip: bool,
    pub(crate) clip_corner_radii: Option<Corners>,
    pub(crate) foreground: Option<Color>,
}

fn type_id_sort_key(id: TypeId) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

fn record_layout_invalidation_transition(count: &mut u32, before: bool, after: bool) {
    if before == after {
        return;
    }
    if after {
        *count = count.saturating_add(1);
    } else {
        debug_assert!(*count > 0);
        *count = count.saturating_sub(1);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InvalidationFlags {
    pub layout: bool,
    pub paint: bool,
    pub hit_test: bool,
}

impl InvalidationFlags {
    pub fn mark(&mut self, inv: Invalidation) {
        match inv {
            Invalidation::Paint => self.paint = true,
            Invalidation::Layout => {
                self.layout = true;
                self.paint = true;
            }
            Invalidation::HitTest => {
                self.hit_test = true;
                self.layout = true;
                self.paint = true;
            }
            Invalidation::HitTestOnly => {
                self.hit_test = true;
                self.paint = true;
            }
        }
    }

    pub fn clear(&mut self) {
        self.layout = false;
        self.paint = false;
        self.hit_test = false;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowCommandActionAvailabilitySnapshotSignature {
    pub(crate) window: Option<AppWindowId>,
    pub(crate) base_root: Option<NodeId>,
    pub(crate) active_focus_layers: Vec<NodeId>,
    pub(crate) barrier_root: Option<NodeId>,
    pub(crate) focus: Option<NodeId>,
    pub(crate) pending: WindowRuntimeSnapshotPendingSignature,
    pub(crate) commands: WindowCommandActionAvailabilityCommandSetSignature,
    pub(crate) command_availability_revision: u64,
    /// Cache key for action-availability publishing.
    ///
    /// This intentionally excludes pointer-arbitration state and dispatch-phase noise so
    /// high-frequency pointer-move traffic does not keep invalidating the whole snapshot when
    /// the actual command-gating inputs are unchanged.
    pub(crate) input_ctx: WindowCommandActionAvailabilityInputSignature,
    pub(crate) key_contexts: Vec<Arc<str>>,
    pub(crate) command_registry_revision: u64,
    pub(crate) menu_bar_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowRuntimeSnapshotPendingSignature {
    pub(crate) declarative_roots: Vec<NodeId>,
    pub(crate) post_layout_refine_frame: Option<FrameId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WindowCommandActionAvailabilityCommandSetSignature {
    AllRegisteredWidgetCommands,
    FilteredWidgetCommands(Vec<CommandId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowCommandActionAvailabilityInputSignature {
    pub(crate) platform: Platform,
    pub(crate) caps: PlatformCapabilities,
    pub(crate) ui_has_modal: bool,
    pub(crate) focus_is_text_input: bool,
    pub(crate) text_boundary_mode: fret_runtime::TextBoundaryMode,
    pub(crate) edit_can_undo: bool,
    pub(crate) edit_can_redo: bool,
    pub(crate) router_can_back: bool,
    pub(crate) router_can_forward: bool,
}

impl From<&InputContext> for WindowCommandActionAvailabilityInputSignature {
    fn from(input_ctx: &InputContext) -> Self {
        Self {
            platform: input_ctx.platform,
            caps: input_ctx.caps.clone(),
            ui_has_modal: input_ctx.ui_has_modal,
            focus_is_text_input: input_ctx.focus_is_text_input,
            text_boundary_mode: input_ctx.text_boundary_mode,
            edit_can_undo: input_ctx.edit_can_undo,
            edit_can_redo: input_ctx.edit_can_redo,
            router_can_back: input_ctx.router_can_back,
            router_can_forward: input_ctx.router_can_forward,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowFocusTraversalAvailabilityCacheKey {
    pub(crate) frame_id: FrameId,
    pub(crate) dispatch_snapshot_generation: u64,
    pub(crate) window: Option<AppWindowId>,
    pub(crate) active_layer_roots: Vec<NodeId>,
    pub(crate) barrier_root: Option<NodeId>,
    pub(crate) scope_root: Option<NodeId>,
    pub(crate) resolved_scope_root: Option<NodeId>,
    pub(crate) command_availability_revision: u64,
    pub(crate) layout_ready: bool,
    pub(crate) inspection_active: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct WindowFocusTraversalAvailabilityCacheEntry {
    pub(crate) key: WindowFocusTraversalAvailabilityCacheKey,
    pub(crate) availability: CommandAvailability,
    pub(crate) needs_layout_refine: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tree) struct WindowCommandAvailabilityInterestCacheKey {
    pub(in crate::tree) frame_id: FrameId,
    pub(in crate::tree) command_availability_revision: u64,
    pub(in crate::tree) window: Option<AppWindowId>,
}

#[derive(Debug, Clone)]
pub(in crate::tree) struct WindowCommandAvailabilityInterestCache {
    pub(in crate::tree) key: WindowCommandAvailabilityInterestCacheKey,
    pub(in crate::tree) by_node: HashMap<NodeId, commands::DeclarativeCommandAvailabilityInterest>,
}

/// Retained UI tree and per-window interaction state machine.
///
/// `UiTree` owns the widget/node graph for a single window and is responsible for:
/// - mounting declarative element roots,
/// - routing input events and commands,
/// - running layout and producing paint scenes,
/// - producing semantics snapshots for accessibility backends,
/// - tracking focus/capture/hover and other interaction state across frames.
///
/// Higher-level driver layers (e.g. `fret-bootstrap`) orchestrate when and how a `UiTree` is
/// ticked and provide host services via the [`UiHost`] trait.
pub struct UiTree<H: UiHost> {
    nodes: SlotMap<NodeId, Node<H>>,
    layers: SlotMap<UiLayerId, UiLayer>,
    layer_order: Vec<UiLayerId>,
    root_to_layer: HashMap<NodeId, UiLayerId>,
    base_layer: Option<UiLayerId>,
    focus: Option<NodeId>,
    pending_focus_target: Option<GlobalElementId>,
    captured: HashMap<PointerId, NodeId>,
    active_touch_drag_target: HashMap<PointerId, GlobalElementId>,
    last_pointer_move_hit: HashMap<PointerId, Option<NodeId>>,
    touch_pointer_down_outside_candidates: HashMap<PointerId, TouchPointerDownOutsideCandidate>,
    hit_test_path_cache: Option<HitTestPathCache>,
    hit_test_bounds_trees: bounds_tree::HitTestBoundsTrees,
    last_internal_drag_target: Option<NodeId>,
    window: Option<AppWindowId>,
    ime_allowed: bool,
    ime_composing: bool,
    suppress_text_input_until_key_up: Option<KeyCode>,
    pending_shortcut: PendingShortcut,
    replaying_pending_shortcut: bool,
    alt_menu_bar_arm_key: Option<KeyCode>,
    alt_menu_bar_canceled: bool,
    observed_in_layout: ObservationIndex,
    observed_in_paint: ObservationIndex,
    observed_globals_in_layout: GlobalObservationIndex,
    observed_globals_in_paint: GlobalObservationIndex,
    measure_stack: Vec<MeasureStackKey>,
    measure_cache_this_frame: HashMap<MeasureStackKey, Size>,
    frame_arena: FrameArenaScratch,
    paint_pass: u64,
    scratch_pending_invalidations: HashMap<NodeId, u8>,
    scratch_node_stack: Vec<NodeId>,
    scratch_element_nodes: Vec<(GlobalElementId, NodeId)>,
    scratch_paint_observed_deps_presence: HashSet<GlobalElementId>,
    scratch_paint_observed_deps_presence_active: bool,
    scratch_bounds_records: Vec<(GlobalElementId, Rect)>,
    scratch_element_root_bounds_records: Vec<(GlobalElementId, NodeId, NodeId, Rect)>,
    scratch_prev_element_root_bounds_records: Vec<(GlobalElementId, NodeId, NodeId, Rect)>,
    scratch_visual_bounds_records: Vec<(GlobalElementId, Rect)>,
    measure_reentrancy_diagnostics: MeasureReentrancyDiagnostics,
    layout_engine: crate::layout_engine::TaffyLayoutEngine,
    layout_call_depth: u32,
    layout_performed_nodes: Vec<NodeId>,
    layout_invalidations_count: u32,
    last_layout_frame_id: Option<FrameId>,
    last_layout_bounds: Option<Rect>,
    last_layout_scale_factor: Option<f32>,
    interactive_resize_active: bool,
    interactive_resize_needs_full_rebuild: bool,
    interactive_resize_stable_frames: u8,
    interactive_resize_last_updated_frame: Option<FrameId>,
    interactive_resize_last_bounds_delta: Option<(fret_core::Px, fret_core::Px)>,
    clean_geometry_scroll_side_effect_fallback_nodes: Vec<NodeId>,
    viewport_roots: Vec<(NodeId, Rect)>,
    pending_barrier_relayouts: Vec<NodeId>,
    pending_declarative_window_snapshot_roots: HashSet<NodeId>,
    pending_post_layout_window_runtime_snapshot_refine: bool,
    dispatch_snapshot_generation: u64,
    dispatch_snapshot_cache: Vec<UiDispatchSnapshotCacheEntry>,
    command_availability_revision: u64,
    last_window_command_action_availability_snapshot_signature:
        Option<WindowCommandActionAvailabilitySnapshotSignature>,
    focus_traversal_availability_cache: Option<WindowFocusTraversalAvailabilityCacheEntry>,
    command_availability_interest_cache: Option<WindowCommandAvailabilityInterestCache>,

    #[cfg(debug_assertions)]
    debug_last_declarative_render_root_frame_id: Option<FrameId>,

    debug_enabled: bool,
    debug_stats: UiDebugFrameStats,
    debug_view_cache_roots: Vec<DebugViewCacheRootRecord>,
    debug_view_cache_contained_relayout_roots: Vec<NodeId>,
    debug_paint_cache_replays: HashMap<NodeId, u32>,
    debug_paint_widget_exclusive_started: Option<Instant>,
    debug_layout_request_build_roots: Vec<UiDebugLayoutRequestBuildRoot>,
    debug_layout_engine_solves: Vec<UiDebugLayoutEngineSolve>,
    debug_clean_geometry_solve_skip_rejections:
        HashMap<NodeId, UiDebugCleanGeometrySolveSkipRejection>,
    debug_layout_hotspots: Vec<UiDebugLayoutHotspot>,
    debug_layout_inclusive_hotspots: Vec<UiDebugLayoutHotspot>,
    debug_layout_stack: Vec<DebugLayoutStackFrame>,
    debug_layout_dirty_sources: HashMap<NodeId, UiDebugLayoutDirtySource>,
    debug_widget_measure_hotspots: Vec<UiDebugWidgetMeasureHotspot>,
    debug_widget_measure_stack: Vec<DebugWidgetMeasureStackFrame>,
    debug_paint_widget_hotspots: Vec<UiDebugPaintWidgetHotspot>,
    debug_paint_text_prepare_hotspots: Vec<UiDebugPaintTextPrepareHotspot>,
    debug_paint_stack: Vec<DebugPaintStackFrame>,
    debug_measure_children: HashMap<NodeId, HashMap<NodeId, DebugMeasureChildRecord>>,
    debug_invalidation_walks: Vec<UiDebugInvalidationWalk>,
    debug_model_change_hotspots: Vec<UiDebugModelChangeHotspot>,
    debug_model_change_unobserved: Vec<UiDebugModelChangeUnobserved>,
    debug_global_change_hotspots: Vec<UiDebugGlobalChangeHotspot>,
    debug_global_change_unobserved: Vec<UiDebugGlobalChangeUnobserved>,
    debug_command_availability_hotspots: Vec<UiDebugCommandAvailabilityHotspot>,
    debug_hover_edge_this_frame: bool,
    debug_hover_declarative_invalidations:
        HashMap<NodeId, UiDebugHoverDeclarativeInvalidationCounts>,
    debug_dirty_views: Vec<UiDebugDirtyView>,
    #[cfg(feature = "diagnostics")]
    debug_notify_requests: Vec<UiDebugNotifyRequest>,
    debug_virtual_list_windows: Vec<UiDebugVirtualListWindow>,
    debug_virtual_list_window_shift_samples: Vec<UiDebugVirtualListWindowShiftSample>,
    debug_retained_virtual_list_reconciles: Vec<UiDebugRetainedVirtualListReconcile>,
    debug_scroll_handle_changes: Vec<UiDebugScrollHandleChange>,
    debug_scroll_nodes: Vec<UiDebugScrollNodeTelemetry>,
    debug_scrollbars: Vec<UiDebugScrollbarTelemetry>,
    debug_prepaint_actions: Vec<UiDebugPrepaintAction>,
    #[cfg(feature = "diagnostics")]
    debug_set_children_writes: HashMap<NodeId, UiDebugSetChildrenWrite>,
    #[cfg(feature = "diagnostics")]
    debug_parent_sever_writes: HashMap<NodeId, UiDebugParentSeverWrite>,
    #[cfg(feature = "diagnostics")]
    debug_layer_visible_writes: Vec<UiDebugSetLayerVisibleWrite>,
    #[cfg(feature = "diagnostics")]
    debug_overlay_policy_decisions: Vec<UiDebugOverlayPolicyDecisionWrite>,
    #[cfg(feature = "diagnostics")]
    debug_remove_subtree_frame_context: HashMap<NodeId, UiDebugRemoveSubtreeFrameContext>,
    #[cfg(feature = "diagnostics")]
    debug_removed_subtrees: Vec<UiDebugRemoveSubtreeRecord>,
    #[cfg(feature = "diagnostics")]
    debug_reachable_from_layer_roots: Option<(FrameId, HashSet<NodeId>)>,
    #[cfg(feature = "diagnostics")]
    debug_dispatch_snapshot: Option<UiDispatchSnapshot>,
    #[cfg(feature = "diagnostics")]
    debug_text_constraints_measured: HashMap<NodeId, TextConstraints>,
    #[cfg(feature = "diagnostics")]
    debug_text_constraints_prepared: HashMap<NodeId, TextConstraints>,

    view_cache_enabled: bool,
    paint_cache_policy: PaintCachePolicy,
    inspection_active: bool,
    paint_cache: PaintCacheState,
    interaction_cache: prepaint::InteractionCacheState,
    view_boundaries: slotmap::SecondaryMap<NodeId, ViewBoundaryState>,
    retained_paint_cache_entries: slotmap::SecondaryMap<NodeId, PaintCacheEntryState>,

    dirty_boundaries: HashSet<NodeId>,
    last_redraw_request_tick: Option<TickId>,

    propagation_depth_generation: u32,
    propagation_depth_cache: SecondaryMap<NodeId, PropagationDepthCacheEntry>,
    propagation_chain: Vec<NodeId>,
    propagation_entries: Vec<(u8, u32, u64, NodeId, Invalidation)>,
    invalidation_dedup: InvalidationDedupTable,
    invalidated_layout_nodes: u32,
    invalidated_paint_nodes: u32,
    invalidated_hit_test_nodes: u32,

    semantics: Option<Arc<SemanticsSnapshot>>,
    semantics_dirty: bool,
    semantics_dirty_all: bool,
    semantics_requested: bool,
    layout_node_profile: Option<LayoutNodeProfileState>,
    measure_node_profile: Option<MeasureNodeProfileState>,
    scroll_layout_kind_profile_stack: Vec<ScrollLayoutKindProfileScope>,
    deferred_cleanup: Vec<Box<dyn Widget<H>>>,
}

#[cfg(test)]
thread_local! {
    static COMMAND_AVAILABILITY_INTEREST_PROBE_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    static COMMAND_AVAILABILITY_INTEREST_PROBE_ENABLED: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn reset_command_availability_interest_probe_count() {
    COMMAND_AVAILABILITY_INTEREST_PROBE_COUNT.with(|count| count.set(0));
    COMMAND_AVAILABILITY_INTEREST_PROBE_ENABLED.with(|enabled| enabled.set(true));
}

#[cfg(test)]
pub(crate) fn record_command_availability_interest_probe() {
    COMMAND_AVAILABILITY_INTEREST_PROBE_ENABLED.with(|enabled| {
        if enabled.get() {
            COMMAND_AVAILABILITY_INTEREST_PROBE_COUNT.with(|count| {
                count.set(count.get().saturating_add(1));
            });
        }
    });
}

#[cfg(test)]
pub(crate) fn take_command_availability_interest_probe_count() -> usize {
    COMMAND_AVAILABILITY_INTEREST_PROBE_ENABLED.with(|enabled| enabled.set(false));
    COMMAND_AVAILABILITY_INTEREST_PROBE_COUNT.with(|count| count.get())
}

#[cfg(test)]
mod tests;
