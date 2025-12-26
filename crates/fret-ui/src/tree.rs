use crate::{
    Theme, UiHost, declarative,
    widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::PlatformCapabilities;
use fret_core::{
    AppWindowId, Corners, Event, FrameId, KeyCode, NodeId, Point, PointerEvent, Px, Rect, Scene,
    SceneOp, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size, UiServices,
};
use fret_runtime::{
    CommandId, DragKind, Effect, InputContext, KeyChord, KeymapService, ModelId, Platform,
};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const PENDING_SHORTCUT_TIMEOUT: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
struct CapturedKeystroke {
    chord: KeyChord,
    text: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct PendingShortcut {
    keystrokes: Vec<CapturedKeystroke>,
    focus: Option<NodeId>,
    barrier_root: Option<NodeId>,
    fallback: Option<CommandId>,
    timer: Option<fret_core::TimerToken>,
    capture_next_text_input_key: Option<KeyCode>,
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
        }
    }

    pub fn clear(&mut self) {
        self.layout = false;
        self.paint = false;
        self.hit_test = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PaintCacheKey {
    width_bits: u32,
    height_bits: u32,
    scale_factor_bits: u32,
    theme_revision: u64,
}

impl PaintCacheKey {
    fn new(bounds: Rect, scale_factor: f32, theme_revision: u64) -> Self {
        Self {
            width_bits: bounds.size.width.0.to_bits(),
            height_bits: bounds.size.height.0.to_bits(),
            scale_factor_bits: scale_factor.to_bits(),
            theme_revision,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PaintCacheEntry {
    generation: u64,
    key: PaintCacheKey,
    origin: Point,
    start: u32,
    end: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PaintCachePolicy {
    /// Enable caching unless the UI is in an inspection/probe mode (e.g. picking, semantics).
    #[default]
    Auto,
    /// Always enable caching.
    Enabled,
    /// Always disable caching.
    Disabled,
}

#[derive(Debug, Default)]
struct PaintCacheState {
    generation: u64,
    prev_ops: Vec<SceneOp>,
    prev_fingerprint: u64,
    source_generation: u64,
    target_generation: u64,
    hits: u32,
    misses: u32,
    replayed_ops: u32,
}

impl PaintCacheState {
    fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_ops = 0;
    }

    fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    fn invalidate_recording(&mut self) {
        self.prev_ops.clear();
        self.prev_fingerprint = 0;
        self.generation = self.generation.saturating_add(1);
    }
}

slotmap::new_key_type! {
    pub struct UiLayerId;
}

#[derive(Debug, Clone)]
struct UiLayer {
    root: NodeId,
    visible: bool,
    blocks_underlay_input: bool,
    hit_testable: bool,
    wants_pointer_move_events: bool,
    wants_timer_events: bool,
}

pub struct Node<H: UiHost> {
    pub widget: Option<Box<dyn Widget<H>>>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub bounds: Rect,
    pub measured_size: Size,
    pub invalidation: InvalidationFlags,
    paint_cache: Option<PaintCacheEntry>,
}

impl<H: UiHost> Node<H> {
    pub fn new(widget: impl Widget<H> + 'static) -> Self {
        Self {
            widget: Some(Box::new(widget)),
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
    by_node: HashMap<NodeId, HashMap<ModelId, ObservationMask>>,
    by_model: HashMap<ModelId, HashMap<NodeId, ObservationMask>>,
}

impl ObservationIndex {
    fn record(&mut self, node: NodeId, observations: Vec<(ModelId, Invalidation)>) {
        let mut next: HashMap<ModelId, ObservationMask> = HashMap::new();
        for (model, inv) in observations {
            next.entry(model).or_default().add(inv);
        }

        let prev = self.by_node.insert(node, next);
        let prev = prev.unwrap_or_default();
        let next = self.by_node.get(&node).cloned().unwrap_or_default();

        for model in prev.keys() {
            if next.contains_key(model) {
                continue;
            }
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
                }
            }
        }

        for (model, mask) in next {
            self.by_model.entry(model).or_default().insert(node, mask);
        }
    }

    fn remove_node(&mut self, node: NodeId) {
        let Some(prev) = self.by_node.remove(&node) else {
            return;
        };
        for model in prev.keys() {
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
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
    suppress_text_input_until_key_up: Option<KeyCode>,
    pending_shortcut: PendingShortcut,
    replaying_pending_shortcut: bool,
    observed_in_layout: ObservationIndex,
    observed_in_paint: ObservationIndex,

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
            suppress_text_input_until_key_up: None,
            pending_shortcut: PendingShortcut::default(),
            replaying_pending_shortcut: false,
            observed_in_layout: ObservationIndex::default(),
            observed_in_paint: ObservationIndex::default(),
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

impl<H: UiHost> UiTree<H> {
    fn dispatch_event_to_node_chain(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
    ) -> bool {
        let services_ptr: *mut dyn UiServices = services;

        let mut node_id = start;
        loop {
            let (invalidations, requested_focus, requested_capture, stop_propagation, parent) =
                self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let children: Vec<NodeId> = tree
                        .nodes
                        .get(node_id)
                        .map(|n| n.children.clone())
                        .unwrap_or_default();
                    let bounds = tree
                        .nodes
                        .get(node_id)
                        .map(|n| n.bounds)
                        .unwrap_or_default();
                    let mut cx = EventCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children: &children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, event);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.stop_propagation,
                        parent,
                    )
                });

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            if let Some(focus) = requested_focus {
                if self.focus != Some(focus) {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }
            }

            if let Some(capture) = requested_capture {
                self.captured = capture;
            };

            if self.captured.is_some() || stop_propagation {
                return true;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        false
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

    fn clear_pending_shortcut(&mut self, app: &mut H) {
        if let Some(token) = self.pending_shortcut.timer.take() {
            app.push_effect(Effect::CancelTimer { token });
        }
        self.pending_shortcut = PendingShortcut::default();
    }

    fn schedule_pending_shortcut_timeout(&mut self, app: &mut H) {
        if self.pending_shortcut.keystrokes.is_empty() {
            return;
        }

        if let Some(token) = self.pending_shortcut.timer.take() {
            app.push_effect(Effect::CancelTimer { token });
        }
        let token = app.next_timer_token();
        self.pending_shortcut.timer = Some(token);
        app.push_effect(Effect::SetTimer {
            window: self.window,
            token,
            after: PENDING_SHORTCUT_TIMEOUT,
            repeat: None,
        });
    }

    fn replay_captured_keystrokes(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        ctx: &InputContext,
        keystrokes: Vec<CapturedKeystroke>,
    ) {
        let prev = self.replaying_pending_shortcut;
        self.replaying_pending_shortcut = true;

        for stroke in keystrokes {
            if let Some(service) = app.global::<KeymapService>()
                && let Some(command) = service.keymap.resolve(ctx, stroke.chord)
            {
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
                continue;
            }

            let down = Event::KeyDown {
                key: stroke.chord.key,
                modifiers: stroke.chord.mods,
                repeat: false,
            };
            self.dispatch_event(app, services, &down);

            if let Some(text) = stroke.text {
                let event = Event::TextInput(text);
                self.dispatch_event(app, services, &event);
            }

            let up = Event::KeyUp {
                key: stroke.chord.key,
                modifiers: stroke.chord.mods,
            };
            self.dispatch_event(app, services, &up);
        }

        self.replaying_pending_shortcut = prev;
    }

    pub fn set_window(&mut self, window: AppWindowId) {
        self.window = Some(window);
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub fn set_focus(&mut self, focus: Option<NodeId>) {
        self.focus = focus;
    }

    pub fn create_node(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.nodes.insert(Node::new(widget))
    }

    pub fn set_base_root(&mut self, root: NodeId) -> UiLayerId {
        if let Some(id) = self.base_layer {
            self.update_layer_root(id, root);
            return id;
        }

        let id = self.layers.insert(UiLayer {
            root,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            wants_pointer_move_events: false,
            wants_timer_events: false,
        });
        self.root_to_layer.insert(root, id);
        self.layer_order.insert(0, id);
        self.base_layer = Some(id);
        id
    }

    pub fn push_overlay_root(&mut self, root: NodeId, blocks_underlay_input: bool) -> UiLayerId {
        self.push_overlay_root_ex(root, blocks_underlay_input, true)
    }

    pub fn push_overlay_root_ex(
        &mut self,
        root: NodeId,
        blocks_underlay_input: bool,
        hit_testable: bool,
    ) -> UiLayerId {
        let id = self.layers.insert(UiLayer {
            root,
            visible: true,
            blocks_underlay_input,
            hit_testable,
            wants_pointer_move_events: false,
            wants_timer_events: false,
        });
        self.root_to_layer.insert(root, id);
        self.layer_order.push(id);
        id
    }

    pub fn set_layer_visible(&mut self, layer: UiLayerId, visible: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.visible = visible;

        if !visible {
            if self
                .captured
                .is_some_and(|n| self.node_layer(n).is_some_and(|lid| lid == layer))
            {
                self.captured = None;
            }
            if self
                .focus
                .is_some_and(|n| self.node_layer(n).is_some_and(|lid| lid == layer))
            {
                self.focus = None;
            }
        }
    }

    pub fn is_layer_visible(&self, layer: UiLayerId) -> bool {
        self.layers.get(layer).is_some_and(|l| l.visible)
    }

    pub fn set_layer_wants_pointer_move_events(&mut self, layer: UiLayerId, wants: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.wants_pointer_move_events = wants;
    }

    pub fn set_layer_wants_timer_events(&mut self, layer: UiLayerId, wants: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.wants_timer_events = wants;
    }

    fn update_layer_root(&mut self, layer: UiLayerId, root: NodeId) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };

        self.root_to_layer.remove(&l.root);
        l.root = root;
        self.root_to_layer.insert(root, layer);
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
        let Some(old_children) = self.nodes.get(parent).map(|n| n.children.clone()) else {
            return;
        };

        for old in old_children {
            if let Some(n) = self.nodes.get_mut(old) {
                if n.parent == Some(parent) {
                    n.parent = None;
                }
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

    pub fn replace_widget(&mut self, node: NodeId, widget: impl Widget<H> + 'static) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.widget = Some(Box::new(widget));
        n.invalidation.hit_test = true;
        n.invalidation.layout = true;
        n.invalidation.paint = true;
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
        removed.push(node);
    }

    pub fn children(&self, parent: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(parent)
            .map(|n| n.children.clone())
            .unwrap_or_default()
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
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }

    pub fn layout_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scale_factor: f32,
    ) {
        let started = self.debug_enabled.then_some(Instant::now());
        if self.debug_enabled {
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.layout_nodes_visited = 0;
            self.debug_stats.layout_nodes_performed = 0;
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured;
        }

        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            let _ = self.layout_in(app, services, root, bounds, scale_factor);
        }

        if self.semantics_requested {
            self.semantics_requested = false;
            self.refresh_semantics_snapshot(app);
        }

        if let Some(started) = started {
            self.debug_stats.layout_time = started.elapsed();
        }
    }

    pub fn paint_all(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
    ) {
        let started = self.debug_enabled.then_some(Instant::now());
        if self.debug_enabled {
            self.debug_stats.frame_id = app.frame_id();
            self.debug_stats.paint_nodes = 0;
            self.debug_stats.paint_nodes_performed = 0;
            self.debug_stats.paint_cache_hits = 0;
            self.debug_stats.paint_cache_misses = 0;
            self.debug_stats.paint_cache_replayed_ops = 0;
            self.debug_stats.focus = self.focus;
            self.debug_stats.captured = self.captured;
        }

        let cache_enabled = self.paint_cache_enabled();
        if cache_enabled {
            self.paint_cache.begin_frame();
        } else {
            self.paint_cache.invalidate_recording();
        }

        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            self.paint(app, services, root, bounds, scene, scale_factor);
        }

        if cache_enabled {
            self.paint_cache.finish_frame();
            if self.debug_enabled {
                self.debug_stats.paint_cache_hits = self.paint_cache.hits;
                self.debug_stats.paint_cache_misses = self.paint_cache.misses;
                self.debug_stats.paint_cache_replayed_ops = self.paint_cache.replayed_ops;
            }
        }

        if let Some(started) = started {
            self.debug_stats.paint_time = started.elapsed();
        }
    }

    pub fn dispatch_event(&mut self, app: &mut H, services: &mut dyn UiServices, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        let focus_is_text_input = self.focus_is_text_input();
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            focus_is_text_input,
        };

        if let Some(window) = self.window {
            let changed = crate::focus_visible::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.invalidate(focus, Invalidation::Paint);
                } else {
                    self.invalidate(base_root, Invalidation::Paint);
                }
                app.request_redraw(window);
            }
        }

        if !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && ((self.pending_shortcut.focus.is_some()
                && self.pending_shortcut.focus != self.focus)
                || self.pending_shortcut.barrier_root != barrier_root)
        {
            self.clear_pending_shortcut(app);
        }

        if let Event::Timer { token } = event
            && !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && self.pending_shortcut.timer == Some(*token)
        {
            let pending = std::mem::take(&mut self.pending_shortcut);
            if let Some(command) = pending.fallback {
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
            } else {
                self.replay_captured_keystrokes(app, services, &input_ctx, pending.keystrokes);
            }
            return;
        }
        if matches!(event, Event::Timer { .. }) {
            let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
            for layer_id in layers.into_iter().rev() {
                let Some(layer) = self.layers.get(layer_id) else {
                    continue;
                };
                if !layer.wants_timer_events || !layer.visible {
                    continue;
                }
                let stopped =
                    self.dispatch_event_to_node_chain(app, services, &input_ctx, layer.root, event);
                if stopped {
                    return;
                }
            }
        }

        if let Event::TextInput(text) = event {
            if !self.replaying_pending_shortcut
                && self.pending_shortcut.capture_next_text_input_key.is_some()
            {
                self.pending_shortcut.capture_next_text_input_key = None;
                if let Some(last) = self.pending_shortcut.keystrokes.last_mut() {
                    last.text = Some(text.clone());
                }
                self.suppress_text_input_until_key_up = None;
                return;
            }

            if self.suppress_text_input_until_key_up.is_some() {
                self.suppress_text_input_until_key_up = None;
                return;
            }
        }

        if let Event::KeyUp { key, .. } = event {
            if self.suppress_text_input_until_key_up == Some(*key) {
                self.suppress_text_input_until_key_up = None;
            }
            if self.pending_shortcut.capture_next_text_input_key == Some(*key) {
                self.pending_shortcut.capture_next_text_input_key = None;
            }
        }

        let mut needs_redraw = false;
        let mut cursor_choice: Option<fret_core::CursorIcon> = None;
        let services_ptr: *mut dyn UiServices = services;

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
        {
            if self.replaying_pending_shortcut {
                // Pending shortcut replay bypasses shortcut matching and sequence state.
            } else if *repeat {
                // Allow key-repeat only for explicitly repeatable commands (e.g. text editing).
                if let Some(service) = app.global::<KeymapService>() {
                    let chord = KeyChord::new(*key, *modifiers);
                    if let Some(command) = service.keymap.resolve(&input_ctx, chord)
                        && app
                            .commands()
                            .get(command.clone())
                            .is_some_and(|m| m.repeatable)
                    {
                        self.suppress_text_input_until_key_up = Some(*key);
                        app.push_effect(Effect::Command {
                            window: self.window,
                            command,
                        });
                        return;
                    }
                }
            } else if let Some(service) = app.global::<KeymapService>() {
                let chord = KeyChord::new(*key, *modifiers);

                if !self.pending_shortcut.keystrokes.is_empty() {
                    self.pending_shortcut
                        .keystrokes
                        .push(CapturedKeystroke { chord, text: None });

                    let sequence: Vec<KeyChord> = self
                        .pending_shortcut
                        .keystrokes
                        .iter()
                        .map(|s| s.chord)
                        .collect();
                    let matched = service.keymap.match_sequence(&input_ctx, &sequence);

                    if matched.has_continuation {
                        self.pending_shortcut.fallback = matched.exact.and_then(|c| c);
                        self.pending_shortcut.focus = self.focus;
                        self.pending_shortcut.barrier_root = barrier_root;
                        self.pending_shortcut.capture_next_text_input_key =
                            (focus_is_text_input && !modifiers.ctrl && !modifiers.meta)
                                .then_some(*key);
                        self.suppress_text_input_until_key_up = Some(*key);
                        self.schedule_pending_shortcut_timeout(app);
                        return;
                    }

                    if let Some(Some(command)) = matched.exact {
                        self.clear_pending_shortcut(app);
                        self.suppress_text_input_until_key_up = Some(*key);
                        app.push_effect(Effect::Command {
                            window: self.window,
                            command,
                        });
                        return;
                    }

                    let pending = std::mem::take(&mut self.pending_shortcut);
                    if let Some(token) = pending.timer {
                        app.push_effect(Effect::CancelTimer { token });
                    }
                    self.replay_captured_keystrokes(app, services, &input_ctx, pending.keystrokes);
                    return;
                }

                let matched = service
                    .keymap
                    .match_sequence(&input_ctx, std::slice::from_ref(&chord));
                if matched.has_continuation {
                    self.pending_shortcut.keystrokes =
                        vec![CapturedKeystroke { chord, text: None }];
                    self.pending_shortcut.focus = self.focus;
                    self.pending_shortcut.barrier_root = barrier_root;
                    self.pending_shortcut.fallback = matched.exact.and_then(|c| c);
                    self.pending_shortcut.capture_next_text_input_key =
                        (focus_is_text_input && !modifiers.ctrl && !modifiers.meta).then_some(*key);
                    self.suppress_text_input_until_key_up = Some(*key);
                    self.schedule_pending_shortcut_timeout(app);
                    return;
                }

                if let Some(command) = service.keymap.resolve(&input_ctx, chord) {
                    self.suppress_text_input_until_key_up = Some(*key);
                    app.push_effect(Effect::Command {
                        window: self.window,
                        command,
                    });
                    return;
                }
            }
        }

        if self
            .captured
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.captured = None;
        }
        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.focus = None;
        }

        let default_root = barrier_root.unwrap_or(base_root);

        // Pointer capture only affects pointer events. Drag-and-drop style events
        // (external/internal) must continue to follow the cursor for correct cross-window UX.
        let captured = match event {
            Event::Pointer(_) => self.captured,
            _ => None,
        };

        // Dock tab drags must be routed to the `DockSpace` root, even if the cursor is over
        // another widget (e.g. menu bar) or outside all hit-testable widgets (tear-off).
        let dock_drag_target = (|| {
            if !matches!(event, Event::InternalDrag(_)) {
                return None;
            }
            let window = self.window?;
            let drag = app.drag()?;
            if !drag.cross_window_hover || drag.kind != DragKind::DockPanel {
                return None;
            }
            let dock = app.global::<crate::DockManager>()?;
            let target = dock.dock_space_node(window)?;
            self.node_in_any_layer(target, &active_layers)
                .then_some(target)
        })();

        if let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
            && let Some(pos) = event_position(event)
        {
            let hit = self.hit_test_layers(&active_layers, pos);
            let hovered_pressable: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let Some(window_frame) = window_frame else {
                        return None;
                    };
                    let mut node = hit;
                    while let Some(id) = node {
                        if let Some(record) = window_frame.instances.get(&id)
                            && matches!(record.instance, declarative::ElementInstance::Pressable(_))
                        {
                            return Some(record.element);
                        }
                        node = self.nodes.get(id).and_then(|n| n.parent);
                    }
                    None
                });

            let (prev_node, next_node) =
                crate::elements::update_hovered_pressable(app, window, hovered_pressable);
            if prev_node.is_some() || next_node.is_some() {
                needs_redraw = true;
                if let Some(node) = prev_node {
                    self.mark_invalidation(node, Invalidation::Paint);
                }
                if let Some(node) = next_node {
                    self.mark_invalidation(node, Invalidation::Paint);
                }
            }

            let hovered_hover_card: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let Some(window_frame) = window_frame else {
                        return None;
                    };
                    let mut node = hit;
                    while let Some(id) = node {
                        if let Some(record) = window_frame.instances.get(&id)
                            && matches!(record.instance, declarative::ElementInstance::HoverCard(_))
                        {
                            return Some(record.element);
                        }
                        node = self.nodes.get(id).and_then(|n| n.parent);
                    }
                    None
                });

            let (prev_node, next_node) =
                crate::elements::update_hovered_hover_card(app, window, hovered_hover_card);
            if prev_node.is_some() || next_node.is_some() {
                needs_redraw = true;
                if let Some(node) = prev_node {
                    self.mark_invalidation(node, Invalidation::Layout);
                    self.mark_invalidation(node, Invalidation::Paint);
                }
                if let Some(node) = next_node {
                    self.mark_invalidation(node, Invalidation::Layout);
                    self.mark_invalidation(node, Invalidation::Paint);
                }
            }
        }

        let target = if let Some(captured) = captured {
            Some(captured)
        } else if let Some(target) = dock_drag_target {
            Some(target)
        } else if let Some(pos) = event_position(event) {
            let hit = self.hit_test_layers(&active_layers, pos);

            if matches!(event, Event::InternalDrag(_)) {
                if let Some(node) = hit {
                    self.last_internal_drag_target = Some(node);
                } else if self
                    .last_internal_drag_target
                    .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
                {
                    self.last_internal_drag_target = None;
                }
            }

            hit.or_else(|| {
                matches!(event, Event::InternalDrag(_)).then_some(self.last_internal_drag_target)?
            })
            .or(barrier_root)
            .or(Some(default_root))
        } else {
            self.focus.or(Some(default_root))
        };

        let Some(mut node_id) = target else {
            return;
        };

        loop {
            let (
                invalidations,
                requested_focus,
                requested_capture,
                requested_cursor,
                stop_propagation,
                parent,
            ) = self.with_widget_mut(node_id, |widget, tree| {
                let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                let children: Vec<NodeId> = tree
                    .nodes
                    .get(node_id)
                    .map(|n| n.children.clone())
                    .unwrap_or_default();
                let bounds = tree
                    .nodes
                    .get(node_id)
                    .map(|n| n.bounds)
                    .unwrap_or_default();
                let mut cx = EventCx {
                    app,
                    services: unsafe { &mut *services_ptr },
                    node: node_id,
                    window: tree.window,
                    input_ctx: input_ctx.clone(),
                    children: &children,
                    focus: tree.focus,
                    captured: tree.captured,
                    bounds,
                    invalidations: Vec::new(),
                    requested_focus: None,
                    requested_capture: None,
                    requested_cursor: None,
                    stop_propagation: false,
                };
                widget.event(&mut cx, event);
                (
                    cx.invalidations,
                    cx.requested_focus,
                    cx.requested_capture,
                    cx.requested_cursor,
                    cx.stop_propagation,
                    parent,
                )
            });

            if !invalidations.is_empty() || requested_focus.is_some() || requested_capture.is_some()
            {
                needs_redraw = true;
            }

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            if let Some(focus) = requested_focus {
                if self.focus != Some(focus) {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }
            }

            if let Some(capture) = requested_capture {
                self.captured = capture;
            };

            if requested_cursor.is_some() && cursor_choice.is_none() {
                cursor_choice = requested_cursor;
            }

            if self.captured.is_some() || stop_propagation {
                break;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        if input_ctx.caps.ui.cursor_icons
            && let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
        {
            let icon = cursor_choice.unwrap_or(fret_core::CursorIcon::Default);
            app.push_effect(Effect::CursorSetIcon { window, icon });
        }

        if needs_redraw && let Some(window) = self.window {
            app.request_redraw(window);
        }
        if let Event::Pointer(PointerEvent::Move { .. }) = event {
            let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
            for layer_id in layers.into_iter().rev() {
                let Some(layer) = self.layers.get(layer_id) else {
                    continue;
                };
                if !layer.wants_pointer_move_events || !layer.visible {
                    continue;
                }
                let _ =
                    self.dispatch_event_to_node_chain(app, services, &input_ctx, layer.root, event);
            }
        }
    }

    pub fn dispatch_command(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        command: &CommandId,
    ) -> bool {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return false;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            focus_is_text_input: self.focus_is_text_input(),
        };

        if self.dispatch_focus_traversal(app, command, &active_layers, barrier_root, base_root) {
            return true;
        }

        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.focus = None;
        }

        let default_root = barrier_root.unwrap_or(base_root);
        let node_id = self.focus.or(Some(default_root));
        let Some(mut node_id) = node_id else {
            return false;
        };

        let mut handled = false;
        let mut needs_redraw = false;
        let services_ptr: *mut dyn UiServices = services;

        loop {
            let (did_handle, invalidations, requested_focus, stop_propagation, parent) = self
                .with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let mut cx = CommandCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        focus: tree.focus,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        stop_propagation: false,
                    };
                    let did_handle = widget.command(&mut cx, command);
                    (
                        did_handle,
                        cx.invalidations,
                        cx.requested_focus,
                        cx.stop_propagation,
                        parent,
                    )
                });

            if did_handle {
                handled = true;
            }

            if !invalidations.is_empty() || requested_focus.is_some() {
                needs_redraw = true;
            }

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            if let Some(focus) = requested_focus {
                if self.focus != Some(focus) {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }
            }

            if did_handle || stop_propagation {
                break;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        if needs_redraw && let Some(window) = self.window {
            app.request_redraw(window);
        }

        handled
    }

    fn dispatch_focus_traversal(
        &mut self,
        app: &mut H,
        command: &CommandId,
        active_layers: &[NodeId],
        barrier_root: Option<NodeId>,
        base_root: NodeId,
    ) -> bool {
        let direction = match command.as_str() {
            "focus.next" => Some(true),
            "focus.previous" => Some(false),
            _ => None,
        };
        let Some(forward) = direction else {
            return false;
        };

        let scope_root = barrier_root.unwrap_or(base_root);
        let scope_bounds = self
            .nodes
            .get(scope_root)
            .map(|n| n.bounds)
            .unwrap_or_default();

        let mut focusables: Vec<NodeId> = Vec::new();
        self.collect_focusables(scope_root, active_layers, scope_bounds, &mut focusables);
        if focusables.is_empty() {
            return true;
        }

        let next = match self
            .focus
            .and_then(|f| focusables.iter().position(|n| *n == f))
        {
            Some(idx) => {
                if forward {
                    focusables[(idx + 1) % focusables.len()]
                } else {
                    focusables[(idx + focusables.len() - 1) % focusables.len()]
                }
            }
            None => {
                if forward {
                    focusables[0]
                } else {
                    focusables[focusables.len() - 1]
                }
            }
        };

        if self.focus != Some(next) {
            if let Some(prev) = self.focus {
                self.mark_invalidation(prev, Invalidation::Paint);
            }
            self.focus = Some(next);
            self.mark_invalidation(next, Invalidation::Paint);
        }
        if let Some(window) = self.window {
            app.request_redraw(window);
        }
        true
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
        if !Self::rects_intersect(n.bounds, scope_bounds) {
            return;
        }

        if n.widget.as_ref().is_some_and(|w| w.is_focusable()) {
            out.push(node);
        }
        for &child in &n.children {
            self.collect_focusables(child, active_layers, scope_bounds, out);
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

    pub fn layout(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        available: Size,
        scale_factor: f32,
    ) -> Size {
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            available,
        );
        self.layout_in(app, services, root, bounds, scale_factor)
    }

    pub fn layout_in(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Size {
        self.layout_node(app, services, root, bounds, scale_factor)
    }

    pub fn paint(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        root: NodeId,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
    ) {
        self.paint_node(app, services, root, bounds, scene, scale_factor);
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

    fn layout_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        scale_factor: f32,
    ) -> Size {
        if self.debug_enabled {
            self.debug_stats.layout_nodes_visited =
                self.debug_stats.layout_nodes_visited.saturating_add(1);
        }

        let (prev_bounds, measured, invalidated) = match self.nodes.get(node) {
            Some(n) => (n.bounds, n.measured_size, n.invalidation.layout),
            None => return Size::default(),
        };

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        if !invalidated
            && prev_bounds.size == bounds.size
            && prev_bounds.origin != bounds.origin
            && measured != Size::default()
        {
            let delta = Point::new(
                bounds.origin.x - prev_bounds.origin.x,
                bounds.origin.y - prev_bounds.origin.y,
            );
            if delta.x.0 != 0.0 || delta.y.0 != 0.0 {
                if let Some(children) = self.nodes.get(node).map(|n| n.children.clone()) {
                    for child in children {
                        self.translate_subtree_bounds(child, delta);
                    }
                }
            }
            return measured;
        }

        let needs_layout = invalidated || prev_bounds != bounds;
        if !needs_layout {
            return measured;
        }
        if self.debug_enabled {
            self.debug_stats.layout_nodes_performed =
                self.debug_stats.layout_nodes_performed.saturating_add(1);
        }

        let tree_ptr: *mut UiTree<H> = self;
        let app_ptr: *mut H = app;
        let services_ptr: *mut dyn UiServices = services;
        let sf = scale_factor;
        let mut layout_child = move |child: NodeId, bounds: Rect| -> Size {
            unsafe {
                (&mut *tree_ptr).layout_node(&mut *app_ptr, &mut *services_ptr, child, bounds, sf)
            }
        };

        let mut observations: Vec<(ModelId, Invalidation)> = Vec::new();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        let size = self.with_widget_mut(node, |widget, tree| {
            let children: Vec<NodeId> = tree
                .nodes
                .get(node)
                .map(|n| n.children.clone())
                .unwrap_or_default();
            let mut cx = LayoutCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: &children,
                bounds,
                available: bounds.size,
                scale_factor: sf,
                services: unsafe { &mut *services_ptr },
                observe_model: &mut observe_model,
                layout_child: &mut layout_child,
            };
            widget.layout(&mut cx)
        });

        self.observed_in_layout.record(node, observations);
        if let Some(n) = self.nodes.get_mut(node) {
            n.measured_size = size;
            n.invalidation.layout = false;
        }

        size
    }

    fn translate_subtree_bounds(&mut self, node: NodeId, delta: Point) {
        let mut stack = vec![node];
        while let Some(id) = stack.pop() {
            let Some(n) = self.nodes.get_mut(id) else {
                continue;
            };
            n.bounds.origin = Point::new(n.bounds.origin.x + delta.x, n.bounds.origin.y + delta.y);
            for &child in &n.children {
                stack.push(child);
            }
        }
    }

    fn paint_node(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        node: NodeId,
        bounds: Rect,
        scene: &mut Scene,
        scale_factor: f32,
    ) {
        if self.debug_enabled {
            self.debug_stats.paint_nodes = self.debug_stats.paint_nodes.saturating_add(1);
        }

        let tree_ref: *const UiTree<H> = self as *const UiTree<H>;
        let tree_ptr: *mut UiTree<H> = self;
        let app_ptr: *mut H = app;
        let services_ptr: *mut dyn UiServices = services;
        let scene_ptr: *mut Scene = scene;
        let sf = scale_factor;
        let mut paint_child = move |child: NodeId, bounds: Rect| {
            unsafe {
                (&mut *tree_ptr).paint_node(
                    &mut *app_ptr,
                    &mut *services_ptr,
                    child,
                    bounds,
                    &mut *scene_ptr,
                    sf,
                )
            };
        };
        let child_bounds = move |child: NodeId| -> Option<Rect> {
            unsafe { (&*tree_ref).nodes.get(child).map(|n| n.bounds) }
        };

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        let (invalidated, prev_cache) = match self.nodes.get(node) {
            Some(n) => (n.invalidation.paint, n.paint_cache),
            None => return,
        };

        let theme_revision = Theme::global(&*app).revision();
        let key = PaintCacheKey::new(bounds, sf, theme_revision);
        let cache_enabled = self.paint_cache_enabled();

        if cache_enabled && !invalidated {
            if let Some(prev) = prev_cache
                && prev.generation == self.paint_cache.source_generation
                && prev.key == key
            {
                let start = scene.ops_len();
                let range = prev.start as usize..prev.end as usize;
                if range.start <= range.end && range.end <= self.paint_cache.prev_ops.len() {
                    let delta = Point::new(
                        bounds.origin.x - prev.origin.x,
                        bounds.origin.y - prev.origin.y,
                    );
                    scene.replay_ops_translated(&self.paint_cache.prev_ops[range.clone()], delta);
                    let end = scene.ops_len();

                    if let Some(n) = self.nodes.get_mut(node) {
                        n.paint_cache = Some(PaintCacheEntry {
                            generation: self.paint_cache.target_generation,
                            key,
                            origin: bounds.origin,
                            start: start as u32,
                            end: end as u32,
                        });
                        n.invalidation.paint = false;
                    }

                    self.paint_cache.hits = self.paint_cache.hits.saturating_add(1);
                    self.paint_cache.replayed_ops = self
                        .paint_cache
                        .replayed_ops
                        .saturating_add((end - start) as u32);
                    return;
                }
            }
            self.paint_cache.misses = self.paint_cache.misses.saturating_add(1);
        }

        let mut observations: Vec<(ModelId, Invalidation)> = Vec::new();
        let mut observe_model = |model: ModelId, inv: Invalidation| {
            observations.push((model, inv));
        };

        if self.debug_enabled {
            self.debug_stats.paint_nodes_performed =
                self.debug_stats.paint_nodes_performed.saturating_add(1);
        }

        let start = scene.ops_len();
        self.with_widget_mut(node, |widget, tree| {
            let children: Vec<NodeId> = tree
                .nodes
                .get(node)
                .map(|n| n.children.clone())
                .unwrap_or_default();
            let mut cx = PaintCx {
                app,
                node,
                window: tree.window,
                focus: tree.focus,
                children: &children,
                bounds,
                scale_factor: sf,
                services: unsafe { &mut *services_ptr },
                observe_model: &mut observe_model,
                scene,
                paint_child: &mut paint_child,
                child_bounds: &child_bounds,
            };
            widget.paint(&mut cx);
        });
        let end = scene.ops_len();

        self.observed_in_paint.record(node, observations);
        if let Some(n) = self.nodes.get_mut(node) {
            n.invalidation.paint = false;
            if cache_enabled {
                n.paint_cache = Some(PaintCacheEntry {
                    generation: self.paint_cache.target_generation,
                    key,
                    origin: bounds.origin,
                    start: start as u32,
                    end: end as u32,
                });
            }
        }
    }

    fn hit_test(&self, root: NodeId, position: Point) -> Option<NodeId> {
        self.hit_test_node(root, position)
    }

    fn hit_test_layers(&self, layers: &[NodeId], position: Point) -> Option<NodeId> {
        for &root in layers {
            if let Some(hit) = self.hit_test(root, position) {
                return Some(hit);
            }
        }
        None
    }

    fn hit_test_node(&self, node: NodeId, position: Point) -> Option<NodeId> {
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let clips_hit_test = widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true);
        if clips_hit_test {
            if !n.bounds.contains(position) {
                return None;
            }
            if let Some(w) = widget
                && let Some(radii) = w.clip_hit_test_corner_radii(n.bounds)
                && !Self::point_in_rounded_rect(n.bounds, radii, position)
            {
                return None;
            }
        }

        let hit_test_children = n
            .widget
            .as_ref()
            .map(|w| w.hit_test_children(n.bounds, position))
            .unwrap_or(true);
        if hit_test_children {
            for &child in n.children.iter().rev() {
                if let Some(hit) = self.hit_test_node(child, position) {
                    return Some(hit);
                }
            }
        }

        let hit = n.bounds.contains(position)
            && n.widget
                .as_ref()
                .map(|w| w.hit_test(n.bounds, position))
                .unwrap_or(true);
        hit.then_some(node)
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

    fn visible_layers_in_paint_order(&self) -> impl Iterator<Item = UiLayerId> + '_ {
        self.layer_order
            .iter()
            .copied()
            .filter(|id| self.layers.get(*id).is_some_and(|l| l.visible))
    }

    fn active_input_layers(&self) -> (Vec<NodeId>, Option<NodeId>) {
        let visible: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        if visible.is_empty() {
            return (Vec::new(), None);
        }

        let mut barrier_index: Option<usize> = None;
        for (idx, layer) in visible.iter().enumerate() {
            if self.layers[*layer].blocks_underlay_input {
                barrier_index = Some(idx);
            }
        }

        let range_start = barrier_index.unwrap_or(0);
        let mut roots: Vec<NodeId> = Vec::new();
        for layer in visible[range_start..].iter().rev() {
            let l = &self.layers[*layer];
            if l.hit_testable {
                roots.push(l.root);
            }
        }

        let barrier_root = barrier_index.map(|idx| self.layers[visible[idx]].root);
        (roots, barrier_root)
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
            let mut stack: Vec<NodeId> = vec![root];
            while let Some(id) = stack.pop() {
                let Some(node) = self.nodes.get_mut(id) else {
                    continue;
                };
                let parent = node.parent;
                let bounds = node.bounds;
                let children = node.children.as_slice();
                let is_text_input = node.widget.as_ref().is_some_and(|w| w.is_text_input());

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

                // Preserve a stable-ish order: visit children in declared order.
                for &child in children.iter().rev() {
                    stack.push(child);
                }

                // Allow widgets to override semantics metadata.
                if let Some(widget) = node.widget.as_mut() {
                    let mut cx = SemanticsCx {
                        app,
                        node: id,
                        window: Some(window),
                        bounds,
                        children,
                        focus,
                        captured,
                        role: &mut role,
                        flags: &mut flags,
                    };
                    widget.semantics(&mut cx);
                }

                nodes.push(SemanticsNode {
                    id,
                    parent,
                    role,
                    bounds,
                    flags,
                });
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
    }

    fn node_in_any_layer(&self, node: NodeId, layer_roots: &[NodeId]) -> bool {
        let Some(node_root) = self.node_root(node) else {
            return false;
        };
        layer_roots.contains(&node_root)
    }

    fn node_layer(&self, node: NodeId) -> Option<UiLayerId> {
        let root = self.node_root(node)?;
        self.root_to_layer.get(&root).copied()
    }

    fn node_root(&self, mut node: NodeId) -> Option<NodeId> {
        while let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) {
            node = parent;
        }
        self.nodes.contains_key(node).then_some(node)
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

fn event_position(event: &Event) -> Option<Point> {
    match event {
        Event::Pointer(pe) => Some(pointer_position(pe)),
        Event::ExternalDrag(e) => Some(e.position),
        Event::InternalDrag(e) => Some(e.position),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        Color, Corners, DrawOrder, Edges, Px, Scene, SceneOp, TextConstraints, TextMetrics,
        TextService, TextStyle, TextWrap,
    };
    use fret_runtime::Model;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Default)]
    struct FakeUiServices;

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                    baseline: fret_core::Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    struct ObservingWidget {
        model: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for ObservingWidget {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.observe_model(self.model, Invalidation::Layout);
            let _ = cx.services.text().prepare(
                "x",
                TextStyle {
                    font: fret_core::FontId::default(),
                    size: fret_core::Px(12.0),
                    ..Default::default()
                },
                TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                },
            );
            Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.observe_model(self.model, Invalidation::Paint);
            let _ = cx.scene;
        }
    }

    struct RoundedClipWidget;

    impl<H: UiHost> Widget<H> for RoundedClipWidget {
        fn clips_hit_test(&self, _bounds: Rect) -> bool {
            true
        }

        fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
            Some(Corners::all(Px(20.0)))
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    #[test]
    fn model_change_invalidates_observers() {
        let mut app = crate::test_host::TestHost::new();
        let model = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());
        ui.set_paint_cache_enabled(true);

        let node = ui.create_node(ObservingWidget { model });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        if let Some(n) = ui.nodes.get_mut(node) {
            n.invalidation.clear();
        }

        let _ = model.update(&mut app, |v, _cx| {
            *v += 1;
        });
        let changed = app.take_changed_models();
        assert!(changed.contains(&model.id()));

        ui.propagate_model_changes(&mut app, &changed);
        let n = ui.nodes.get(node).unwrap();
        assert!(n.invalidation.layout);
        assert!(n.invalidation.paint);
    }

    #[test]
    fn hit_test_respects_rounded_overflow_clip() {
        let mut app = crate::test_host::TestHost::new();
        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let node = ui.create_node(RoundedClipWidget);
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );

        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Inside bounds, but outside the rounded corner arc.
        assert_eq!(ui.hit_test(node, Point::new(Px(1.0), Px(1.0))), None);

        // Inside the rounded rectangle.
        assert_eq!(ui.hit_test(node, Point::new(Px(25.0), Px(25.0))), Some(node));
    }

    struct CountingPaintWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for CountingPaintWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: Color::TRANSPARENT,
                border: Edges::default(),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::default(),
            });
        }
    }

    #[test]
    fn paint_cache_replays_subtree_ops_when_clean() {
        let mut app = crate::test_host::TestHost::new();

        let paints = Arc::new(AtomicUsize::new(0));
        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());
        ui.set_paint_cache_enabled(true);

        let node = ui.create_node(CountingPaintWidget {
            paints: paints.clone(),
        });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 1);
        assert_eq!(scene.ops_len(), 1);

        ui.ingest_paint_cache_source(&mut scene);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 1);
        assert_eq!(scene.ops_len(), 1);

        ui.invalidate(node, Invalidation::Paint);

        ui.ingest_paint_cache_source(&mut scene);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 2);
        assert_eq!(scene.ops_len(), 1);

        let bounds2 = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(200.0), fret_core::Px(100.0)),
        );
        ui.ingest_paint_cache_source(&mut scene);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds2, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 3);
        assert_eq!(scene.ops_len(), 1);
    }

    struct TransparentOverlay;

    impl<H: UiHost> Widget<H> for TransparentOverlay {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }
    }

    struct ClickCounter {
        clicks: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for ClickCounter {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(
                event,
                Event::Pointer(fret_core::PointerEvent::Up {
                    button: fret_core::MouseButton::Left,
                    ..
                })
            ) {
                let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    #[test]
    fn hit_test_can_make_overlay_pointer_transparent() {
        let window = AppWindowId::default();

        let mut app = crate::test_host::TestHost::new();
        let clicks = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(ClickCounter { clicks });
        ui.set_root(base);

        let overlay = ui.create_node(TransparentOverlay);
        let _ = ui.push_overlay_root_ex(overlay, false, true);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let value = app.models().get(clicks).copied().unwrap_or(0);
        assert_eq!(value, 1);
    }

    #[test]
    fn paint_cache_replays_ops_when_node_translates() {
        let mut app = crate::test_host::TestHost::new();

        let paints = Arc::new(AtomicUsize::new(0));
        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());
        ui.set_paint_cache_enabled(true);

        let node = ui.create_node(CountingPaintWidget {
            paints: paints.clone(),
        });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let mut scene = Scene::default();

        let bounds_a = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
        );
        ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 1);
        assert_eq!(scene.ops_len(), 1);

        ui.ingest_paint_cache_source(&mut scene);
        scene.clear();

        let bounds_b = Rect::new(
            Point::new(fret_core::Px(20.0), fret_core::Px(15.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
        );
        ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);
        assert_eq!(paints.load(Ordering::SeqCst), 1);
        assert_eq!(scene.ops_len(), 1);

        match scene.ops()[0] {
            SceneOp::Quad { rect, .. } => assert_eq!(rect, bounds_b),
            _ => panic!("expected quad op"),
        }
    }

    #[test]
    fn semantics_snapshot_includes_visible_roots_and_barrier() {
        let mut app = crate::test_host::TestHost::new();

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let base = ui.create_node(crate::primitives::Stack::new());
        ui.set_root(base);
        let base_child = ui.create_node(crate::primitives::Stack::new());
        ui.add_child(base, base_child);

        let overlay_root = ui.create_node(crate::primitives::Stack::new());
        ui.push_overlay_root(overlay_root, true);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert_eq!(snap.roots.len(), 2);
        assert_eq!(snap.barrier_root, Some(overlay_root));
        assert_eq!(
            snap.nodes.iter().find(|n| n.id == base).unwrap().role,
            SemanticsRole::Window
        );
        assert_ne!(
            snap.nodes
                .iter()
                .find(|n| n.id == overlay_root)
                .unwrap()
                .role,
            SemanticsRole::Window
        );
        assert!(snap.nodes.iter().any(|n| n.id == base));
        assert!(snap.nodes.iter().any(|n| n.id == base_child));
        assert!(snap.nodes.iter().any(|n| n.id == overlay_root));
    }

    #[test]
    fn event_cx_bounds_tracks_translated_nodes() {
        struct BoundsProbe {
            out: Model<Point>,
        }

        impl BoundsProbe {
            fn new(out: Model<Point>) -> Self {
                Self { out }
            }
        }

        impl<H: UiHost> Widget<H> for BoundsProbe {
            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if !matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                    return;
                }
                let origin = cx.bounds.origin;
                let _ = cx.app.models_mut().update(self.out, |v| *v = origin);
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let out = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let root = ui.create_node(crate::primitives::Stack::new());
        let probe = ui.create_node(BoundsProbe::new(out));
        ui.add_child(root, probe);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let size = Size::new(Px(120.0), Px(40.0));

        ui.layout_in(
            &mut app,
            &mut services,
            root,
            Rect::new(Point::new(Px(0.0), Px(0.0)), size),
            1.0,
        );

        // Layout again with the same size but translated origin: the tree uses a fast-path that
        // translates node bounds without re-running widget.layout for the subtree.
        ui.layout_in(
            &mut app,
            &mut services,
            root,
            Rect::new(Point::new(Px(0.0), Px(100.0)), size),
            1.0,
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(10.0), Px(110.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let origin = app.models().get(out).copied().unwrap_or_default();
        assert_eq!(origin, Point::new(Px(0.0), Px(100.0)));
    }
}
