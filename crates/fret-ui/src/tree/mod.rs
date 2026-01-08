use crate::{
    Theme, UiHost, declarative,
    elements::GlobalElementId,
    widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::time::{Duration, Instant};
use fret_core::{
    AppWindowId, Corners, Event, KeyCode, NodeId, Point, PointerEvent, Px, Rect, Scene, SceneOp,
    SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size, Transform2D, UiServices,
};
use fret_runtime::{
    CommandId, Effect, FrameId, InputContext, InputDispatchPhase, KeyChord, KeymapService, ModelId,
    Platform, PlatformCapabilities,
};
use slotmap::SlotMap;
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
mod semantics;
mod shortcuts;

use layers::UiLayer;
pub use layers::UiLayerId;
pub use paint_cache::PaintCachePolicy;
use paint_cache::{PaintCacheEntry, PaintCacheKey, PaintCacheState};
use shortcuts::{
    KeydownShortcutParams, PendingShortcut, PointerDownOutsideOutcome, PointerDownOutsideParams,
};

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
        }
    }

    pub fn clear(&mut self) {
        self.layout = false;
        self.paint = false;
        self.hit_test = false;
    }
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
    pub paint_time: Duration,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugLayerInfo {
    pub id: UiLayerId,
    pub root: NodeId,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
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
    captured: Option<NodeId>,
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

    debug_enabled: bool,
    debug_stats: UiDebugFrameStats,

    paint_cache_policy: PaintCachePolicy,
    inspection_active: bool,
    paint_cache: PaintCacheState,

    semantics: Option<Arc<SemanticsSnapshot>>,
    semantics_requested: bool,
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
            captured: None,
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
            debug_enabled: false,
            debug_stats: UiDebugFrameStats::default(),
            paint_cache_policy: PaintCachePolicy::Auto,
            inspection_active: false,
            paint_cache: PaintCacheState::default(),
            semantics: None,
            semantics_requested: false,
        }
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
    pub(crate) fn node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
    }

    pub(crate) fn set_node_element(&mut self, node: NodeId, element: Option<GlobalElementId>) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.element = element;
        }
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
        if self
            .captured
            .is_some_and(|n| !self.node_in_any_layer(n, active_roots))
        {
            self.captured = None;
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    pub fn debug_stats(&self) -> UiDebugFrameStats {
        self.debug_stats
    }

    pub fn set_paint_cache_policy(&mut self, policy: PaintCachePolicy) {
        self.paint_cache_policy = policy;
    }

    pub fn paint_cache_policy(&self) -> PaintCachePolicy {
        self.paint_cache_policy
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

    /// Ingests the previous frame's recorded ops from `scene`.
    ///
    /// Call this **before** clearing `scene` for the next frame.
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
        self.captured
    }

    pub fn debug_node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
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

    pub fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
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
    }

    pub fn remove_subtree(&mut self, services: &mut dyn UiServices, root: NodeId) -> Vec<NodeId> {
        if self.root_to_layer.contains_key(&root) {
            return Vec::new();
        }
        let mut removed: Vec<NodeId> = Vec::new();
        self.remove_subtree_inner(services, root, &mut removed);
        removed
    }

    fn remove_subtree_inner(
        &mut self,
        services: &mut dyn UiServices,
        node: NodeId,
        removed: &mut Vec<NodeId>,
    ) {
        if self.root_to_layer.contains_key(&node) {
            return;
        }
        let Some(n) = self.nodes.get(node) else {
            return;
        };
        let parent = n.parent;
        let children = n.children.clone();

        for child in children {
            self.remove_subtree_inner(services, child, removed);
        }

        if let Some(parent) = parent
            && let Some(p) = self.nodes.get_mut(parent)
        {
            p.children.retain(|&c| c != node);
        }

        if self.focus == Some(node) {
            self.focus = None;
        }
        if self.captured == Some(node) {
            self.captured = None;
        }

        self.cleanup_subtree_inner(services, node);
        self.nodes.remove(node);
        self.observed_in_layout.remove_node(node);
        self.observed_in_paint.remove_node(node);
        self.observed_globals_in_layout.remove_node(node);
        self.observed_globals_in_paint.remove_node(node);
        removed.push(node);
    }

    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(parent)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub fn node_parent(&self, node: NodeId) -> Option<NodeId> {
        self.nodes.get(node).and_then(|n| n.parent)
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

    fn dispatch_pointer_down_outside(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        params: PointerDownOutsideParams<'_>,
    ) -> PointerDownOutsideOutcome {
        let hit = params.hit;
        let hit_root = hit.and_then(|n| self.node_root(n));

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
            self.dispatch_event_to_node_chain_observer(
                app,
                services,
                params.input_ctx,
                root,
                params.event,
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
        self.cleanup_subtree_inner(services, root);
    }

    fn cleanup_subtree_inner(&mut self, services: &mut dyn UiServices, node: NodeId) {
        let Some(n) = self.nodes.get(node) else {
            return;
        };
        let children = n.children.clone();

        self.with_widget_mut(node, |widget, _tree| widget.cleanup_resources(services));

        for child in children {
            self.cleanup_subtree_inner(services, child);
        }
    }

    fn with_widget_mut<R>(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut dyn Widget<H>, &mut UiTree<H>) -> R,
    ) -> R {
        let widget = self
            .nodes
            .get_mut(node)
            .and_then(|n| n.widget.take())
            .expect("node widget must exist");
        let mut widget = widget;
        let result = f(widget.as_mut(), self);
        if let Some(n) = self.nodes.get_mut(node) {
            n.widget = Some(widget);
        }
        result
    }

    fn node_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.render_transform(n.bounds)?;
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
        let mut current = Some(node);
        while let Some(id) = current {
            if let Some(n) = self.nodes.get_mut(id) {
                n.invalidation.mark(inv);
                current = n.parent;
            } else {
                break;
            }
        }
    }

    pub fn invalidate(&mut self, node: NodeId, inv: Invalidation) {
        self.mark_invalidation(node, inv);
    }

    pub fn propagate_model_changes(&mut self, app: &mut H, changed: &[ModelId]) -> bool {
        if changed.is_empty() {
            return false;
        }

        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();
        for &model in changed {
            if let Some(nodes) = self.observed_in_layout.by_model.get(&model) {
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_in_paint.by_model.get(&model) {
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
        }

        let mut did_invalidate = false;
        for (node, mask) in combined {
            if mask.is_empty() || !self.nodes.contains_key(node) {
                continue;
            }
            if mask.hit_test {
                self.mark_invalidation(node, Invalidation::HitTest);
            }
            if mask.layout {
                self.mark_invalidation(node, Invalidation::Layout);
            }
            if mask.paint {
                self.mark_invalidation(node, Invalidation::Paint);
            }
            did_invalidate = true;
        }

        if did_invalidate && let Some(window) = self.window {
            app.request_redraw(window);
        }

        did_invalidate
    }

    pub fn propagate_global_changes(&mut self, app: &mut H, changed: &[TypeId]) -> bool {
        if changed.is_empty() {
            return false;
        }

        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();
        for &global in changed {
            if let Some(nodes) = self.observed_globals_in_layout.by_global.get(&global) {
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_globals_in_paint.by_global.get(&global) {
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
        }

        let mut did_invalidate = false;
        for (node, mask) in combined {
            if mask.is_empty() || !self.nodes.contains_key(node) {
                continue;
            }
            if mask.hit_test {
                self.mark_invalidation(node, Invalidation::HitTest);
            }
            if mask.layout {
                self.mark_invalidation(node, Invalidation::Layout);
            }
            if mask.paint {
                self.mark_invalidation(node, Invalidation::Paint);
            }
            did_invalidate = true;
        }

        if did_invalidate && let Some(window) = self.window {
            app.request_redraw(window);
        }

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
        let captured = self.captured;

        let mut nodes: Vec<SemanticsNode> = Vec::with_capacity(self.nodes.len());

        for root in roots.iter().map(|r| r.root) {
            let mut visited: HashSet<NodeId> = HashSet::new();
            let mut stack: Vec<NodeId> = vec![root];
            while let Some(id) = stack.pop() {
                if !visited.insert(id) {
                    if cfg!(debug_assertions) {
                        panic!("cycle detected while building semantics snapshot: node={id:?}");
                    } else {
                        tracing::error!(?id, "cycle detected while building semantics snapshot");
                        continue;
                    }
                }
                let Some(node) = self.nodes.get_mut(id) else {
                    continue;
                };
                if node.widget.as_ref().is_some_and(|w| !w.semantics_present()) {
                    continue;
                }
                let parent = node.parent;
                let bounds = node.bounds;
                let children = node.children.as_slice();
                let is_text_input = node.widget.as_ref().is_some_and(|w| w.is_text_input());
                let is_focusable = node.widget.as_ref().is_some_and(|w| w.is_focusable());

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
                if let Some(widget) = node.widget.as_mut() {
                    let mut cx = SemanticsCx {
                        app,
                        node: id,
                        window: Some(window),
                        element_id_map: Some(&element_id_map),
                        bounds,
                        children,
                        focus,
                        captured,
                        role: &mut role,
                        flags: &mut flags,
                        label: &mut label,
                        value: &mut value,
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

                let traverse_children = node
                    .widget
                    .as_ref()
                    .map(|w| w.semantics_children())
                    .unwrap_or(true);
                if traverse_children {
                    // Preserve a stable-ish order: visit children in declared order.
                    for &child in children.iter().rev() {
                        stack.push(child);
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
        | PointerEvent::Wheel { position, .. } => *position,
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
        Event::ExternalDrag(e) => Some(e.position),
        Event::InternalDrag(e) => Some(e.position),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
