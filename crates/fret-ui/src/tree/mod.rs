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
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
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
mod observation;
mod paint;
mod paint_cache;
mod prepaint;
mod propagation_depth;
mod semantics;
mod shortcuts;
mod ui_tree_default;
mod ui_tree_impl;
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
use propagation_depth::PropagationDepthCacheEntry;

#[cfg(feature = "diagnostics")]
pub use debug::{
    UiDebugOverlayPolicyDecisionWrite, UiDebugParentSeverWrite, UiDebugRemoveSubtreeFrameContext,
    UiDebugRemoveSubtreeOutcome, UiDebugRemoveSubtreeRecord, UiDebugSetChildrenWrite,
    UiDebugSetLayerVisibleWrite,
};

use layers::UiLayer;
pub use layers::UiLayerId;
pub use paint_cache::PaintCachePolicy;
use paint_cache::{PaintCacheEntry, PaintCacheKey, PaintCacheState};
use shortcuts::{
    KeydownShortcutParams, PendingShortcut, PointerDownOutsideOutcome, PointerDownOutsideParams,
};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ViewCacheFlags {
    enabled: bool,
    contained_layout: bool,
    /// Whether the cache root's own box is layout-definite (i.e. it does not size-to-content).
    ///
    /// This is used to decide whether layout/hit-test invalidations can be truncated at the cache
    /// root when view caching is active. Auto-sized cache roots must allow invalidations to reach
    /// ancestors so the root can be placed before running contained relayouts.
    layout_definite: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct NodeMeasureCacheKey {
    known_w_bits: Option<u32>,
    known_h_bits: Option<u32>,
    avail_w: (u8, u32),
    avail_h: (u8, u32),
    scale_bits: u32,
}

#[derive(Debug, Clone, Copy)]
struct NodeMeasureCache {
    key: NodeMeasureCacheKey,
    size: Size,
}

struct Node<H: UiHost> {
    widget: Option<Box<dyn Widget<H>>>,
    element: Option<GlobalElementId>,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    bounds: Rect,
    bounds_written_paint_pass: u64,
    measured_size: Size,
    measure_cache: Option<NodeMeasureCache>,
    invalidation: InvalidationFlags,
    paint_invalidated_by_hit_test_only: bool,
    paint_cache: Option<PaintCacheEntry>,
    interaction_cache: Option<prepaint::InteractionCacheEntry>,
    prepaint_outputs: PrepaintOutputs,
    prepaint_hit_test: Option<PrepaintHitTestCache>,
    view_cache: ViewCacheFlags,
    view_cache_needs_rerender: bool,
    text_boundary_mode_override: Option<fret_runtime::TextBoundaryMode>,
}

#[derive(Debug, Clone)]
struct HitTestPathCache {
    layer_root: NodeId,
    path: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy)]
struct PrepaintHitTestCache {
    render_transform_inv: Option<Transform2D>,
    children_render_transform_inv: Option<Transform2D>,
    clips_hit_test: bool,
    clip_hit_test_corner_radii: Option<Corners>,
    is_focusable: bool,
    focus_traversal_children: bool,
    can_scroll_descendant_into_view: bool,
}

#[derive(Default)]
struct PrepaintOutputs {
    key: Option<PaintCacheKey>,
    values: Vec<(TypeId, Box<dyn Any>)>,
}

impl PrepaintOutputs {
    fn begin_frame(&mut self, key: PaintCacheKey) {
        if self.key != Some(key) {
            self.key = Some(key);
            self.values.clear();
        }
    }

    fn set<T: Any>(&mut self, value: T) {
        let ty = TypeId::of::<T>();
        if let Some((_, existing)) = self.values.iter_mut().find(|(id, _)| *id == ty) {
            *existing = Box::new(value);
            return;
        }
        self.values.push((ty, Box::new(value)));
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let ty = TypeId::of::<T>();
        self.values
            .iter()
            .find(|(id, _)| *id == ty)
            .and_then(|(_, value)| value.downcast_ref::<T>())
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let ty = TypeId::of::<T>();
        self.values
            .iter_mut()
            .find(|(id, _)| *id == ty)
            .and_then(|(_, value)| value.downcast_mut::<T>())
    }
}

