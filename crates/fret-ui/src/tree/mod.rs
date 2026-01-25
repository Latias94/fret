use crate::{
    Theme, UiHost, declarative,
    elements::GlobalElementId,
    widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::time::{Duration, Instant};
use fret_core::{
    AppWindowId, Corners, Event, KeyCode, NodeId, Point, PointerEvent, PointerId, Px, Rect, Scene,
    SceneOp, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size, Transform2D,
    UiServices, ViewId,
};
use fret_runtime::{
    CommandId, Effect, FrameId, InputContext, InputDispatchPhase, KeyChord, KeymapService,
    ModelCreatedDebugInfo, ModelId, Platform, PlatformCapabilities, TickId,
};
use slotmap::{Key, SlotMap};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
use std::slice;
use std::sync::Arc;

mod commands;
mod dispatch;
mod hit_test;
mod layers;
mod layout;
mod paint;
mod paint_cache;
mod prepaint;
mod semantics;
mod shortcuts;

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

struct Node<H: UiHost> {
    widget: Option<Box<dyn Widget<H>>>,
    element: Option<GlobalElementId>,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    bounds: Rect,
    measured_size: Size,
    invalidation: InvalidationFlags,
    paint_cache: Option<PaintCacheEntry>,
    interaction_cache: Option<prepaint::InteractionCacheEntry>,
    prepaint_hit_test: Option<PrepaintHitTestCache>,
    view_cache: ViewCacheFlags,
    view_cache_needs_rerender: bool,
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
}

impl<H: UiHost> Node<H> {
    fn new(widget: impl Widget<H> + 'static) -> Self {
        Self {
            widget: Some(Box::new(widget)),
            element: None,
            parent: None,
            children: Vec::new(),
            bounds: Rect::default(),
            measured_size: Size::default(),
            invalidation: InvalidationFlags {
                layout: true,
                paint: true,
                hit_test: true,
            },
            paint_cache: None,
            interaction_cache: None,
            prepaint_hit_test: None,
            view_cache: ViewCacheFlags::default(),
            view_cache_needs_rerender: false,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugFrameStats {
    pub frame_id: FrameId,
    pub layout_time: Duration,
    pub prepaint_time: Duration,
    pub paint_time: Duration,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    pub interaction_cache_hits: u32,
    pub interaction_cache_misses: u32,
    pub interaction_cache_replayed_records: u32,
    pub interaction_records: u32,
    /// Number of layout engine root solves performed during the current frame.
    pub layout_engine_solves: u64,
    /// Total time spent in layout engine solves during the current frame.
    pub layout_engine_solve_time: Duration,
    /// Number of "widget-local" layout engine solves triggered as a fallback when a widget cannot
    /// consume already-solved engine child rects.
    ///
    /// The goal for v2 is to keep this at `0` for normal UI trees by ensuring explicit layout
    /// barriers (scroll/virtualization/splits/...) register viewport roots or explicitly solve
    /// their child roots.
    pub layout_engine_widget_fallback_solves: u64,
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
struct UiDebugHoverDeclarativeInvalidationCounts {
    hit_test: u32,
    layout: u32,
    paint: u32,
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
    ScrollHandle,
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
            Self::ScrollHandle => Some("scroll_handle"),
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

#[derive(Debug, Clone, Copy)]
pub struct UiDebugLayerInfo {
    pub id: UiLayerId,
    pub root: NodeId,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub pointer_occlusion: PointerOcclusion,
    pub wants_pointer_down_outside_events: bool,
    pub wants_pointer_move_events: bool,
    pub wants_timer_events: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugHitTest {
    pub hit: Option<NodeId>,
    pub active_layer_roots: Vec<NodeId>,
    pub barrier_root: Option<NodeId>,
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

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugSetChildrenWrite {
    pub parent: NodeId,
    pub frame_id: FrameId,
    pub old_len: u32,
    pub new_len: u32,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
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
    pub root_children_len: u32,
    pub root_parent_children_len: Option<u32>,
    pub root_path_len: u8,
    pub root_path: [u64; 16],
    pub root_path_truncated: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugCacheRootReuseReason {
    FirstMount,
    NodeRecreated,
    MarkedReuseRoot,
    NotMarkedReuseRoot,
    CacheKeyMismatch,
}

impl UiDebugCacheRootReuseReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstMount => "first_mount",
            Self::NodeRecreated => "node_recreated",
            Self::MarkedReuseRoot => "marked_reuse_root",
            Self::NotMarkedReuseRoot => "not_marked_reuse_root",
            Self::CacheKeyMismatch => "cache_key_mismatch",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DebugViewCacheRootRecord {
    root: NodeId,
    reused: bool,
    contained_layout: bool,
    reuse_reason: UiDebugCacheRootReuseReason,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ObservationMask {
    paint: bool,
    layout: bool,
    hit_test: bool,
}

impl ObservationMask {
    fn add(&mut self, inv: Invalidation) {
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

    fn union(self, other: Self) -> Self {
        Self {
            paint: self.paint || other.paint,
            layout: self.layout || other.layout,
            hit_test: self.hit_test || other.hit_test,
        }
    }

    fn is_empty(self) -> bool {
        !(self.paint || self.layout || self.hit_test)
    }
}

#[derive(Default)]
struct ObservationIndex {
    by_node: HashMap<NodeId, Vec<(ModelId, ObservationMask)>>,
    by_model: HashMap<ModelId, HashMap<NodeId, ObservationMask>>,
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
        unsafe { slice::from_raw_parts(self.inline.as_ptr() as *const NodeId, self.len) }
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
            return;
        }

        if self.spill.is_empty() {
            self.spill.reserve(self.len.saturating_add(1));
            for i in 0..self.len {
                // SAFETY: indices 0..len have been written.
                let v = unsafe { self.inline[i].assume_init() };
                self.spill.push(v);
            }
            self.len = 0;
        }

        self.spill.push(value);
    }

    pub(super) fn as_slice(&self) -> &[T] {
        if !self.spill.is_empty() {
            return self.spill.as_slice();
        }
        unsafe { slice::from_raw_parts(self.inline.as_ptr() as *const T, self.len) }
    }
}

impl ObservationIndex {
    fn record(&mut self, node: NodeId, observations: &[(ModelId, Invalidation)]) {
        let entry = self.by_node.entry(node).or_default();

        let mut prev_models = SmallCopyList::<ModelId, 8>::default();
        for (model, _) in entry.iter() {
            prev_models.push(*model);
        }

        entry.clear();
        entry.reserve(observations.len());
        for &(model, inv) in observations {
            if let Some((_, mask)) = entry.iter_mut().find(|(m, _)| *m == model) {
                mask.add(inv);
            } else {
                let mut mask = ObservationMask::default();
                mask.add(inv);
                entry.push((model, mask));
            }
        }

        for model in prev_models.as_slice() {
            if entry.iter().any(|(m, _)| *m == *model) {
                continue;
            }
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
                }
            }
        }

        for (model, mask) in entry.iter().copied() {
            self.by_model.entry(model).or_default().insert(node, mask);
        }
    }

    fn remove_node(&mut self, node: NodeId) {
        let Some(prev) = self.by_node.remove(&node) else {
            return;
        };
        for (model, _) in &prev {
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
                }
            }
        }
    }
}

#[derive(Default)]
struct GlobalObservationIndex {
    by_node: HashMap<NodeId, Vec<(TypeId, ObservationMask)>>,
    by_global: HashMap<TypeId, HashMap<NodeId, ObservationMask>>,
}

impl GlobalObservationIndex {
    fn record(&mut self, node: NodeId, observations: &[(TypeId, Invalidation)]) {
        let entry = self.by_node.entry(node).or_default();

        let mut prev_globals = SmallCopyList::<TypeId, 8>::default();
        for (global, _) in entry.iter() {
            prev_globals.push(*global);
        }

        entry.clear();
        entry.reserve(observations.len());
        for &(global, inv) in observations {
            if let Some((_, mask)) = entry.iter_mut().find(|(g, _)| *g == global) {
                mask.add(inv);
            } else {
                let mut mask = ObservationMask::default();
                mask.add(inv);
                entry.push((global, mask));
            }
        }

        for global in prev_globals.as_slice() {
            if entry.iter().any(|(g, _)| *g == *global) {
                continue;
            }
            if let Some(nodes) = self.by_global.get_mut(global) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_global.remove(global);
                }
            }
        }

        for (global, mask) in entry.iter().copied() {
            self.by_global.entry(global).or_default().insert(node, mask);
        }
    }

    fn remove_node(&mut self, node: NodeId) {
        let Some(prev) = self.by_node.remove(&node) else {
            return;
        };
        for (global, _) in &prev {
            if let Some(nodes) = self.by_global.get_mut(global) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_global.remove(global);
                }
            }
        }
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
    last_internal_drag_target: Option<NodeId>,
    window: Option<AppWindowId>,
    ime_allowed: bool,
    ime_composing: bool,
    suppress_text_input_until_key_up: Option<KeyCode>,
    pending_shortcut: PendingShortcut,
    replaying_pending_shortcut: bool,
    observed_in_layout: ObservationIndex,
    observed_in_paint: ObservationIndex,
    observed_globals_in_layout: GlobalObservationIndex,
    observed_globals_in_paint: GlobalObservationIndex,
    measure_stack: Vec<MeasureStackKey>,
    measure_reentrancy_diagnostics: MeasureReentrancyDiagnostics,
    layout_engine: crate::layout_engine::TaffyLayoutEngine,
    viewport_roots: Vec<(NodeId, Rect)>,
    pending_barrier_relayouts: Vec<NodeId>,

    debug_enabled: bool,
    debug_stats: UiDebugFrameStats,
    debug_view_cache_roots: Vec<DebugViewCacheRootRecord>,
    debug_view_cache_contained_relayout_roots: Vec<NodeId>,
    debug_paint_cache_replays: HashMap<NodeId, u32>,
    debug_layout_engine_solves: Vec<UiDebugLayoutEngineSolve>,
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
    debug_set_children_writes: HashMap<NodeId, UiDebugSetChildrenWrite>,
    #[cfg(feature = "diagnostics")]
    debug_removed_subtrees: Vec<UiDebugRemoveSubtreeRecord>,

    view_cache_enabled: bool,
    paint_cache_policy: PaintCachePolicy,
    inspection_active: bool,
    paint_cache: PaintCacheState,
    interaction_cache: prepaint::InteractionCacheState,

    dirty_cache_roots: HashSet<NodeId>,
    dirty_cache_root_reasons:
        HashMap<NodeId, (UiDebugInvalidationSource, UiDebugInvalidationDetail)>,
    last_redraw_request_tick: Option<TickId>,

    propagation_depth_cache: HashMap<NodeId, u32>,
    propagation_chain: Vec<NodeId>,
    propagation_entries: Vec<(u8, u32, u64, NodeId, Invalidation)>,
    propagation_visited: HashMap<NodeId, u8>,

    semantics: Option<Arc<SemanticsSnapshot>>,
    semantics_requested: bool,
    deferred_cleanup: Vec<Box<dyn Widget<H>>>,
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

impl<H: UiHost> Default for UiTree<H> {
    fn default() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            layers: SlotMap::with_key(),
            layer_order: Vec::new(),
            root_to_layer: HashMap::new(),
            base_layer: None,
            focus: None,
            captured: HashMap::new(),
            last_pointer_move_hit: HashMap::new(),
            touch_pointer_down_outside_candidates: HashMap::new(),
            hit_test_path_cache: None,
            last_internal_drag_target: None,
            window: None,
            ime_allowed: false,
            ime_composing: false,
            suppress_text_input_until_key_up: None,
            pending_shortcut: PendingShortcut::default(),
            replaying_pending_shortcut: false,
            observed_in_layout: ObservationIndex::default(),
            observed_in_paint: ObservationIndex::default(),
            observed_globals_in_layout: GlobalObservationIndex::default(),
            observed_globals_in_paint: GlobalObservationIndex::default(),
            measure_stack: Vec::new(),
            measure_reentrancy_diagnostics: MeasureReentrancyDiagnostics::default(),
            layout_engine: crate::layout_engine::TaffyLayoutEngine::default(),
            viewport_roots: Vec::new(),
            pending_barrier_relayouts: Vec::new(),
            debug_enabled: false,
            debug_stats: UiDebugFrameStats::default(),
            debug_view_cache_roots: Vec::new(),
            debug_view_cache_contained_relayout_roots: Vec::new(),
            debug_paint_cache_replays: HashMap::new(),
            debug_layout_engine_solves: Vec::new(),
            debug_measure_children: HashMap::new(),
            debug_invalidation_walks: Vec::new(),
            debug_model_change_hotspots: Vec::new(),
            debug_model_change_unobserved: Vec::new(),
            debug_global_change_hotspots: Vec::new(),
            debug_global_change_unobserved: Vec::new(),
            debug_hover_edge_this_frame: false,
            debug_hover_declarative_invalidations: HashMap::new(),
            debug_dirty_views: Vec::new(),
            #[cfg(feature = "diagnostics")]
            debug_set_children_writes: HashMap::new(),
            #[cfg(feature = "diagnostics")]
            debug_removed_subtrees: Vec::new(),
            view_cache_enabled: false,
            paint_cache_policy: PaintCachePolicy::Auto,
            inspection_active: false,
            paint_cache: PaintCacheState::default(),
            interaction_cache: prepaint::InteractionCacheState::default(),
            dirty_cache_roots: HashSet::new(),
            dirty_cache_root_reasons: HashMap::new(),
            last_redraw_request_tick: None,
            propagation_depth_cache: HashMap::new(),
            propagation_chain: Vec::new(),
            propagation_entries: Vec::new(),
            propagation_visited: HashMap::new(),
            semantics: None,
            semantics_requested: false,
            deferred_cleanup: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct MeasureReentrancyDiagnostics {
    /// Frame ID of the last emitted warning.
    last_log_frame: Option<FrameId>,
    /// Number of suppressed re-entrancy events since the last emitted warning.
    suppressed_since_last_log: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DebugMeasureChildRecord {
    total_time: Duration,
    calls: u64,
}

impl MeasureReentrancyDiagnostics {
    const MIN_FRAMES_BETWEEN_LOGS: u64 = 120;

    fn record(&mut self, frame_id: FrameId) -> Option<u64> {
        let should_log = match self.last_log_frame {
            None => true,
            Some(last) => frame_id.0.saturating_sub(last.0) >= Self::MIN_FRAMES_BETWEEN_LOGS,
        };

        if !should_log {
            self.suppressed_since_last_log = self.suppressed_since_last_log.saturating_add(1);
            return None;
        }

        self.last_log_frame = Some(frame_id);
        Some(std::mem::take(&mut self.suppressed_since_last_log))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MeasureStackKey {
    node: NodeId,
    known_w_bits: Option<u32>,
    known_h_bits: Option<u32>,
    avail_w: (u8, u32),
    avail_h: (u8, u32),
    scale_bits: u32,
}

impl<H: UiHost> UiTree<H> {
    fn invalidation_marks_view_dirty(
        source: UiDebugInvalidationSource,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) -> bool {
        matches!(
            source,
            UiDebugInvalidationSource::Notify
                | UiDebugInvalidationSource::ModelChange
                | UiDebugInvalidationSource::GlobalChange
        ) || (detail == UiDebugInvalidationDetail::ScrollHandle && inv == Invalidation::Layout)
    }

    pub(crate) fn request_redraw_coalesced(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            return;
        };
        let tick = app.tick_id();
        if self.last_redraw_request_tick == Some(tick) {
            return;
        }
        self.last_redraw_request_tick = Some(tick);
        app.request_redraw(window);
    }

    fn mark_cache_root_dirty(
        &mut self,
        root: NodeId,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        self.dirty_cache_roots.insert(root);
        self.dirty_cache_root_reasons.insert(root, (source, detail));
    }

    pub(crate) fn begin_debug_frame_if_needed(&mut self, frame_id: FrameId) {
        if !self.debug_enabled {
            return;
        }
        if self.debug_stats.frame_id == frame_id {
            return;
        }

        self.debug_stats.frame_id = frame_id;
        self.debug_stats.model_change_invalidation_roots = 0;
        self.debug_stats.model_change_models = 0;
        self.debug_stats.model_change_observation_edges = 0;
        self.debug_stats.model_change_unobserved_models = 0;
        self.debug_stats.global_change_invalidation_roots = 0;
        self.debug_stats.global_change_globals = 0;
        self.debug_stats.global_change_observation_edges = 0;
        self.debug_stats.global_change_unobserved_globals = 0;
        self.debug_stats.invalidation_walk_nodes = 0;
        self.debug_stats.invalidation_walk_calls = 0;
        self.debug_stats.invalidation_walk_nodes_model_change = 0;
        self.debug_stats.invalidation_walk_calls_model_change = 0;
        self.debug_stats.invalidation_walk_nodes_global_change = 0;
        self.debug_stats.invalidation_walk_calls_global_change = 0;
        self.debug_stats.invalidation_walk_nodes_hover = 0;
        self.debug_stats.invalidation_walk_calls_hover = 0;
        self.debug_stats.invalidation_walk_nodes_focus = 0;
        self.debug_stats.invalidation_walk_calls_focus = 0;
        self.debug_stats.invalidation_walk_nodes_other = 0;
        self.debug_stats.invalidation_walk_calls_other = 0;
        self.debug_stats.hover_pressable_target_changes = 0;
        self.debug_stats.hover_hover_region_target_changes = 0;
        self.debug_stats.hover_declarative_instance_changes = 0;
        self.debug_stats.hover_declarative_hit_test_invalidations = 0;
        self.debug_stats.hover_declarative_layout_invalidations = 0;
        self.debug_stats.hover_declarative_paint_invalidations = 0;
        self.debug_stats.view_cache_active = self.view_cache_active();
        self.debug_stats.view_cache_invalidation_truncations = 0;
        self.debug_stats.view_cache_contained_relayouts = 0;
        self.debug_stats.set_children_barrier_writes = 0;
        self.debug_stats.barrier_relayouts_scheduled = 0;
        self.debug_stats.barrier_relayouts_performed = 0;
        self.debug_stats.virtual_list_visible_range_checks = 0;
        self.debug_stats.virtual_list_visible_range_refreshes = 0;

        self.debug_view_cache_roots.clear();
        self.debug_view_cache_contained_relayout_roots.clear();
        self.debug_paint_cache_replays.clear();
        self.debug_layout_engine_solves.clear();
        self.debug_measure_children.clear();
        self.debug_invalidation_walks.clear();
        self.debug_model_change_hotspots.clear();
        self.debug_model_change_unobserved.clear();
        self.debug_global_change_hotspots.clear();
        self.debug_global_change_unobserved.clear();
        self.debug_hover_edge_this_frame = false;
        self.debug_hover_declarative_invalidations.clear();
        self.debug_dirty_views.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_set_children_writes.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_removed_subtrees.clear();
        let mut dirty_roots: Vec<NodeId> = self.dirty_cache_roots.iter().copied().collect();
        dirty_roots.sort_by_key(|id| id.data().as_ffi());
        for root in dirty_roots {
            let element = self.nodes.get(root).and_then(|n| n.element);
            let (source, detail) = self
                .dirty_cache_root_reasons
                .get(&root)
                .copied()
                .unwrap_or((
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::Unknown,
                ));
            self.debug_dirty_views.push(UiDebugDirtyView {
                view: ViewId(root),
                element,
                source,
                detail,
            });
        }
    }

    pub(crate) fn debug_record_hover_edge_pressable(&mut self) {
        if !self.debug_enabled {
            return;
        }
        self.debug_hover_edge_this_frame = true;
        self.debug_stats.hover_pressable_target_changes = self
            .debug_stats
            .hover_pressable_target_changes
            .saturating_add(1);
    }

    pub(crate) fn debug_record_hover_edge_hover_region(&mut self) {
        if !self.debug_enabled {
            return;
        }
        self.debug_hover_edge_this_frame = true;
        self.debug_stats.hover_hover_region_target_changes = self
            .debug_stats
            .hover_hover_region_target_changes
            .saturating_add(1);
    }

    pub(crate) fn debug_record_hover_declarative_invalidation(
        &mut self,
        node: NodeId,
        hit_test: bool,
        layout: bool,
        paint: bool,
    ) {
        if !self.debug_enabled || !self.debug_hover_edge_this_frame {
            return;
        }

        self.debug_stats.hover_declarative_instance_changes = self
            .debug_stats
            .hover_declarative_instance_changes
            .saturating_add(1);

        self.debug_stats.hover_declarative_hit_test_invalidations = self
            .debug_stats
            .hover_declarative_hit_test_invalidations
            .saturating_add(hit_test as u32);
        self.debug_stats.hover_declarative_layout_invalidations = self
            .debug_stats
            .hover_declarative_layout_invalidations
            .saturating_add(layout as u32);
        self.debug_stats.hover_declarative_paint_invalidations = self
            .debug_stats
            .hover_declarative_paint_invalidations
            .saturating_add(paint as u32);

        let entry = self
            .debug_hover_declarative_invalidations
            .entry(node)
            .or_default();
        entry.hit_test = entry.hit_test.saturating_add(hit_test as u32);
        entry.layout = entry.layout.saturating_add(layout as u32);
        entry.paint = entry.paint.saturating_add(paint as u32);
    }

    pub(crate) fn debug_record_measure_child(
        &mut self,
        parent: NodeId,
        child: NodeId,
        elapsed: Duration,
    ) {
        if !self.debug_enabled {
            return;
        }
        let entry = self
            .debug_measure_children
            .entry(parent)
            .or_default()
            .entry(child)
            .or_default();
        entry.total_time += elapsed;
        entry.calls = entry.calls.saturating_add(1);
    }

    fn debug_take_top_measure_children(
        &mut self,
        parent: NodeId,
        max: usize,
    ) -> Vec<(NodeId, DebugMeasureChildRecord)> {
        let Some(children) = self.debug_measure_children.remove(&parent) else {
            return Vec::new();
        };
        let mut items: Vec<(NodeId, DebugMeasureChildRecord)> = children.into_iter().collect();
        items.sort_by_key(|(_, r)| std::cmp::Reverse(r.total_time));
        items.truncate(max);
        items
    }

    pub(crate) fn debug_record_view_cache_root(
        &mut self,
        root: NodeId,
        reused: bool,
        contained_layout: bool,
        reuse_reason: UiDebugCacheRootReuseReason,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_view_cache_roots.push(DebugViewCacheRootRecord {
            root,
            reused,
            contained_layout,
            reuse_reason,
        });
    }

    pub(crate) fn debug_record_paint_cache_replay(&mut self, node: NodeId, replayed_ops: u32) {
        if !self.debug_enabled {
            return;
        }
        *self.debug_paint_cache_replays.entry(node).or_default() += replayed_ops;
    }

    pub(crate) fn debug_record_layout_engine_solve(
        &mut self,
        root: NodeId,
        solve_time: Duration,
        measure_calls: u64,
        measure_cache_hits: u64,
        measure_time: Duration,
        top_measures: Vec<UiDebugLayoutEngineMeasureHotspot>,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_layout_engine_solves
            .push(UiDebugLayoutEngineSolve {
                root,
                solve_time,
                measure_calls,
                measure_cache_hits,
                measure_time,
                top_measures,
            });
    }

    pub fn debug_cache_root_stats(&self) -> Vec<UiDebugCacheRootStats> {
        if !self.debug_enabled {
            return Vec::new();
        }

        let mut out: Vec<UiDebugCacheRootStats> = self
            .debug_view_cache_roots
            .iter()
            .map(|r| UiDebugCacheRootStats {
                root: r.root,
                element: self.nodes.get(r.root).and_then(|n| n.element),
                reused: r.reused,
                contained_layout: r.contained_layout,
                paint_replayed_ops: self
                    .debug_paint_cache_replays
                    .get(&r.root)
                    .copied()
                    .unwrap_or(0),
                reuse_reason: r.reuse_reason,
            })
            .collect();

        out.sort_by_key(|s| std::cmp::Reverse(s.paint_replayed_ops));
        out
    }

    pub fn debug_view_cache_contained_relayout_roots(&self) -> &[NodeId] {
        if !self.debug_enabled {
            return &[];
        }
        &self.debug_view_cache_contained_relayout_roots
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_set_children_write_for(&self, parent: NodeId) -> Option<UiDebugSetChildrenWrite> {
        if !self.debug_enabled {
            return None;
        }
        self.debug_set_children_writes.get(&parent).copied()
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_removed_subtrees(&self) -> &[UiDebugRemoveSubtreeRecord] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_removed_subtrees.as_slice()
    }

    pub fn debug_layout_engine_solves(&self) -> &[UiDebugLayoutEngineSolve] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_layout_engine_solves.as_slice()
    }

    pub(crate) fn node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
    }

    pub(crate) fn node_needs_layout(&self, node: NodeId) -> bool {
        self.nodes.get(node).is_some_and(|n| n.invalidation.layout)
    }

    pub(crate) fn set_node_element(&mut self, node: NodeId, element: Option<GlobalElementId>) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.element = element;
        }
    }

    pub(crate) fn node_element(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes.get(node).and_then(|n| n.element)
    }

    pub(crate) fn should_reuse_view_cache_node(&self, node: NodeId) -> bool {
        if !self.view_cache_active() {
            return false;
        }
        let Some(n) = self.nodes.get(node) else {
            return false;
        };
        if !n.view_cache.enabled {
            return false;
        }
        if n.view_cache_needs_rerender {
            return false;
        }
        // View-cache reuse is an authoring-level "skip re-render" decision, not a "skip repaint"
        // decision: paint invalidations (e.g. hover/focus) should not force a child render pass.
        !n.invalidation.layout
    }

    pub(crate) fn set_node_view_cache_flags(
        &mut self,
        node: NodeId,
        enabled: bool,
        contained_layout: bool,
        layout_definite: bool,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.view_cache = ViewCacheFlags {
                enabled,
                contained_layout,
                layout_definite,
            };
        }
    }

    pub(crate) fn set_node_view_cache_needs_rerender(&mut self, node: NodeId, needs: bool) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.view_cache_needs_rerender = needs;
        }
        if !needs {
            self.dirty_cache_roots.remove(&node);
            self.dirty_cache_root_reasons.remove(&node);
        }
    }

    /// Repair invalidation propagation for newly mounted auto-sized cache roots.
    ///
    /// During declarative mounting we may discover `ViewCache` roots before their parent pointers
    /// are fully connected. When view caching is active, invalidation propagation can be
    /// truncated at cache roots, and a cache root that is only marked dirty on itself may never be
    /// laid out by its (still-clean) ancestors. This shows up as cache-root subtrees stuck at
    /// `Rect::default()` origins (e.g. scripted clicks using semantics bounds land in the wrong
    /// place).
    ///
    /// Call this after `repair_parent_pointers_from_layer_roots()` and before `layout_all` so the
    /// next layout pass walks far enough to place newly mounted cache-root subtrees.
    pub(crate) fn propagate_auto_sized_view_cache_root_invalidations(&mut self) {
        if !self.view_cache_active() {
            return;
        }

        let targets: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, n)| {
                (n.view_cache.enabled
                    && n.view_cache.contained_layout
                    && !n.view_cache.layout_definite
                    && (n.invalidation.layout || n.invalidation.hit_test))
                    .then_some(id)
            })
            .collect();

        for root in targets {
            self.mark_invalidation_with_source(
                root,
                Invalidation::HitTest,
                UiDebugInvalidationSource::Other,
            );
        }
    }

    pub(crate) fn take_layout_engine(&mut self) -> crate::layout_engine::TaffyLayoutEngine {
        std::mem::take(&mut self.layout_engine)
    }

    pub(crate) fn put_layout_engine(&mut self, engine: crate::layout_engine::TaffyLayoutEngine) {
        self.layout_engine = engine;
    }

    pub(crate) fn register_viewport_root(&mut self, root: NodeId, bounds: Rect) {
        self.viewport_roots.push((root, bounds));
    }

    #[allow(dead_code)]
    pub(crate) fn viewport_roots(&self) -> &[(NodeId, Rect)] {
        &self.viewport_roots
    }

    fn set_ime_allowed(&mut self, app: &mut H, enabled: bool) {
        if self.ime_allowed == enabled {
            return;
        }
        self.ime_allowed = enabled;
        let Some(window) = self.window else {
            return;
        };
        app.push_effect(Effect::ImeAllow { window, enabled });
    }

    fn enforce_modal_barrier_scope(&mut self, active_roots: &[NodeId]) {
        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, active_roots))
        {
            self.focus = None;
        }
        let to_remove: Vec<PointerId> = self
            .captured
            .iter()
            .filter_map(|(p, n)| (!self.node_in_any_layer(*n, active_roots)).then_some(*p))
            .collect();
        for p in to_remove {
            self.captured.remove(&p);
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn node_exists(&self, node: NodeId) -> bool {
        self.nodes.contains_key(node)
    }

    pub(crate) fn flush_deferred_cleanup(&mut self, services: &mut dyn UiServices) {
        for mut widget in self.deferred_cleanup.drain(..) {
            widget.cleanup_resources(services);
        }
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    pub(crate) fn debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    pub fn debug_stats(&self) -> UiDebugFrameStats {
        self.debug_stats
    }

    pub fn debug_hover_declarative_invalidation_hotspots(
        &self,
        max: usize,
    ) -> Vec<UiDebugHoverDeclarativeInvalidationHotspot> {
        if !self.debug_enabled || max == 0 {
            return Vec::new();
        }

        let mut out: Vec<UiDebugHoverDeclarativeInvalidationHotspot> = self
            .debug_hover_declarative_invalidations
            .iter()
            .map(
                |(&node, counts)| UiDebugHoverDeclarativeInvalidationHotspot {
                    node,
                    element: self.nodes.get(node).and_then(|n| n.element),
                    hit_test: counts.hit_test,
                    layout: counts.layout,
                    paint: counts.paint,
                },
            )
            .collect();

        out.sort_by_key(|hs| {
            (
                std::cmp::Reverse(hs.layout),
                std::cmp::Reverse(hs.hit_test),
                std::cmp::Reverse(hs.paint),
            )
        });
        out.truncate(max);
        out
    }

    pub fn debug_invalidation_walks(&self) -> &[UiDebugInvalidationWalk] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_invalidation_walks.as_slice()
    }

    pub fn debug_dirty_views(&self) -> &[UiDebugDirtyView] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_dirty_views.as_slice()
    }

    pub fn debug_model_change_hotspots(&self) -> &[UiDebugModelChangeHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_model_change_hotspots.as_slice()
    }

    pub fn debug_model_change_unobserved(&self) -> &[UiDebugModelChangeUnobserved] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_model_change_unobserved.as_slice()
    }
    pub fn debug_global_change_hotspots(&self) -> &[UiDebugGlobalChangeHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_global_change_hotspots.as_slice()
    }

    pub fn debug_global_change_unobserved(&self) -> &[UiDebugGlobalChangeUnobserved] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_global_change_unobserved.as_slice()
    }
    pub fn captured_for(&self, pointer_id: PointerId) -> Option<NodeId> {
        self.captured.get(&pointer_id).copied()
    }

    pub fn set_paint_cache_policy(&mut self, policy: PaintCachePolicy) {
        self.paint_cache_policy = policy;
    }

    pub fn paint_cache_policy(&self) -> PaintCachePolicy {
        self.paint_cache_policy
    }

    pub fn set_view_cache_enabled(&mut self, enabled: bool) {
        self.view_cache_enabled = enabled;
    }

    pub fn view_cache_enabled(&self) -> bool {
        self.view_cache_enabled
    }

    pub fn set_inspection_active(&mut self, active: bool) {
        self.inspection_active = active;
    }

    pub fn inspection_active(&self) -> bool {
        self.inspection_active
    }

    pub fn set_paint_cache_enabled(&mut self, enabled: bool) {
        self.set_paint_cache_policy(if enabled {
            PaintCachePolicy::Enabled
        } else {
            PaintCachePolicy::Disabled
        });
    }

    pub fn paint_cache_enabled(&self) -> bool {
        match self.paint_cache_policy {
            PaintCachePolicy::Auto => !self.inspection_active,
            PaintCachePolicy::Enabled => true,
            PaintCachePolicy::Disabled => false,
        }
    }

    fn view_cache_active(&self) -> bool {
        self.view_cache_enabled && !self.inspection_active
    }

    fn nearest_view_cache_root(&self, node: NodeId) -> Option<NodeId> {
        let mut current = Some(node);
        while let Some(id) = current {
            let n = self.nodes.get(id)?;
            if n.view_cache.enabled {
                return Some(id);
            }
            current = n.parent;
        }
        None
    }

    fn notify_target_for_node(&self, node: NodeId) -> NodeId {
        self.nearest_view_cache_root(node)
            .unwrap_or_else(|| self.node_root(node).unwrap_or(node))
    }

    fn collapse_observation_index_to_view_cache_roots(
        &self,
        mut index: ObservationIndex,
    ) -> ObservationIndex {
        let mut per_root: HashMap<NodeId, HashMap<ModelId, ObservationMask>> = HashMap::new();
        for (node, entries) in index.by_node.drain() {
            let target = self.nearest_view_cache_root(node).unwrap_or(node);
            let models = per_root.entry(target).or_default();
            for (model, mask) in entries {
                models
                    .entry(model)
                    .and_modify(|m| *m = m.union(mask))
                    .or_insert(mask);
            }
        }

        let mut out = ObservationIndex::default();
        for (node, models) in per_root {
            let mut list: Vec<(ModelId, ObservationMask)> = Vec::with_capacity(models.len());
            for (model, mask) in models {
                list.push((model, mask));
            }
            out.by_node.insert(node, list.clone());
            for (model, mask) in list {
                out.by_model.entry(model).or_default().insert(node, mask);
            }
        }
        out
    }

    fn collapse_global_observation_index_to_view_cache_roots(
        &self,
        mut index: GlobalObservationIndex,
    ) -> GlobalObservationIndex {
        let mut per_root: HashMap<NodeId, HashMap<TypeId, ObservationMask>> = HashMap::new();
        for (node, entries) in index.by_node.drain() {
            let target = self.nearest_view_cache_root(node).unwrap_or(node);
            let globals = per_root.entry(target).or_default();
            for (global, mask) in entries {
                globals
                    .entry(global)
                    .and_modify(|m| *m = m.union(mask))
                    .or_insert(mask);
            }
        }

        let mut out = GlobalObservationIndex::default();
        for (node, globals) in per_root {
            let mut list: Vec<(TypeId, ObservationMask)> = Vec::with_capacity(globals.len());
            for (global, mask) in globals {
                list.push((global, mask));
            }
            out.by_node.insert(node, list.clone());
            for (global, mask) in list {
                out.by_global.entry(global).or_default().insert(node, mask);
            }
        }
        out
    }

    fn collapse_layout_observations_to_view_cache_roots_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let observed_in_layout = std::mem::take(&mut self.observed_in_layout);
        self.observed_in_layout =
            self.collapse_observation_index_to_view_cache_roots(observed_in_layout);

        let observed_globals_in_layout = std::mem::take(&mut self.observed_globals_in_layout);
        self.observed_globals_in_layout =
            self.collapse_global_observation_index_to_view_cache_roots(observed_globals_in_layout);
    }

    fn collapse_paint_observations_to_view_cache_roots_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let observed_in_paint = std::mem::take(&mut self.observed_in_paint);
        self.observed_in_paint =
            self.collapse_observation_index_to_view_cache_roots(observed_in_paint);

        let observed_globals_in_paint = std::mem::take(&mut self.observed_globals_in_paint);
        self.observed_globals_in_paint =
            self.collapse_global_observation_index_to_view_cache_roots(observed_globals_in_paint);
    }

    fn expand_view_cache_layout_invalidations_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let targets: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, n)| (n.view_cache.enabled && n.invalidation.layout).then_some(id))
            .collect();
        if targets.is_empty() {
            return;
        }
        for root in targets {
            self.mark_view_cache_layout_dirty_subtree(root);
        }
    }

    fn mark_view_cache_layout_dirty_subtree(&mut self, root: NodeId) {
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(id) = stack.pop() {
            let Some(n) = self.nodes.get_mut(id) else {
                continue;
            };
            n.invalidation.mark(Invalidation::Layout);
            for &child in &n.children {
                stack.push(child);
            }
        }
    }

    /// Ingest the previous frame's recorded ops from `scene` for paint-cache replay.
    ///
    /// Call this **before** clearing `scene` for the next frame.
    ///
    /// Important:
    /// - This method is destructive: it swaps the scene op storage into the UI tree. Do not call
    ///   it more than once for the same `Scene` before `Scene::clear()`.
    /// - `scene` must contain the previous frame ops that were produced by **this** `UiTree`.
    /// - The paint cache records absolute op index ranges into the previous frame ops vector, so
    ///   sharing a single `Scene` across multiple `UiTree`s is not compatible with paint-cache
    ///   ingestion unless each tree records into an isolated scene.
    pub fn ingest_paint_cache_source(&mut self, scene: &mut Scene) {
        scene.swap_storage(
            &mut self.paint_cache.prev_ops,
            &mut self.paint_cache.prev_fingerprint,
        );
    }

    pub fn request_semantics_snapshot(&mut self) {
        self.semantics_requested = true;
    }

    pub fn semantics_snapshot(&self) -> Option<&SemanticsSnapshot> {
        self.semantics.as_deref()
    }

    pub fn semantics_snapshot_arc(&self) -> Option<Arc<SemanticsSnapshot>> {
        self.semantics.clone()
    }

    pub fn captured(&self) -> Option<NodeId> {
        self.captured_for(PointerId(0))
    }

    pub fn any_captured_node(&self) -> Option<NodeId> {
        self.captured.values().copied().next()
    }

    pub fn input_arbitration_snapshot(&self) -> UiInputArbitrationSnapshot {
        let (_active, barrier_root) = self.active_input_layers();

        let (pointer_occlusion_layer, pointer_occlusion) = self
            .topmost_pointer_occlusion_layer(barrier_root)
            .map(|(layer, occlusion)| (Some(layer), occlusion))
            .unwrap_or((None, PointerOcclusion::None));

        let mut pointer_capture_active = false;
        let mut pointer_capture_layer: Option<UiLayerId> = None;
        let mut pointer_capture_multiple_layers = false;
        for &node in self.captured.values() {
            pointer_capture_active = true;
            let Some(layer) = self.node_layer(node) else {
                pointer_capture_layer = None;
                pointer_capture_multiple_layers = true;
                break;
            };

            match pointer_capture_layer {
                None => pointer_capture_layer = Some(layer),
                Some(prev) => {
                    if prev != layer {
                        pointer_capture_layer = None;
                        pointer_capture_multiple_layers = true;
                        break;
                    }
                }
            }
        }

        UiInputArbitrationSnapshot {
            modal_barrier_root: barrier_root,
            pointer_occlusion,
            pointer_occlusion_layer,
            pointer_capture_active,
            pointer_capture_layer,
            pointer_capture_multiple_layers,
        }
    }

    pub fn debug_node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
    }

    /// Returns the node bounds after applying the accumulated `render_transform` stack.
    ///
    /// This is intended for debugging and tests that need screen-space geometry for overlay
    /// placement/hit-testing scenarios. Unlike `debug_node_bounds`, this includes render-time
    /// transforms such as `Anchored` placement.
    ///
    /// This is not a stable cross-frame geometry query (see
    /// `fret_ui::elements::visual_bounds_for_element` for that contract).
    pub fn debug_node_visual_bounds(&self, node: NodeId) -> Option<Rect> {
        let bounds = self.nodes.get(node).map(|n| n.bounds)?;
        let path = self.debug_node_path(node);
        let mut before = Transform2D::IDENTITY;
        let mut transform = Transform2D::IDENTITY;
        for (idx, id) in path.iter().copied().enumerate() {
            let node_transform = self
                .node_render_transform(id)
                .unwrap_or(Transform2D::IDENTITY);
            let at_node = before.compose(node_transform);
            if id == node {
                transform = at_node;
                break;
            }
            let child_transform = self
                .node_children_render_transform(id)
                .unwrap_or(Transform2D::IDENTITY);
            before = at_node.compose(child_transform);

            // Defensive: if the node wasn't found in `path`, keep identity.
            if idx == path.len().saturating_sub(1) {
                transform = at_node;
            }
        }

        Some(rect_aabb_transformed(bounds, transform))
    }

    pub(crate) fn layout_engine_child_local_rect(
        &self,
        parent: NodeId,
        child: NodeId,
    ) -> Option<Rect> {
        self.layout_engine
            .child_layout_rect_if_solved(parent, child)
    }

    #[allow(dead_code)]
    pub(crate) fn flow_subtree_is_engine_backed(&self, root: NodeId) -> bool {
        let Some(&child) = self.children(root).first() else {
            return false;
        };
        self.layout_engine_child_local_rect(root, child).is_some()
    }

    #[cfg(test)]
    pub(crate) fn layout_engine_has_node(&self, node: NodeId) -> bool {
        self.layout_engine.layout_id_for_node(node).is_some()
    }

    pub fn debug_node_path(&self, node: NodeId) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = Vec::new();
        let mut current = Some(node);
        while let Some(id) = current {
            out.push(id);
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        out.reverse();
        out
    }

    pub fn debug_layers_in_paint_order(&self) -> Vec<UiDebugLayerInfo> {
        self.layer_order
            .iter()
            .copied()
            .filter_map(|id| {
                let layer = self.layers.get(id)?;
                Some(UiDebugLayerInfo {
                    id,
                    root: layer.root,
                    visible: layer.visible,
                    blocks_underlay_input: layer.blocks_underlay_input,
                    hit_testable: layer.hit_testable,
                    pointer_occlusion: layer.pointer_occlusion,
                    wants_pointer_down_outside_events: layer.wants_pointer_down_outside_events,
                    wants_pointer_move_events: layer.wants_pointer_move_events,
                    wants_timer_events: layer.wants_timer_events,
                })
            })
            .collect()
    }

    pub fn debug_hit_test(&self, position: Point) -> UiDebugHitTest {
        let (active_roots, barrier_root) = self.active_input_layers();
        let hit = self.hit_test_layers(&active_roots, position);
        UiDebugHitTest {
            hit,
            active_layer_roots: active_roots,
            barrier_root,
        }
    }

    pub fn set_window(&mut self, window: AppWindowId) {
        self.window = Some(window);
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub fn set_focus(&mut self, focus: Option<NodeId>) {
        if self.focus != focus {
            self.ime_composing = false;
        }
        self.focus = focus;
    }

    const TOUCH_POINTER_DOWN_OUTSIDE_SLOP_PX: f32 = 6.0;

    fn update_touch_pointer_down_outside_move(&mut self, pointer_id: PointerId, position: Point) {
        let Some(candidate) = self
            .touch_pointer_down_outside_candidates
            .get_mut(&pointer_id)
        else {
            return;
        };
        if candidate.moved {
            return;
        }
        let dx = position.x.0 - candidate.start_pos.x.0;
        let dy = position.y.0 - candidate.start_pos.y.0;
        if (dx * dx + dy * dy).sqrt() > Self::TOUCH_POINTER_DOWN_OUTSIDE_SLOP_PX {
            candidate.moved = true;
        }
    }

    pub(crate) fn create_node(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.nodes.insert(Node::new(widget))
    }

    #[cfg(test)]
    pub(crate) fn create_node_for_element(
        &mut self,
        element: GlobalElementId,
        widget: impl Widget<H> + 'static,
    ) -> NodeId {
        self.nodes.insert(Node::new_for_element(element, widget))
    }

    pub fn set_root(&mut self, root: NodeId) {
        let _ = self.set_base_root(root);
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(node) = self.nodes.get_mut(child) {
            node.parent = Some(parent);
        }
        if let Some(node) = self.nodes.get_mut(parent) {
            node.children.push(child);
            node.invalidation.hit_test = true;
            node.invalidation.layout = true;
            node.invalidation.paint = true;
        }
    }

    #[track_caller]
    pub fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
        let Some(_old_len) = self.nodes.get(parent).map(|n| n.children.len()) else {
            return;
        };

        // Keep parent pointers consistent even when the child list is unchanged.
        //
        // This matters for view-cache reuse and GC/repair flows where a node may be temporarily
        // detached and then re-attached without changing the parent's child list. Invalidation
        // propagation relies on `parent` pointers even when semantics/debug traversals use the
        // child lists.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            for &child in &children {
                if let Some(n) = self.nodes.get_mut(child) {
                    n.parent = Some(parent);
                }
            }
            return;
        }

        #[cfg(feature = "diagnostics")]
        if self.debug_enabled {
            let location = std::panic::Location::caller();
            self.debug_set_children_writes.insert(
                parent,
                UiDebugSetChildrenWrite {
                    parent,
                    frame_id: self.debug_stats.frame_id,
                    old_len: _old_len.min(u32::MAX as usize) as u32,
                    new_len: children.len().min(u32::MAX as usize) as u32,
                    file: location.file(),
                    line: location.line(),
                    column: location.column(),
                },
            );
        }

        let Some(old_children) = self
            .nodes
            .get_mut(parent)
            .map(|n| std::mem::take(&mut n.children))
        else {
            return;
        };

        for old in old_children {
            if let Some(n) = self.nodes.get_mut(old)
                && n.parent == Some(parent)
            {
                n.parent = None;
            }
        }

        for &child in &children {
            if let Some(n) = self.nodes.get_mut(child) {
                n.parent = Some(parent);
            }
        }

        let mut propagate = false;
        if let Some(n) = self.nodes.get_mut(parent) {
            n.children = children;
            n.invalidation.hit_test = true;
            n.invalidation.layout = true;
            n.invalidation.paint = true;
            propagate = true;
        }

        if propagate {
            // Structural changes must invalidate ancestors so the next layout pass walks far
            // enough to place newly mounted subtrees, even when view-cache invalidation
            // truncation is enabled.
            self.mark_invalidation_with_source(
                parent,
                Invalidation::HitTest,
                UiDebugInvalidationSource::Other,
            );
        }
    }

    /// Set a node's child list without forcing ancestor relayout.
    ///
    /// This is intended for explicit layout barriers (virtualization, scroll, etc.) whose bounds
    /// are stable and do not depend on the size or presence of their children. In these cases,
    /// structural changes should not require re-laying out ancestors, but the subtree still needs
    /// a contained relayout to place newly mounted children.
    ///
    /// The tree will schedule a contained relayout for `parent` during the next layout pass.
    #[track_caller]
    pub(crate) fn set_children_barrier(&mut self, parent: NodeId, children: Vec<NodeId>) {
        let Some(_old_len) = self.nodes.get(parent).map(|n| n.children.len()) else {
            return;
        };

        // Keep parent pointers consistent even when the child list is unchanged.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            for &child in &children {
                if let Some(n) = self.nodes.get_mut(child) {
                    n.parent = Some(parent);
                }
            }
            return;
        }

        #[cfg(feature = "diagnostics")]
        if self.debug_enabled {
            let location = std::panic::Location::caller();
            self.debug_set_children_writes.insert(
                parent,
                UiDebugSetChildrenWrite {
                    parent,
                    frame_id: self.debug_stats.frame_id,
                    old_len: _old_len.min(u32::MAX as usize) as u32,
                    new_len: children.len().min(u32::MAX as usize) as u32,
                    file: location.file(),
                    line: location.line(),
                    column: location.column(),
                },
            );
        }

        if self.debug_enabled {
            self.debug_stats.set_children_barrier_writes = self
                .debug_stats
                .set_children_barrier_writes
                .saturating_add(1);
            self.debug_stats.barrier_relayouts_scheduled = self
                .debug_stats
                .barrier_relayouts_scheduled
                .saturating_add(1);
        }

        let Some(old_children) = self
            .nodes
            .get_mut(parent)
            .map(|n| std::mem::take(&mut n.children))
        else {
            return;
        };

        for old in old_children {
            if let Some(n) = self.nodes.get_mut(old)
                && n.parent == Some(parent)
            {
                n.parent = None;
            }
        }

        for &child in &children {
            if let Some(n) = self.nodes.get_mut(child) {
                n.parent = Some(parent);
            }
        }

        if let Some(n) = self.nodes.get_mut(parent) {
            n.children = children;
            n.invalidation.hit_test = true;
            n.invalidation.layout = true;
            n.invalidation.paint = true;
        }

        // Structural changes must invalidate paint/hit-testing so routing and rendering see the
        // updated tree, but we intentionally avoid forcing a full ancestor relayout.
        self.mark_invalidation_with_source(
            parent,
            Invalidation::HitTestOnly,
            UiDebugInvalidationSource::Other,
        );

        self.pending_barrier_relayouts.push(parent);
    }

    pub(crate) fn debug_record_virtual_list_visible_range_check(
        &mut self,
        requested_refresh: bool,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.virtual_list_visible_range_checks = self
            .debug_stats
            .virtual_list_visible_range_checks
            .saturating_add(1);
        if requested_refresh {
            self.debug_stats.virtual_list_visible_range_refreshes = self
                .debug_stats
                .virtual_list_visible_range_refreshes
                .saturating_add(1);
        }
    }

    pub(crate) fn take_pending_barrier_relayouts(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.pending_barrier_relayouts)
    }

    #[track_caller]
    pub fn remove_subtree(&mut self, services: &mut dyn UiServices, root: NodeId) -> Vec<NodeId> {
        #[cfg(feature = "diagnostics")]
        let remove_record = if self.debug_enabled {
            let location = std::panic::Location::caller();
            let pre_exists = self.nodes.contains_key(root);
            let root_element = self.nodes.get(root).and_then(|n| n.element);
            let root_parent = self.nodes.get(root).and_then(|n| n.parent);
            let root_parent_element =
                root_parent.and_then(|p| self.nodes.get(p).and_then(|n| n.element));
            let root_root = self.node_root(root);
            let root_layer = self.node_layer(root);
            let root_children_len = self
                .nodes
                .get(root)
                .map(|n| n.children.len().min(u32::MAX as usize) as u32)
                .unwrap_or(0);
            let root_parent_children_len = root_parent.and_then(|p| {
                self.nodes
                    .get(p)
                    .map(|n| n.children.len().min(u32::MAX as usize) as u32)
            });
            let mut root_path: [u64; 16] = [0u64; 16];
            let mut root_path_len: u8 = 0;
            let mut root_path_truncated = false;
            let mut current = Some(root);
            while let Some(id) = current {
                if (root_path_len as usize) >= root_path.len() {
                    root_path_truncated = true;
                    break;
                }
                root_path[root_path_len as usize] = id.data().as_ffi();
                root_path_len = root_path_len.saturating_add(1);
                current = self.nodes.get(id).and_then(|n| n.parent);
            }
            Some((
                location.file(),
                location.line(),
                location.column(),
                pre_exists,
                root_element,
                root_parent,
                root_parent_element,
                root_root,
                root_layer,
                root_children_len,
                root_parent_children_len,
                root_path_len,
                root_path,
                root_path_truncated,
            ))
        } else {
            None
        };

        if self.root_to_layer.contains_key(&root) {
            #[cfg(feature = "diagnostics")]
            if let Some((
                file,
                line,
                column,
                _pre_exists,
                root_element,
                root_parent,
                root_parent_element,
                root_root,
                root_layer,
                root_children_len,
                root_parent_children_len,
                root_path_len,
                root_path,
                root_path_truncated,
            )) = remove_record
            {
                self.debug_removed_subtrees
                    .push(UiDebugRemoveSubtreeRecord {
                        outcome: UiDebugRemoveSubtreeOutcome::SkippedLayerRoot,
                        frame_id: self.debug_stats.frame_id,
                        root,
                        root_element,
                        root_parent,
                        root_parent_element,
                        root_root,
                        root_layer,
                        root_children_len,
                        root_parent_children_len,
                        root_path_len,
                        root_path,
                        root_path_truncated,
                        removed_nodes: 0,
                        removed_head_len: 0,
                        removed_head: [0u64; 16],
                        removed_tail_len: 0,
                        removed_tail: [0u64; 16],
                        file,
                        line,
                        column,
                    });
            }
            return Vec::new();
        }
        let mut removed: Vec<NodeId> = Vec::new();
        self.remove_subtree_inner(services, root, &mut removed);

        #[cfg(feature = "diagnostics")]
        if let Some((
            file,
            line,
            column,
            pre_exists,
            root_element,
            root_parent,
            root_parent_element,
            root_root,
            root_layer,
            root_children_len,
            root_parent_children_len,
            root_path_len,
            root_path,
            root_path_truncated,
        )) = remove_record
        {
            let outcome = if pre_exists {
                UiDebugRemoveSubtreeOutcome::Removed
            } else {
                UiDebugRemoveSubtreeOutcome::RootMissing
            };

            let mut removed_head: [u64; 16] = [0u64; 16];
            let mut removed_head_len: u8 = 0;
            for (idx, node) in removed.iter().take(16).enumerate() {
                removed_head[idx] = node.data().as_ffi();
                removed_head_len = removed_head_len.saturating_add(1);
            }

            let mut removed_tail: [u64; 16] = [0u64; 16];
            let mut removed_tail_len: u8 = 0;
            for (idx, node) in removed.iter().rev().take(16).enumerate() {
                removed_tail[idx] = node.data().as_ffi();
                removed_tail_len = removed_tail_len.saturating_add(1);
            }

            self.debug_removed_subtrees
                .push(UiDebugRemoveSubtreeRecord {
                    outcome,
                    frame_id: self.debug_stats.frame_id,
                    root,
                    root_element,
                    root_parent,
                    root_parent_element,
                    root_root,
                    root_layer,
                    root_children_len,
                    root_parent_children_len,
                    root_path_len,
                    root_path,
                    root_path_truncated,
                    removed_nodes: removed.len().min(u32::MAX as usize) as u32,
                    removed_head_len,
                    removed_head,
                    removed_tail_len,
                    removed_tail,
                    file,
                    line,
                    column,
                });
        }

        removed
    }

    fn remove_subtree_inner(
        &mut self,
        services: &mut dyn UiServices,
        root: NodeId,
        removed: &mut Vec<NodeId>,
    ) {
        // Avoid recursion: removing or cleaning up deep trees can overflow the stack.
        //
        // We remove nodes in a post-order traversal so children are removed before their parent.
        let mut stack: Vec<(NodeId, bool)> = Vec::new();
        stack.push((root, false));

        while let Some((node, children_pushed)) = stack.pop() {
            if self.root_to_layer.contains_key(&node) {
                continue;
            }
            let Some(n) = self.nodes.get(node) else {
                continue;
            };

            if !children_pushed {
                let children = n.children.clone();
                stack.push((node, true));
                for child in children {
                    stack.push((child, false));
                }
                continue;
            }

            let parent = self.nodes.get(node).and_then(|n| n.parent);
            if let Some(parent) = parent
                && let Some(p) = self.nodes.get_mut(parent)
            {
                p.children.retain(|&c| c != node);
            }

            if self.focus == Some(node) {
                self.focus = None;
            }
            self.captured.retain(|_, n| *n != node);

            self.cleanup_node_resources(services, node);
            self.nodes.remove(node);
            self.observed_in_layout.remove_node(node);
            self.observed_in_paint.remove_node(node);
            self.observed_globals_in_layout.remove_node(node);
            self.observed_globals_in_paint.remove_node(node);
            removed.push(node);
        }
    }

    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(parent)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    /// Best-effort repair pass for parent pointers based on child edges from layer roots.
    ///
    /// Parent pointers are used for cache-root discovery (`nearest_view_cache_root`) and for
    /// determining whether nodes are attached to any layer (`node_layer`). If a bug or GC edge
    /// case leaves a reachable node with a missing/incorrect `parent`, this can cascade into
    /// incorrect invalidation truncation and overly-aggressive subtree sweeping.
    ///
    /// This intentionally only walks nodes reachable from the installed layer roots; it does not
    /// attempt to "rescue" detached islands.
    pub(crate) fn repair_parent_pointers_from_layer_roots(&mut self) -> u32 {
        let roots = self.all_layer_roots();
        if roots.is_empty() {
            return 0;
        }

        let mut repaired: u32 = 0;
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(Option<NodeId>, NodeId)> = Vec::with_capacity(roots.len());
        for root in roots {
            stack.push((None, root));
        }

        while let Some((expected_parent, node)) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }

            let (current_parent, children) = match self.nodes.get(node) {
                Some(n) => (n.parent, n.children.clone()),
                None => continue,
            };

            if current_parent != expected_parent {
                if let Some(n) = self.nodes.get_mut(node) {
                    n.parent = expected_parent;
                    repaired = repaired.saturating_add(1);
                }
            }

            for child in children {
                stack.push((Some(node), child));
            }
        }

        repaired
    }

    pub fn node_parent(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.parent)
    }

    pub fn first_focusable_ancestor_including_declarative(
        &self,
        app: &mut H,
        window: AppWindowId,
        start: NodeId,
    ) -> Option<NodeId> {
        let mut node = Some(start);
        while let Some(id) = node {
            let focusable = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    _ => false,
                }
            } else {
                self.nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable())
            };

            if focusable {
                return Some(id);
            }

            node = self.nodes.get(id).and_then(|n| n.parent);
        }
        None
    }

    pub fn first_focusable_descendant(&self, root: NodeId) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let focusable = self
                .nodes
                .get(id)
                .and_then(|n| n.widget.as_ref())
                .is_some_and(|w| w.is_focusable());
            if focusable {
                return Some(id);
            }

            if let Some(node) = self.nodes.get(id) {
                let traverse_children = node
                    .widget
                    .as_ref()
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                if traverse_children {
                    for &child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }
            }
        }
        None
    }

    /// Like `first_focusable_descendant`, but also considers declarative element instances that
    /// haven't run layout yet.
    ///
    /// This is needed because declarative nodes derive focusability from their element instance
    /// (`PressableProps.focusable`, `TextInput`, ...), and the `ElementHostWidget` only caches that
    /// information during layout. Overlay policies commonly want to set initial focus immediately
    /// after installing an overlay root, before layout runs.
    pub fn first_focusable_descendant_including_declarative(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
    ) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let (focusable, traverse_children) = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                let focusable = match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    _ => false,
                };
                let traverse_children = match &record.instance {
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled,
                    crate::declarative::ElementInstance::InteractivityGate(p) => {
                        p.present && p.interactive
                    }
                    crate::declarative::ElementInstance::Spinner(_) => false,
                    _ => true,
                };
                (focusable, traverse_children)
            } else {
                let traverse_children = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                let focusable = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable());
                (focusable, traverse_children)
            };

            if focusable {
                return Some(id);
            }

            if traverse_children && let Some(node) = self.nodes.get(id) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }

    /// Like `first_focusable_descendant_including_declarative`, but treats `InteractivityGate`
    /// as a *pointer/activation* gate, not a traversal boundary for initial focus.
    ///
    /// This is useful for overlay autofocus policies where content may be temporarily
    /// non-interactive (e.g. during motion) but still present and should be eligible for focus.
    pub fn first_focusable_descendant_including_declarative_present_only(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
    ) -> Option<NodeId> {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let (focusable, traverse_children) = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                let focusable = match &record.instance {
                    crate::declarative::ElementInstance::TextInput(_) => true,
                    crate::declarative::ElementInstance::TextArea(_) => true,
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled && p.focusable,
                    _ => false,
                };
                let traverse_children = match &record.instance {
                    crate::declarative::ElementInstance::Pressable(p) => p.enabled,
                    crate::declarative::ElementInstance::InteractivityGate(p) => p.present,
                    crate::declarative::ElementInstance::Spinner(_) => false,
                    _ => true,
                };
                (focusable, traverse_children)
            } else {
                let traverse_children = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true);
                let focusable = self
                    .nodes
                    .get(id)
                    .and_then(|n| n.widget.as_ref())
                    .is_some_and(|w| w.is_focusable());
                (focusable, traverse_children)
            };

            if focusable {
                return Some(id);
            }

            if traverse_children && let Some(node) = self.nodes.get(id) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }

    fn dispatch_pointer_down_outside(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        params: PointerDownOutsideParams<'_>,
        invalidation_visited: &mut HashMap<NodeId, u8>,
    ) -> PointerDownOutsideOutcome {
        let hit = params.hit;
        let hit_root = hit.and_then(|n| self.node_root(n));

        let (event_pointer_id, touch_candidate): (
            Option<PointerId>,
            Option<(PointerId, Point, Event)>,
        ) = match params.event {
            Event::Pointer(PointerEvent::Down {
                pointer_id,
                position,
                pointer_type: fret_core::PointerType::Touch,
                ..
            }) => (
                Some(*pointer_id),
                Some((*pointer_id, *position, params.event.clone())),
            ),
            Event::Pointer(PointerEvent::Down { pointer_id, .. }) => (Some(*pointer_id), None),
            _ => (None, None),
        };

        if let Some((pointer_id, _, _)) = touch_candidate {
            self.touch_pointer_down_outside_candidates
                .remove(&pointer_id);
        }

        // Only the topmost "dismissable" non-modal overlay should observe outside presses.
        // This mirrors Radix-style DismissableLayer semantics while staying click-through:
        // the observer pass must not block the underlying hit-tested dispatch.
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers.into_iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            if layer.root == params.base_root {
                continue;
            }
            if layer.blocks_underlay_input {
                continue;
            }
            if !params.active_layer_roots.contains(&layer.root) {
                continue;
            }

            // If another pointer is captured by a different UI layer, treat the active capture as
            // exclusive for outside-press dismissal.
            //
            // This avoids accidental multi-pointer dismissal while editor-style interactions
            // (viewport tools, drags) are in progress (ADR 0049).
            if let Some(event_pointer_id) = event_pointer_id
                && self.captured.iter().any(|(pid, node)| {
                    *pid != event_pointer_id
                        && self
                            .node_layer(*node)
                            .is_some_and(|layer| layer != layer_id)
                })
            {
                return PointerDownOutsideOutcome::default();
            }

            // If the pointer event is inside this layer, it will be handled by the normal hit-test
            // dispatch. Do not dismiss anything under it.
            if hit_root == Some(layer.root) {
                break;
            }

            // Radix-aligned outcome: allow per-layer "branches" that should not trigger outside
            // dismissal even though they live outside the layer subtree (e.g. trigger elements).
            if hit.is_some_and(|hit| {
                layer
                    .pointer_down_outside_branches
                    .iter()
                    .copied()
                    .any(|branch| self.is_descendant(branch, hit))
            }) {
                break;
            }

            if !layer.wants_pointer_down_outside_events {
                continue;
            }

            let root = layer.root;
            let consume = layer.consume_pointer_down_outside_events;

            if let Some((pointer_id, position, down_event)) = touch_candidate {
                // Radix-aligned touch behavior: delay outside-press dismissal until pointer-up,
                // and cancel it when the touch turns into a scroll/drag gesture.
                self.touch_pointer_down_outside_candidates.insert(
                    pointer_id,
                    TouchPointerDownOutsideCandidate {
                        layer_id,
                        root,
                        consume,
                        down_event,
                        start_pos: position,
                        moved: false,
                    },
                );
                return PointerDownOutsideOutcome::default();
            }

            self.dispatch_event_to_node_chain_observer(
                app,
                services,
                params.input_ctx,
                root,
                params.event,
                invalidation_visited,
            );
            return PointerDownOutsideOutcome {
                dispatched: true,
                suppress_hit_test_dispatch: consume,
            };
        }

        PointerDownOutsideOutcome::default()
    }

    fn rects_intersect(a: Rect, b: Rect) -> bool {
        let ax0 = a.origin.x.0;
        let ay0 = a.origin.y.0;
        let ax1 = ax0 + a.size.width.0;
        let ay1 = ay0 + a.size.height.0;

        let bx0 = b.origin.x.0;
        let by0 = b.origin.y.0;
        let bx1 = bx0 + b.size.width.0;
        let by1 = by0 + b.size.height.0;

        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }

    fn collect_focusables(
        &self,
        node: NodeId,
        active_layers: &[NodeId],
        scope_bounds: Rect,
        out: &mut Vec<NodeId>,
    ) {
        if !self.node_in_any_layer(node, active_layers) {
            return;
        }

        let Some(n) = self.nodes.get(node) else {
            return;
        };
        if n.bounds.size.width.0 <= 0.0 || n.bounds.size.height.0 <= 0.0 {
            return;
        }
        if !Self::rects_intersect(n.bounds, scope_bounds)
            && !self.node_has_scrollable_ancestor_in_scope(node, active_layers, scope_bounds)
        {
            return;
        }

        if n.widget.as_ref().is_some_and(|w| w.is_focusable()) {
            out.push(node);
        }

        let traverse_children = n
            .widget
            .as_ref()
            .map(|w| w.focus_traversal_children())
            .unwrap_or(true);
        if traverse_children {
            for &child in &n.children {
                self.collect_focusables(child, active_layers, scope_bounds, out);
            }
        }
    }

    fn node_has_scrollable_ancestor_in_scope(
        &self,
        mut node: NodeId,
        active_layers: &[NodeId],
        scope_bounds: Rect,
    ) -> bool {
        loop {
            let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;

            if !self.node_in_any_layer(node, active_layers) {
                return false;
            }

            let Some(n) = self.nodes.get(node) else {
                return false;
            };
            if n.bounds.size.width.0 <= 0.0 || n.bounds.size.height.0 <= 0.0 {
                continue;
            }
            if !Self::rects_intersect(n.bounds, scope_bounds) {
                continue;
            }

            if n.widget
                .as_ref()
                .is_some_and(|w| w.can_scroll_descendant_into_view())
            {
                return true;
            }
        }
    }

    fn focus_is_text_input(&mut self) -> bool {
        let Some(focus) = self.focus else {
            return false;
        };
        if self
            .nodes
            .get(focus)
            .and_then(|n| n.widget.as_ref())
            .is_none()
        {
            return false;
        }
        self.with_widget_mut(focus, |widget, _tree| widget.is_text_input())
    }

    pub fn cleanup_subtree(&mut self, services: &mut dyn UiServices, root: NodeId) {
        // Avoid recursion: deep trees can overflow the stack during cleanup.
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            let Some(n) = self.nodes.get(node) else {
                continue;
            };
            let children = n.children.clone();
            for child in children {
                stack.push(child);
            }

            self.cleanup_node_resources(services, node);
        }
    }

    fn cleanup_node_resources(&mut self, services: &mut dyn UiServices, node: NodeId) {
        let widget = self.nodes.get_mut(node).and_then(|n| n.widget.take());
        if let Some(mut widget) = widget {
            widget.cleanup_resources(services);
            if let Some(n) = self.nodes.get_mut(node) {
                n.widget = Some(widget);
            } else {
                self.deferred_cleanup.push(widget);
            }
        }
    }

    fn with_widget_mut<R>(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut dyn Widget<H>, &mut UiTree<H>) -> R,
    ) -> R {
        let Some(n) = self.nodes.get_mut(node) else {
            panic!("node must exist: {node:?}");
        };
        let Some(widget) = n.widget.take() else {
            panic!("node widget must exist: {node:?}");
        };
        let mut widget = widget;
        let result = f(widget.as_mut(), self);
        if let Some(n) = self.nodes.get_mut(node) {
            n.widget = Some(widget);
        } else {
            self.deferred_cleanup.push(widget);
        }
        result
    }

    fn node_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.render_transform(n.bounds)?;
        t.inverse().is_some().then_some(t)
    }

    pub(crate) fn node_children_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.children_render_transform(n.bounds)?;
        t.inverse().is_some().then_some(t)
    }

    fn point_in_rounded_rect(bounds: Rect, radii: Corners, position: Point) -> bool {
        if !bounds.contains(position) {
            return false;
        }

        let w = bounds.size.width.0.max(0.0);
        let h = bounds.size.height.0.max(0.0);
        let limit = 0.5 * w.min(h);

        let tl = Px(radii.top_left.0.max(0.0).min(limit));
        let tr = Px(radii.top_right.0.max(0.0).min(limit));
        let br = Px(radii.bottom_right.0.max(0.0).min(limit));
        let bl = Px(radii.bottom_left.0.max(0.0).min(limit));

        let left = bounds.origin.x.0;
        let top = bounds.origin.y.0;
        let right = left + w;
        let bottom = top + h;

        let x = position.x.0;
        let y = position.y.0;

        // Top-left corner
        if tl.0 > 0.0 && x < left + tl.0 && y < top + tl.0 {
            let cx = left + tl.0;
            let cy = top + tl.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= tl.0 * tl.0;
        }

        // Top-right corner
        if tr.0 > 0.0 && x > right - tr.0 && y < top + tr.0 {
            let cx = right - tr.0;
            let cy = top + tr.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= tr.0 * tr.0;
        }

        // Bottom-right corner
        if br.0 > 0.0 && x > right - br.0 && y > bottom - br.0 {
            let cx = right - br.0;
            let cy = bottom - br.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= br.0 * br.0;
        }

        // Bottom-left corner
        if bl.0 > 0.0 && x < left + bl.0 && y > bottom - bl.0 {
            let cx = left + bl.0;
            let cy = bottom - bl.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= bl.0 * bl.0;
        }

        true
    }

    fn mark_invalidation(&mut self, node: NodeId, inv: Invalidation) {
        self.mark_invalidation_with_source(node, inv, UiDebugInvalidationSource::Other);
    }

    fn record_invalidation_walk_call(&mut self, source: UiDebugInvalidationSource) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.invalidation_walk_calls =
            self.debug_stats.invalidation_walk_calls.saturating_add(1);
        match source {
            UiDebugInvalidationSource::ModelChange => {
                self.debug_stats.invalidation_walk_calls_model_change = self
                    .debug_stats
                    .invalidation_walk_calls_model_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::GlobalChange => {
                self.debug_stats.invalidation_walk_calls_global_change = self
                    .debug_stats
                    .invalidation_walk_calls_global_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Notify => {
                self.debug_stats.invalidation_walk_calls_other = self
                    .debug_stats
                    .invalidation_walk_calls_other
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Hover => {
                self.debug_stats.invalidation_walk_calls_hover = self
                    .debug_stats
                    .invalidation_walk_calls_hover
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Focus => {
                self.debug_stats.invalidation_walk_calls_focus = self
                    .debug_stats
                    .invalidation_walk_calls_focus
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Other => {
                self.debug_stats.invalidation_walk_calls_other = self
                    .debug_stats
                    .invalidation_walk_calls_other
                    .saturating_add(1);
            }
        }
    }

    fn record_invalidation_walk_node(&mut self, source: UiDebugInvalidationSource) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.invalidation_walk_nodes =
            self.debug_stats.invalidation_walk_nodes.saturating_add(1);
        match source {
            UiDebugInvalidationSource::ModelChange => {
                self.debug_stats.invalidation_walk_nodes_model_change = self
                    .debug_stats
                    .invalidation_walk_nodes_model_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::GlobalChange => {
                self.debug_stats.invalidation_walk_nodes_global_change = self
                    .debug_stats
                    .invalidation_walk_nodes_global_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Notify => {
                self.debug_stats.invalidation_walk_nodes_other = self
                    .debug_stats
                    .invalidation_walk_nodes_other
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Hover => {
                self.debug_stats.invalidation_walk_nodes_hover = self
                    .debug_stats
                    .invalidation_walk_nodes_hover
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Focus => {
                self.debug_stats.invalidation_walk_nodes_focus = self
                    .debug_stats
                    .invalidation_walk_nodes_focus
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Other => {
                self.debug_stats.invalidation_walk_nodes_other = self
                    .debug_stats
                    .invalidation_walk_nodes_other
                    .saturating_add(1);
            }
        }
    }

    fn mark_invalidation_with_source(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }

    fn mark_invalidation_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let stop_at_view_cache = self.view_cache_active();
        self.record_invalidation_walk_call(source);
        let mut current = Some(node);
        let mut hit_cache_root: Option<NodeId> = None;
        let root_element = self.nodes.get(node).and_then(|n| n.element);
        let mut walked_nodes: u32 = 0;
        while let Some(id) = current {
            if self.nodes.contains_key(id) {
                self.record_invalidation_walk_node(source);
                walked_nodes = walked_nodes.saturating_add(1);
            }
            let mut next_parent: Option<NodeId> = None;
            let mut did_stop = false;
            let mut mark_dirty = false;
            if let Some(n) = self.nodes.get_mut(id) {
                n.invalidation.mark(inv);
                let can_truncate_at_cache_root = inv == Invalidation::Paint
                    || (n.view_cache.contained_layout
                        && n.view_cache.layout_definite
                        && n.bounds.size != Size::default())
                    // For auto-sized cache roots, allow descendant invalidations to truncate at
                    // the first cache boundary we hit. A separate repair step
                    // (`propagate_auto_sized_view_cache_root_invalidations`) will propagate a
                    // single invalidation from the cache root to its ancestors so the root can be
                    // placed before running contained relayouts.
                    //
                    // Importantly, do *not* truncate when the invalidation originates at the
                    // cache root itself (e.g. the repair step), so it can still reach ancestors.
                    || (n.view_cache.contained_layout && !n.view_cache.layout_definite && id != node);
                if stop_at_view_cache && n.view_cache.enabled && can_truncate_at_cache_root {
                    if self.debug_enabled {
                        self.debug_stats.view_cache_invalidation_truncations = self
                            .debug_stats
                            .view_cache_invalidation_truncations
                            .saturating_add(1);
                    }
                    hit_cache_root = Some(id);
                    did_stop = true;
                    if Self::invalidation_marks_view_dirty(source, inv, detail) {
                        n.view_cache_needs_rerender = true;
                        mark_dirty = true;
                    }
                } else {
                    next_parent = n.parent;
                }
            } else {
                break;
            }

            if did_stop {
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                break;
            }
            current = next_parent;
        }

        if self.debug_enabled {
            self.debug_invalidation_walks.push(UiDebugInvalidationWalk {
                root: node,
                root_element,
                inv,
                source,
                detail,
                walked_nodes,
                truncated_at: hit_cache_root,
            });
        }

        // Nested cache-root correctness: if a descendant cache root is invalidated, any ancestor
        // cache roots must also be invalidated for the same categories so they cannot replay stale
        // recorded ranges that include the old descendant output.
        if stop_at_view_cache && let Some(cache_root) = hit_cache_root {
            let mut parent = self.nodes.get(cache_root).and_then(|n| n.parent);
            while let Some(id) = parent {
                let next_parent = self.nodes.get(id).and_then(|n| n.parent);
                let mut mark_dirty = false;
                if let Some(n) = self.nodes.get_mut(id)
                    && n.view_cache.enabled
                {
                    n.invalidation.mark(inv);
                    if Self::invalidation_marks_view_dirty(source, inv, detail) {
                        n.view_cache_needs_rerender = true;
                        mark_dirty = true;
                    }
                }
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                parent = next_parent;
            }
        }
    }

    fn invalidation_mask(inv: Invalidation) -> u8 {
        const PAINT: u8 = 1 << 0;
        const LAYOUT: u8 = 1 << 1;
        const HIT_TEST: u8 = 1 << 2;
        match inv {
            Invalidation::Paint => PAINT,
            Invalidation::Layout => PAINT | LAYOUT,
            Invalidation::HitTest => PAINT | LAYOUT | HIT_TEST,
            Invalidation::HitTestOnly => PAINT | HIT_TEST,
        }
    }

    fn mark_invalidation_dedup_with_source(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        visited: &mut HashMap<NodeId, u8>,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_dedup_with_detail(node, inv, visited, source, detail);
    }

    fn mark_invalidation_dedup_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        visited: &mut HashMap<NodeId, u8>,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let stop_at_view_cache = self.view_cache_active();
        let needed = Self::invalidation_mask(inv);
        if source != UiDebugInvalidationSource::Notify
            && visited
                .get(&node)
                .is_some_and(|already| (*already & needed) == needed)
        {
            return;
        }
        self.record_invalidation_walk_call(source);

        let mut current = Some(node);
        let mut hit_cache_root: Option<NodeId> = None;
        let root_element = self.nodes.get(node).and_then(|n| n.element);
        let mut walked_nodes: u32 = 0;
        while let Some(id) = current {
            let already = visited.get(&id).copied().unwrap_or_default();
            if source != UiDebugInvalidationSource::Notify
                && (already & needed) == needed
                && !(stop_at_view_cache && Self::invalidation_marks_view_dirty(source, inv, detail))
            {
                break;
            }

            if self.nodes.contains_key(id) {
                self.record_invalidation_walk_node(source);
                walked_nodes = walked_nodes.saturating_add(1);
            }
            let mut next_parent: Option<NodeId> = None;
            let mut did_stop = false;
            let mut mark_dirty = false;
            if let Some(n) = self.nodes.get_mut(id) {
                if source == UiDebugInvalidationSource::Notify || (already & needed) != needed {
                    n.invalidation.mark(inv);
                    visited.insert(id, already | needed);
                }

                let can_truncate_at_cache_root = inv == Invalidation::Paint
                    || (n.view_cache.contained_layout
                        && n.view_cache.layout_definite
                        && n.bounds.size != Size::default())
                    || (n.view_cache.contained_layout
                        && !n.view_cache.layout_definite
                        && id != node);
                if stop_at_view_cache && n.view_cache.enabled && can_truncate_at_cache_root {
                    if self.debug_enabled {
                        self.debug_stats.view_cache_invalidation_truncations = self
                            .debug_stats
                            .view_cache_invalidation_truncations
                            .saturating_add(1);
                    }
                    if Self::invalidation_marks_view_dirty(source, inv, detail) {
                        n.view_cache_needs_rerender = true;
                        mark_dirty = true;
                    }
                    hit_cache_root = Some(id);
                    did_stop = true;
                } else {
                    next_parent = n.parent;
                }
            } else {
                break;
            }

            if did_stop {
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                break;
            }
            current = next_parent;
        }

        if self.debug_enabled {
            self.debug_invalidation_walks.push(UiDebugInvalidationWalk {
                root: node,
                root_element,
                inv,
                source,
                detail,
                walked_nodes,
                truncated_at: hit_cache_root,
            });
        }

        // Nested cache-root correctness: if a descendant cache root is invalidated, any ancestor
        // cache roots must also be invalidated for the same categories so they cannot replay stale
        // recorded ranges that include the old descendant output.
        if stop_at_view_cache && let Some(cache_root) = hit_cache_root {
            let mut parent = self.nodes.get(cache_root).and_then(|n| n.parent);
            while let Some(id) = parent {
                let next_parent = self.nodes.get(id).and_then(|n| n.parent);
                let already = visited.get(&id).copied().unwrap_or_default();
                if self.nodes.get(id).is_some_and(|n| n.view_cache.enabled) {
                    let mut mark_dirty = false;
                    if let Some(n) = self.nodes.get_mut(id) {
                        if Self::invalidation_marks_view_dirty(source, inv, detail) {
                            n.view_cache_needs_rerender = true;
                            mark_dirty = true;
                        }
                        if (already & needed) != needed {
                            n.invalidation.mark(inv);
                        }
                    }
                    if mark_dirty {
                        self.mark_cache_root_dirty(id, source, detail);
                    }
                    visited.insert(id, already | needed);
                }
                parent = next_parent;
            }
        }
    }

    pub fn invalidate(&mut self, node: NodeId, inv: Invalidation) {
        self.mark_invalidation(node, inv);
    }

    pub fn invalidate_with_source(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }

    pub fn invalidate_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) {
        self.mark_invalidation_with_detail(node, inv, UiDebugInvalidationSource::Other, detail);
    }

    pub fn invalidate_with_source_and_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }

    fn propagation_depth_for(&mut self, start: NodeId) -> u32 {
        if let Some(depth) = self.propagation_depth_cache.get(&start) {
            return *depth;
        }

        self.propagation_chain.clear();

        let mut current = Some(start);
        while let Some(node) = current {
            if let Some(depth) = self.propagation_depth_cache.get(&node) {
                let mut d = *depth;
                for id in self.propagation_chain.drain(..).rev() {
                    d = d.saturating_add(1);
                    self.propagation_depth_cache.insert(id, d);
                }
                return self
                    .propagation_depth_cache
                    .get(&start)
                    .copied()
                    .unwrap_or_default();
            }

            self.propagation_chain.push(node);
            current = self.nodes.get(node).and_then(|n| n.parent);
        }

        let mut d = 0u32;
        for id in self.propagation_chain.drain(..).rev() {
            self.propagation_depth_cache.insert(id, d);
            d = d.saturating_add(1);
        }

        self.propagation_depth_cache
            .get(&start)
            .copied()
            .unwrap_or_default()
    }

    fn propagate_observation_masks(
        &mut self,
        app: &mut H,
        masks: impl IntoIterator<Item = (NodeId, ObservationMask)>,
        source: UiDebugInvalidationSource,
    ) -> bool {
        self.propagation_depth_cache.clear();
        self.propagation_chain.clear();
        self.propagation_entries.clear();

        for (node, mask) in masks {
            if mask.is_empty() || !self.nodes.contains_key(node) {
                continue;
            }

            let (strength, inv) = if mask.hit_test {
                (3, Invalidation::HitTest)
            } else if mask.layout {
                (2, Invalidation::Layout)
            } else if mask.paint {
                (1, Invalidation::Paint)
            } else {
                continue;
            };

            let depth = self.propagation_depth_for(node);
            let key = node.data().as_ffi();
            self.propagation_entries
                .push((strength, depth, key, node, inv));
        }

        if self.propagation_entries.is_empty() {
            return false;
        }

        self.propagation_entries.sort_by(|a, b| {
            // Higher-strength invalidations first to maximize reuse via `visited`.
            b.0.cmp(&a.0)
                // Within the same strength, prefer ancestors first to reduce redundant walks.
                .then(a.1.cmp(&b.1))
                // Stabilize order for determinism in stats/perf.
                .then(a.2.cmp(&b.2))
        });

        self.propagation_visited.clear();
        let mut did_invalidate = false;
        let mut visited = std::mem::take(&mut self.propagation_visited);
        let mut entries = std::mem::take(&mut self.propagation_entries);
        for (_, _, _, node, inv) in entries.drain(..) {
            self.mark_invalidation_dedup_with_source(node, inv, &mut visited, source);
            did_invalidate = true;
        }
        self.propagation_visited = visited;
        self.propagation_entries = entries;

        if did_invalidate {
            self.request_redraw_coalesced(app);
        }

        did_invalidate
    }

    fn propagate_model_changes_from_elements(&mut self, app: &mut H, changed: &[ModelId]) -> bool {
        let Some(window) = self.window else {
            return false;
        };
        if changed.is_empty() {
            return false;
        }

        let changed: std::collections::HashSet<ModelId> = changed.iter().copied().collect();
        let frame_id = app.frame_id();
        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, _app| {
            let Some(window_state) = runtime.for_window(window) else {
                return;
            };
            window_state.for_each_observed_model_for_invalidation(
                frame_id,
                |element, observations| {
                    let mut mask = ObservationMask::default();
                    for (model, inv) in observations {
                        if changed.contains(model) {
                            mask.add(*inv);
                        }
                    }
                    if mask.is_empty() {
                        return;
                    }
                    let Some(node) = window_state.node_entry(element).map(|e| e.node) else {
                        return;
                    };
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                },
            );
        });

        if combined.is_empty() {
            return false;
        }
        self.propagate_observation_masks(app, combined, UiDebugInvalidationSource::ModelChange)
    }

    fn propagate_global_changes_from_elements(&mut self, app: &mut H, changed: &[TypeId]) -> bool {
        let Some(window) = self.window else {
            return false;
        };
        if changed.is_empty() {
            return false;
        }

        let changed: std::collections::HashSet<TypeId> = changed.iter().copied().collect();
        let frame_id = app.frame_id();
        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, _app| {
            let Some(window_state) = runtime.for_window(window) else {
                return;
            };
            window_state.for_each_observed_global_for_invalidation(
                frame_id,
                |element, observations| {
                    let mut mask = ObservationMask::default();
                    for (global, inv) in observations {
                        if changed.contains(global) {
                            mask.add(*inv);
                        }
                    }
                    if mask.is_empty() {
                        return;
                    }
                    let Some(node) = window_state.node_entry(element).map(|e| e.node) else {
                        return;
                    };
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                },
            );
        });

        if combined.is_empty() {
            return false;
        }
        self.propagate_observation_masks(app, combined, UiDebugInvalidationSource::GlobalChange)
    }

    pub fn propagate_model_changes(&mut self, app: &mut H, changed: &[ModelId]) -> bool {
        if changed.is_empty() {
            return false;
        }
        self.begin_debug_frame_if_needed(app.frame_id());
        if self.debug_enabled {
            self.debug_model_change_hotspots.clear();
            self.debug_model_change_unobserved.clear();
        }

        let mut did_invalidate = false;

        if changed.len() == 1 {
            let model = changed[0];
            let layout_nodes = self.observed_in_layout.by_model.get(&model);
            let paint_nodes = self.observed_in_paint.by_model.get(&model);
            if let (Some(nodes), None) | (None, Some(nodes)) = (layout_nodes, paint_nodes) {
                // Copy out the observations so we don't hold a borrow across the invalidation walk.
                let masks: Vec<(NodeId, ObservationMask)> =
                    nodes.iter().map(|(&n, &m)| (n, m)).collect();
                if self.debug_enabled {
                    self.debug_stats.model_change_invalidation_roots =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.model_change_models = 1;
                    self.debug_stats.model_change_observation_edges =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.model_change_unobserved_models = 0;
                    self.debug_model_change_hotspots = vec![UiDebugModelChangeHotspot {
                        model,
                        observation_edges: masks.len().min(u32::MAX as usize) as u32,
                        changed: app.models().debug_last_changed_info_for_id(model),
                    }];
                }
                did_invalidate |= self.propagate_observation_masks(
                    app,
                    masks,
                    UiDebugInvalidationSource::ModelChange,
                );
                did_invalidate |= self.propagate_model_changes_from_elements(app, changed);
                return did_invalidate;
            }
        }

        let mut combined: HashMap<NodeId, ObservationMask> =
            HashMap::with_capacity(changed.len().saturating_mul(8));
        let mut observation_edges_scanned = 0usize;
        let mut unobserved_models = 0usize;
        for &model in changed {
            let mut edges = 0usize;
            if let Some(nodes) = self.observed_in_layout.by_model.get(&model) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_in_paint.by_model.get(&model) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if self.debug_enabled && edges > 0 {
                self.debug_model_change_hotspots
                    .push(UiDebugModelChangeHotspot {
                        model,
                        observation_edges: edges.min(u32::MAX as usize) as u32,
                        changed: app.models().debug_last_changed_info_for_id(model),
                    });
            }
            if edges == 0 {
                unobserved_models = unobserved_models.saturating_add(1);
                if self.debug_enabled {
                    self.debug_model_change_unobserved
                        .push(UiDebugModelChangeUnobserved {
                            model,
                            created: app.models().debug_created_info_for_id(model),
                            changed: app.models().debug_last_changed_info_for_id(model),
                        });
                }
            }
        }

        if self.debug_enabled {
            self.debug_stats.model_change_invalidation_roots =
                combined.len().min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_models = changed.len().min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_observation_edges =
                observation_edges_scanned.min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_unobserved_models =
                unobserved_models.min(u32::MAX as usize) as u32;

            self.debug_model_change_hotspots
                .sort_by(|a, b| b.observation_edges.cmp(&a.observation_edges));
            self.debug_model_change_hotspots.truncate(5);

            self.debug_model_change_unobserved
                .sort_by(|a, b| a.model.data().as_ffi().cmp(&b.model.data().as_ffi()));
            self.debug_model_change_unobserved.truncate(5);
        }
        did_invalidate |= self.propagate_observation_masks(
            app,
            combined.into_iter(),
            UiDebugInvalidationSource::ModelChange,
        );
        did_invalidate |= self.propagate_model_changes_from_elements(app, changed);
        did_invalidate
    }

    pub fn propagate_global_changes(&mut self, app: &mut H, changed: &[TypeId]) -> bool {
        if changed.is_empty() {
            return false;
        }
        self.begin_debug_frame_if_needed(app.frame_id());
        if self.debug_enabled {
            self.debug_global_change_hotspots.clear();
            self.debug_global_change_unobserved.clear();
        }

        let mut did_invalidate = false;

        if changed.len() == 1 {
            let global = changed[0];
            let layout_nodes = self.observed_globals_in_layout.by_global.get(&global);
            let paint_nodes = self.observed_globals_in_paint.by_global.get(&global);
            if let (Some(nodes), None) | (None, Some(nodes)) = (layout_nodes, paint_nodes) {
                // Copy out the observations so we don't hold a borrow across the invalidation walk.
                let masks: Vec<(NodeId, ObservationMask)> =
                    nodes.iter().map(|(&n, &m)| (n, m)).collect();
                if self.debug_enabled {
                    self.debug_stats.global_change_invalidation_roots =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.global_change_globals = 1;
                    self.debug_stats.global_change_observation_edges =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.global_change_unobserved_globals = 0;
                }
                did_invalidate |= self.propagate_observation_masks(
                    app,
                    masks,
                    UiDebugInvalidationSource::GlobalChange,
                );
                did_invalidate |= self.propagate_global_changes_from_elements(app, changed);
                return did_invalidate;
            }
        }

        let mut combined: HashMap<NodeId, ObservationMask> =
            HashMap::with_capacity(changed.len().saturating_mul(8));
        let mut observation_edges_scanned = 0usize;
        let mut unobserved_globals = 0usize;
        for &global in changed {
            let mut edges = 0usize;
            if let Some(nodes) = self.observed_globals_in_layout.by_global.get(&global) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_globals_in_paint.by_global.get(&global) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if self.debug_enabled && edges > 0 {
                self.debug_global_change_hotspots
                    .push(UiDebugGlobalChangeHotspot {
                        global,
                        observation_edges: edges.min(u32::MAX as usize) as u32,
                    });
            }
            if edges == 0 {
                unobserved_globals = unobserved_globals.saturating_add(1);
                if self.debug_enabled {
                    self.debug_global_change_unobserved
                        .push(UiDebugGlobalChangeUnobserved { global });
                }
            }
        }

        if self.debug_enabled {
            self.debug_stats.global_change_invalidation_roots =
                combined.len().min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_globals = changed.len().min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_observation_edges =
                observation_edges_scanned.min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_unobserved_globals =
                unobserved_globals.min(u32::MAX as usize) as u32;

            self.debug_global_change_hotspots
                .sort_by(|a, b| b.observation_edges.cmp(&a.observation_edges));
            self.debug_global_change_hotspots.truncate(5);

            self.debug_global_change_unobserved
                .sort_by_key(|u| type_id_sort_key(u.global));
            self.debug_global_change_unobserved.truncate(5);
        }
        did_invalidate |= self.propagate_observation_masks(
            app,
            combined.into_iter(),
            UiDebugInvalidationSource::GlobalChange,
        );
        did_invalidate |= self.propagate_global_changes_from_elements(app, changed);
        did_invalidate
    }

    fn refresh_semantics_snapshot(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            self.semantics = None;
            return;
        };

        let base_root = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root));

        let visible_layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        if visible_layers.is_empty() {
            self.semantics = Some(Arc::new(SemanticsSnapshot {
                window,
                ..SemanticsSnapshot::default()
            }));
            return;
        }

        let element_id_map: HashMap<u64, NodeId> =
            crate::declarative::frame::element_id_map_for_window(app, window);

        let mut barrier_index: Option<usize> = None;
        for (idx, layer) in visible_layers.iter().enumerate() {
            if self.layers[*layer].blocks_underlay_input {
                barrier_index = Some(idx);
            }
        }
        let barrier_root = barrier_index.map(|idx| self.layers[visible_layers[idx]].root);

        let mut roots: Vec<SemanticsRoot> = Vec::with_capacity(visible_layers.len());
        for (z, layer_id) in visible_layers.iter().enumerate() {
            let layer = &self.layers[*layer_id];
            roots.push(SemanticsRoot {
                root: layer.root,
                visible: layer.visible,
                blocks_underlay_input: layer.blocks_underlay_input,
                hit_testable: layer.hit_testable,
                z_index: z as u32,
            });
        }

        let focus = self.focus;
        let captured = self.captured_for(PointerId(0));

        let mut nodes: Vec<SemanticsNode> = Vec::with_capacity(self.nodes.len());

        for root in roots.iter().map(|r| r.root) {
            let mut visited: HashSet<NodeId> = HashSet::new();
            // Stack entries carry the transform that maps this node's local bounds into
            // screen-space (excluding this node's own `render_transform`).
            let mut stack: Vec<(NodeId, Transform2D)> = vec![(root, Transform2D::IDENTITY)];
            while let Some((id, before)) = stack.pop() {
                if !visited.insert(id) {
                    if cfg!(debug_assertions) {
                        panic!("cycle detected while building semantics snapshot: node={id:?}");
                    } else {
                        tracing::error!(?id, "cycle detected while building semantics snapshot");
                        continue;
                    }
                }
                let (
                    parent,
                    bounds,
                    children,
                    is_text_input,
                    is_focusable,
                    traverse_children,
                    before_child,
                ) = {
                    let Some(node) = self.nodes.get(id) else {
                        continue;
                    };
                    let widget = node.widget.as_ref();
                    if widget.is_some_and(|w| !w.semantics_present()) {
                        continue;
                    }

                    let node_transform = widget
                        .and_then(|w| w.render_transform(node.bounds))
                        .filter(|t| t.inverse().is_some())
                        .unwrap_or(Transform2D::IDENTITY);
                    let at_node = before.compose(node_transform);
                    let bounds = rect_aabb_transformed(node.bounds, at_node);
                    let children = node.children.clone();
                    let is_text_input = widget.is_some_and(|w| w.is_text_input());
                    let is_focusable = widget.is_some_and(|w| w.is_focusable());
                    let traverse_children = widget.map(|w| w.semantics_children()).unwrap_or(true);
                    let child_transform = widget
                        .and_then(|w| w.children_render_transform(node.bounds))
                        .filter(|t| t.inverse().is_some())
                        .unwrap_or(Transform2D::IDENTITY);
                    let before_child = at_node.compose(child_transform);

                    (
                        node.parent,
                        bounds,
                        children,
                        is_text_input,
                        is_focusable,
                        traverse_children,
                        before_child,
                    )
                };

                let mut role = if Some(id) == base_root {
                    SemanticsRole::Window
                } else {
                    SemanticsRole::Generic
                };
                // Heuristic baseline: text-input widgets should surface as text fields even if
                // they don't implement an explicit semantics hook yet.
                if is_text_input {
                    role = SemanticsRole::TextField;
                }

                let mut flags = fret_core::SemanticsFlags {
                    focused: focus == Some(id),
                    captured: captured == Some(id),
                    ..fret_core::SemanticsFlags::default()
                };

                let mut active_descendant: Option<NodeId> = None;
                let mut pos_in_set: Option<u32> = None;
                let mut set_size: Option<u32> = None;
                let mut label: Option<String> = None;
                let mut value: Option<String> = None;
                let mut test_id: Option<String> = None;
                let mut text_selection: Option<(u32, u32)> = None;
                let mut text_composition: Option<(u32, u32)> = None;
                let mut labelled_by: Vec<NodeId> = Vec::new();
                let mut described_by: Vec<NodeId> = Vec::new();
                let mut controls: Vec<NodeId> = Vec::new();
                let mut actions = fret_core::SemanticsActions {
                    focus: is_focusable || is_text_input,
                    invoke: false,
                    set_value: is_text_input,
                    set_text_selection: is_text_input,
                };

                // Allow widgets to override semantics metadata.
                if let Some(widget) = self.nodes.get_mut(id).and_then(|node| node.widget.as_mut()) {
                    let mut cx = SemanticsCx {
                        app,
                        node: id,
                        window: Some(window),
                        element_id_map: Some(&element_id_map),
                        bounds,
                        children: children.as_slice(),
                        focus,
                        captured,
                        role: &mut role,
                        flags: &mut flags,
                        label: &mut label,
                        value: &mut value,
                        test_id: &mut test_id,
                        text_selection: &mut text_selection,
                        text_composition: &mut text_composition,
                        actions: &mut actions,
                        active_descendant: &mut active_descendant,
                        pos_in_set: &mut pos_in_set,
                        set_size: &mut set_size,
                        labelled_by: &mut labelled_by,
                        described_by: &mut described_by,
                        controls: &mut controls,
                    };
                    widget.semantics(&mut cx);
                }

                if pos_in_set.is_some_and(|p| p == 0) {
                    pos_in_set = None;
                }
                if set_size.is_some_and(|s| s == 0) {
                    set_size = None;
                }
                if let (Some(pos), Some(size)) = (pos_in_set, set_size)
                    && pos > size
                {
                    pos_in_set = None;
                    set_size = None;
                }

                nodes.push(SemanticsNode {
                    id,
                    parent,
                    role,
                    bounds,
                    flags,
                    test_id,
                    active_descendant,
                    pos_in_set,
                    set_size,
                    label,
                    value,
                    text_selection,
                    text_composition,
                    actions,
                    labelled_by,
                    described_by,
                    controls,
                });

                if traverse_children {
                    // Preserve a stable-ish order: visit children in declared order.
                    for &child in children.iter().rev() {
                        stack.push((child, before_child));
                    }
                }
            }
        }

        // Normalize relation edges: for some composite widgets, authoring only sets `labelled_by`
        // (e.g. TabPanel -> Tab) but the platform-facing semantics want the controller to also
        // advertise `controls` (e.g. Tab -> TabPanel). We derive that edge for the subset of
        // role pairs where this bidirectional link is expected.
        let mut index_by_id: HashMap<NodeId, usize> = HashMap::with_capacity(nodes.len());
        for (idx, node) in nodes.iter().enumerate() {
            index_by_id.insert(node.id, idx);
        }
        for idx in 0..nodes.len() {
            let controlled = nodes[idx].id;
            let controlled_role = nodes[idx].role;
            let controllers = nodes[idx].labelled_by.clone();
            for controller in controllers {
                if let Some(&controller_idx) = index_by_id.get(&controller) {
                    let controller_role = nodes[controller_idx].role;
                    let derive = matches!(
                        controlled_role,
                        SemanticsRole::TabPanel | SemanticsRole::ListBox
                    ) && matches!(
                        controller_role,
                        SemanticsRole::Tab
                            | SemanticsRole::TextField
                            | SemanticsRole::ComboBox
                            | SemanticsRole::Button
                    );
                    if !derive {
                        continue;
                    }
                    if !nodes[controller_idx].controls.contains(&controlled) {
                        nodes[controller_idx].controls.push(controlled);
                    }
                }
            }
        }

        self.semantics = Some(Arc::new(SemanticsSnapshot {
            window,
            roots,
            barrier_root,
            focus,
            captured,
            nodes,
        }));

        if let Some(snapshot) = self.semantics.as_deref() {
            semantics::validate_semantics_if_enabled(snapshot);
        }
    }

    fn node_in_any_layer(&self, node: NodeId, layer_roots: &[NodeId]) -> bool {
        let Some(node_root) = self.node_root(node) else {
            return false;
        };
        layer_roots.contains(&node_root)
    }

    fn node_root(&self, mut node: NodeId) -> Option<NodeId> {
        while let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) {
            node = parent;
        }
        self.nodes.contains_key(node).then_some(node)
    }

    pub fn is_descendant(&self, root: NodeId, mut node: NodeId) -> bool {
        if root == node {
            return true;
        }
        while let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) {
            if parent == root {
                return true;
            }
            node = parent;
        }
        false
    }
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

#[cfg(test)]
mod tests;
