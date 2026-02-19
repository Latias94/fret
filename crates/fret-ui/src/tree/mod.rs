use crate::{
    Theme, UiHost, declarative,
    elements::GlobalElementId,
    widget::{
        CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, PlatformTextInputCx, SemanticsCx,
        Widget,
    },
};
use fret_core::time::{Duration, Instant};
use fret_core::{
    AppWindowId, Corners, Event, KeyCode, NodeId, Point, PointerEvent, PointerId, Px, Rect, Scene,
    SceneOp, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size, TextConstraints,
    Transform2D, UiServices, ViewId,
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
mod prepaint;
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
mod ui_tree_text_input;
mod ui_tree_view_cache;
mod ui_tree_widget;
mod util;
use debug::{
    DebugLayoutStackFrame, DebugPaintStackFrame, DebugViewCacheRootRecord,
    DebugWidgetMeasureStackFrame, UiDebugHoverDeclarativeInvalidationCounts,
};
pub use debug::{
    PointerOcclusion, UiDebugCacheRootReuseReason, UiDebugCacheRootStats, UiDebugDirtyView,
    UiDebugFrameStats, UiDebugGlobalChangeHotspot, UiDebugGlobalChangeUnobserved, UiDebugHitTest,
    UiDebugHoverDeclarativeInvalidationHotspot, UiDebugInvalidationDetail,
    UiDebugInvalidationSource, UiDebugInvalidationWalk, UiDebugLayerInfo,
    UiDebugLayoutEngineMeasureChildHotspot, UiDebugLayoutEngineMeasureHotspot,
    UiDebugLayoutEngineSolve, UiDebugLayoutHotspot, UiDebugModelChangeHotspot,
    UiDebugModelChangeUnobserved, UiDebugNotifyRequest, UiDebugPaintTextPrepareHotspot,
    UiDebugPaintWidgetHotspot, UiDebugPrepaintAction, UiDebugPrepaintActionKind,
    UiDebugRetainedVirtualListReconcile, UiDebugRetainedVirtualListReconcileKind,
    UiDebugScrollAxis, UiDebugScrollHandleChange, UiDebugScrollHandleChangeKind,
    UiDebugScrollNodeTelemetry, UiDebugScrollbarTelemetry, UiDebugTextConstraintsSnapshot,
    UiDebugVirtualListWindow, UiDebugVirtualListWindowShiftApplyMode,
    UiDebugVirtualListWindowShiftKind, UiDebugVirtualListWindowShiftReason,
    UiDebugVirtualListWindowShiftSample, UiDebugVirtualListWindowSource,
    UiDebugWidgetMeasureHotspot, UiInputArbitrationSnapshot,
};
use frame_arena::FrameArenaScratch;
use invalidation_dedup::{InvalidationDedupTable, InvalidationVisited};
use measure::{DebugMeasureChildRecord, MeasureReentrancyDiagnostics, MeasureStackKey};
use observation::{GlobalObservationIndex, ObservationIndex, ObservationMask};
use profiling::{
    LayoutNodeProfileConfig, LayoutNodeProfileState, MeasureNodeProfileConfig,
    MeasureNodeProfileState,
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
    UiDebugOverlayPolicyDecisionWrite, UiDebugParentSeverWrite, UiDebugRemoveSubtreeFrameContext,
    UiDebugRemoveSubtreeOutcome, UiDebugRemoveSubtreeRecord, UiDebugSetChildrenWrite,
    UiDebugSetLayerVisibleWrite,
};

use layers::UiLayer;
pub use layers::UiLayerId;
use node_storage::{
    HitTestPathCache, Node, NodeMeasureCache, NodeMeasureCacheKey, PrepaintHitTestCache,
    ViewCacheFlags,
};
pub use paint_cache::PaintCachePolicy;
use paint_cache::{PaintCacheEntry, PaintCacheKey, PaintCacheState};
use shortcuts::{
    KeydownShortcutParams, PendingShortcut, PointerDownOutsideOutcome, PointerDownOutsideParams,
};
use small_list::{SmallCopyList, SmallNodeList};

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
    captured: HashMap<PointerId, NodeId>,
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
    scratch_visual_bounds_records: Vec<(GlobalElementId, Rect)>,
    measure_reentrancy_diagnostics: MeasureReentrancyDiagnostics,
    layout_engine: crate::layout_engine::TaffyLayoutEngine,
    layout_invalidations_count: u32,
    last_layout_bounds: Option<Rect>,
    last_layout_scale_factor: Option<f32>,
    interactive_resize_active: bool,
    interactive_resize_stable_frames: u8,
    interactive_resize_last_updated_frame: Option<FrameId>,
    interactive_resize_last_bounds_delta: Option<(fret_core::Px, fret_core::Px)>,
    viewport_roots: Vec<(NodeId, Rect)>,
    pending_barrier_relayouts: Vec<NodeId>,

    debug_enabled: bool,
    debug_stats: UiDebugFrameStats,
    debug_view_cache_roots: Vec<DebugViewCacheRootRecord>,
    debug_view_cache_contained_relayout_roots: Vec<NodeId>,
    debug_paint_cache_replays: HashMap<NodeId, u32>,
    debug_paint_widget_exclusive_started: Option<Instant>,
    debug_layout_engine_solves: Vec<UiDebugLayoutEngineSolve>,
    debug_layout_hotspots: Vec<UiDebugLayoutHotspot>,
    debug_layout_inclusive_hotspots: Vec<UiDebugLayoutHotspot>,
    debug_layout_stack: Vec<DebugLayoutStackFrame>,
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
    debug_text_constraints_measured: HashMap<NodeId, TextConstraints>,
    #[cfg(feature = "diagnostics")]
    debug_text_constraints_prepared: HashMap<NodeId, TextConstraints>,

    view_cache_enabled: bool,
    paint_cache_policy: PaintCachePolicy,
    inspection_active: bool,
    paint_cache: PaintCacheState,
    interaction_cache: prepaint::InteractionCacheState,

    dirty_cache_roots: HashSet<NodeId>,
    dirty_cache_root_reasons:
        HashMap<NodeId, (UiDebugInvalidationSource, UiDebugInvalidationDetail)>,
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
    semantics_requested: bool,
    layout_node_profile: Option<LayoutNodeProfileState>,
    measure_node_profile: Option<MeasureNodeProfileState>,
    deferred_cleanup: Vec<Box<dyn Widget<H>>>,
}

#[cfg(test)]
mod tests;