impl<H: UiHost> Node<H> {
    fn new(widget: impl Widget<H> + 'static) -> Self {
        Self {
            widget: Some(Box::new(widget)),
            element: None,
            parent: None,
            children: Vec::new(),
            bounds: Rect::default(),
            bounds_written_paint_pass: 0,
            measured_size: Size::default(),
            measure_cache: None,
            invalidation: InvalidationFlags {
                layout: true,
                paint: true,
                hit_test: true,
            },
            paint_invalidated_by_hit_test_only: false,
            paint_cache: None,
            interaction_cache: None,
            prepaint_outputs: PrepaintOutputs::default(),
            prepaint_hit_test: None,
            view_cache: ViewCacheFlags::default(),
            view_cache_needs_rerender: false,
            text_boundary_mode_override: None,
        }
    }

    #[cfg(test)]
    fn new_for_element(element: GlobalElementId, widget: impl Widget<H> + 'static) -> Self {
        Self {
            element: Some(element),
            ..Self::new(widget)
        }
    }
}

/// # Safety
///
/// The caller must guarantee that every element in `slice` is initialized.
#[inline]
unsafe fn assume_init_slice_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: `MaybeUninit<T>` has the same layout as `T`, and the caller guarantees initialization.
    //
    // Note: our pinned toolchain does not expose a standard-library helper for assuming init on
    // `&[MaybeUninit<T>]`, so we use the conventional `from_raw_parts` cast.
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<T>(), slice.len()) }
}

#[derive(Debug)]
pub(super) struct SmallNodeList<const N: usize> {
    len: usize,
    inline: [MaybeUninit<NodeId>; N],
    spill: Vec<NodeId>,
}

impl<const N: usize> Default for SmallNodeList<N> {
    fn default() -> Self {
        Self {
            len: 0,
            inline: [MaybeUninit::uninit(); N],
            spill: Vec::new(),
        }
    }
}

impl<const N: usize> SmallNodeList<N> {
    pub(super) fn set(&mut self, nodes: &[NodeId]) {
        if nodes.len() <= N {
            self.spill.clear();
            self.len = nodes.len();
            for (i, &id) in nodes.iter().enumerate() {
                self.inline[i].write(id);
            }
        } else {
            self.len = 0;
            self.spill.clear();
            self.spill.extend_from_slice(nodes);
        }
    }

    pub(super) fn as_slice(&self) -> &[NodeId] {
        if !self.spill.is_empty() {
            return self.spill.as_slice();
        }
        debug_assert!(self.len <= N);
        // SAFETY: when `spill` is empty, indices `0..len` are initialized via `set()`.
        unsafe { assume_init_slice_ref(&self.inline[..self.len]) }
    }
}

#[derive(Debug)]
pub(super) struct SmallCopyList<T: Copy, const N: usize> {
    len: usize,
    inline: [MaybeUninit<T>; N],
    spill: Vec<T>,
}

impl<T: Copy, const N: usize> Default for SmallCopyList<T, N> {
    fn default() -> Self {
        Self {
            len: 0,
            inline: [MaybeUninit::uninit(); N],
            spill: Vec::new(),
        }
    }
}

impl<T: Copy, const N: usize> SmallCopyList<T, N> {
    pub(super) fn push(&mut self, value: T) {
        if self.spill.is_empty() && self.len < N {
            self.inline[self.len].write(value);
            self.len += 1;
            debug_assert!(self.len <= N);
            return;
        }

        if self.spill.is_empty() {
            debug_assert!(self.len <= N);
            self.spill.reserve(self.len.saturating_add(1));
            // SAFETY: indices `0..len` are initialized while `spill` is empty.
            let inline = unsafe { assume_init_slice_ref(&self.inline[..self.len]) };
            self.spill.extend_from_slice(inline);
            self.len = 0;
        }

        self.spill.push(value);
    }

    pub(super) fn as_slice(&self) -> &[T] {
        if !self.spill.is_empty() {
            return self.spill.as_slice();
        }
        debug_assert!(self.len <= N);
        // SAFETY: indices `0..len` are initialized until we spill.
        unsafe { assume_init_slice_ref(&self.inline[..self.len]) }
    }
}

#[cfg(test)]
mod small_list_tests {
    use super::*;
    use slotmap::KeyData;

    fn node(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    #[test]
    fn small_node_list_uses_inline_storage_for_small_slices() {
        let mut list: SmallNodeList<4> = SmallNodeList::default();
        let nodes = [node(1), node(2), node(3)];
        list.set(&nodes);

        assert!(list.spill.is_empty());
        assert_eq!(list.len, nodes.len());
        assert_eq!(list.as_slice(), nodes.as_slice());
    }

    #[test]
    fn small_node_list_spills_for_large_slices_and_can_return_to_inline() {
        let mut list: SmallNodeList<2> = SmallNodeList::default();

        let spilled = [node(10), node(11), node(12)];
        list.set(&spilled);
        assert_eq!(list.len, 0);
        assert_eq!(list.spill.as_slice(), spilled.as_slice());
        assert_eq!(list.as_slice(), spilled.as_slice());

        let inline = [node(20)];
        list.set(&inline);
        assert!(list.spill.is_empty());
        assert_eq!(list.len, inline.len());
        assert_eq!(list.as_slice(), inline.as_slice());
    }

    #[test]
    fn small_copy_list_stays_inline_until_full_and_then_spills_in_order() {
        let mut list: SmallCopyList<u32, 3> = SmallCopyList::default();

        list.push(1);
        list.push(2);
        list.push(3);
        assert!(list.spill.is_empty());
        assert_eq!(list.len, 3);
        assert_eq!(list.as_slice(), &[1, 2, 3]);

        list.push(4);
        assert_eq!(list.len, 0);
        assert_eq!(list.spill.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(list.as_slice(), &[1, 2, 3, 4]);

        list.push(5);
        assert_eq!(list.spill.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(list.as_slice(), &[1, 2, 3, 4, 5]);
    }
}

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

#[derive(Debug, Clone, Copy)]
struct LayoutNodeProfileEntry {
    node: NodeId,
    pass_kind: crate::layout_pass::LayoutPassKind,
    bounds: Rect,
    elapsed_total: Duration,
    elapsed_self: Duration,
}

#[derive(Debug, Clone, Copy)]
struct LayoutNodeProfileConfig {
    top_n: usize,
    min_elapsed: Duration,
}

impl LayoutNodeProfileConfig {
    fn from_env() -> Option<Self> {
        let cfg = crate::runtime_config::ui_runtime_config().layout_node_profile?;
        Some(Self {
            top_n: cfg.top_n,
            min_elapsed: cfg.min_elapsed,
        })
    }
}

#[derive(Debug)]
struct LayoutNodeProfileState {
    config: LayoutNodeProfileConfig,
    frame_id: FrameId,
    entries: Vec<LayoutNodeProfileEntry>,
    stack: Vec<LayoutNodeProfileStackEntry>,
    total_self_time: Duration,
    nodes_profiled: u64,
}

impl LayoutNodeProfileState {
    fn new(config: LayoutNodeProfileConfig, frame_id: FrameId) -> Self {
        Self {
            config,
            frame_id,
            entries: Vec::new(),
            stack: Vec::new(),
            total_self_time: Duration::default(),
            nodes_profiled: 0,
        }
    }

    fn enter(&mut self, node: NodeId, pass_kind: crate::layout_pass::LayoutPassKind, bounds: Rect) {
        self.stack.push(LayoutNodeProfileStackEntry {
            node,
            pass_kind,
            bounds,
            started: fret_core::time::Instant::now(),
            child_time: Duration::default(),
        });
    }

    fn exit(&mut self, node: NodeId) {
        let Some(entry) = self.stack.pop() else {
            return;
        };
        if entry.node != node {
            // Best-effort: avoid poisoning the layout pass if the stack gets out of sync.
            self.stack.clear();
            return;
        }

        let elapsed_total = entry.started.elapsed();
        let elapsed_self = elapsed_total.saturating_sub(entry.child_time);
        self.total_self_time = self.total_self_time.saturating_add(elapsed_self);
        self.nodes_profiled = self.nodes_profiled.saturating_add(1);

        if let Some(parent) = self.stack.last_mut() {
            parent.child_time = parent.child_time.saturating_add(elapsed_total);
        }

        self.record(LayoutNodeProfileEntry {
            node: entry.node,
            pass_kind: entry.pass_kind,
            bounds: entry.bounds,
            elapsed_total,
            elapsed_self,
        });
    }

    fn record(&mut self, entry: LayoutNodeProfileEntry) {
        if entry.elapsed_self < self.config.min_elapsed {
            return;
        }

        // Keep a stable, small "top N" list; N is tiny (default 16), so O(N) insertion is fine.
        let mut inserted = false;
        for i in 0..self.entries.len() {
            if entry.elapsed_self > self.entries[i].elapsed_self {
                self.entries.insert(i, entry);
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.entries.push(entry);
        }
        if self.entries.len() > self.config.top_n {
            self.entries.truncate(self.config.top_n);
        }
    }
}

#[derive(Debug)]
struct LayoutNodeProfileStackEntry {
    node: NodeId,
    pass_kind: crate::layout_pass::LayoutPassKind,
    bounds: Rect,
    started: fret_core::time::Instant,
    child_time: Duration,
}

#[derive(Debug, Clone, Copy)]
struct MeasureNodeProfileEntry {
    node: NodeId,
    constraints: crate::layout_constraints::LayoutConstraints,
    elapsed_total: Duration,
    elapsed_self: Duration,
}

#[derive(Debug, Clone, Copy)]
struct MeasureNodeProfileConfig {
    top_n: usize,
    min_elapsed: Duration,
}

impl MeasureNodeProfileConfig {
    fn from_env() -> Option<Self> {
        let cfg = crate::runtime_config::ui_runtime_config().measure_node_profile?;
        Some(Self {
            top_n: cfg.top_n,
            min_elapsed: cfg.min_elapsed,
        })
    }
}

#[derive(Debug)]
struct MeasureNodeProfileState {
    config: MeasureNodeProfileConfig,
    frame_id: FrameId,
    entries: Vec<MeasureNodeProfileEntry>,
    stack: Vec<MeasureNodeProfileStackEntry>,
    total_self_time: Duration,
    nodes_profiled: u64,
}

impl MeasureNodeProfileState {
    fn new(config: MeasureNodeProfileConfig, frame_id: FrameId) -> Self {
        Self {
            config,
            frame_id,
            entries: Vec::new(),
            stack: Vec::new(),
            total_self_time: Duration::default(),
            nodes_profiled: 0,
        }
    }

    fn enter(&mut self, node: NodeId, constraints: crate::layout_constraints::LayoutConstraints) {
        self.stack.push(MeasureNodeProfileStackEntry {
            node,
            constraints,
            started: fret_core::time::Instant::now(),
            child_time: Duration::default(),
        });
    }

    fn exit(&mut self, node: NodeId) {
        let Some(entry) = self.stack.pop() else {
            return;
        };
        if entry.node != node {
            self.stack.clear();
            return;
        }

        let elapsed_total = entry.started.elapsed();
        let elapsed_self = elapsed_total.saturating_sub(entry.child_time);
        self.total_self_time = self.total_self_time.saturating_add(elapsed_self);
        self.nodes_profiled = self.nodes_profiled.saturating_add(1);

        if let Some(parent) = self.stack.last_mut() {
            parent.child_time = parent.child_time.saturating_add(elapsed_total);
        }

        self.record(MeasureNodeProfileEntry {
            node: entry.node,
            constraints: entry.constraints,
            elapsed_total,
            elapsed_self,
        });
    }

    fn record(&mut self, entry: MeasureNodeProfileEntry) {
        if entry.elapsed_total < self.config.min_elapsed {
            return;
        }

        let mut inserted = false;
        for i in 0..self.entries.len() {
            if entry.elapsed_total > self.entries[i].elapsed_total {
                self.entries.insert(i, entry);
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.entries.push(entry);
        }
        if self.entries.len() > self.config.top_n {
            self.entries.truncate(self.config.top_n);
        }
    }
}

#[derive(Debug)]
struct MeasureNodeProfileStackEntry {
    node: NodeId,
    constraints: crate::layout_constraints::LayoutConstraints,
    started: fret_core::time::Instant,
    child_time: Duration,
}

#[derive(Clone)]
struct TouchPointerDownOutsideCandidate {
    layer_id: UiLayerId,
    root: NodeId,
    consume: bool,
    down_event: Event,
    start_pos: Point,
    moved: bool,
}

fn pointer_position(pe: &PointerEvent) -> Point {
    match pe {
        PointerEvent::Move { position, .. }
        | PointerEvent::Down { position, .. }
        | PointerEvent::Up { position, .. }
        | PointerEvent::Wheel { position, .. }
        | PointerEvent::PinchGesture { position, .. } => *position,
    }
}

fn rect_aabb_transformed(rect: Rect, t: Transform2D) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p00 = t.apply_point(Point::new(Px(x0), Px(y0)));
    let p10 = t.apply_point(Point::new(Px(x1), Px(y0)));
    let p01 = t.apply_point(Point::new(Px(x0), Px(y1)));
    let p11 = t.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
    let max_x = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
    let min_y = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
    let max_y = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

fn event_position(event: &Event) -> Option<Point> {
    match event {
        Event::Pointer(pe) => Some(pointer_position(pe)),
        Event::PointerCancel(e) => e.position,
        Event::ExternalDrag(e) => Some(e.position),
        Event::InternalDrag(e) => Some(e.position),
        _ => None,
    }
}

#[cfg(test)]
fn event_allows_hit_test_path_cache_reuse(event: &Event) -> bool {
    matches!(
        event,
        Event::Pointer(PointerEvent::Move { .. })
            | Event::Pointer(PointerEvent::Wheel { .. })
            | Event::Pointer(PointerEvent::PinchGesture { .. })
            | Event::ExternalDrag(_)
            | Event::InternalDrag(_)
    )
}

fn pointer_type_supports_hover(pointer_type: fret_core::PointerType) -> bool {
    // Hover is a cursor-driven affordance (Mouse/Pen). Touch pointers must not perturb hover state,
    // otherwise multi-pointer input can cause spurious hover exits while a mouse cursor remains in
    // place.
    //
    // `Unknown` is treated as hover-capable to keep desktop backends usable when pointer
    // classification is incomplete.
    matches!(
        pointer_type,
        fret_core::PointerType::Mouse
            | fret_core::PointerType::Pen
            | fret_core::PointerType::Unknown
    )
}

fn interactive_resize_stable_frames_required() -> u8 {
    crate::runtime_config::ui_runtime_config().interactive_resize_stable_frames_required
}

fn text_wrap_width_bucket_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_bucket_px
}

fn text_wrap_width_small_step_bucket_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_small_step_bucket_px
}

fn text_wrap_width_small_step_max_dw_px() -> u8 {
    crate::runtime_config::ui_runtime_config().text_wrap_width_small_step_max_dw_px
}

#[cfg(test)]
mod tests;
