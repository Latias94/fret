use crate::{
    Theme, UiHost, declarative,
    elements::GlobalElementId,
    widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::PlatformCapabilities;
use fret_core::{
    AppWindowId, Corners, Event, FrameId, KeyCode, NodeId, Point, PointerEvent, Px, Rect, Scene,
    SceneOp, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size, Transform2D,
    UiServices,
};
use fret_runtime::{
    CommandId, Effect, InputContext, InputDispatchPhase, KeyChord, KeymapService, ModelId, Platform,
};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
use shortcuts::{KeydownShortcutParams, PendingShortcut, PointerDownOutsideParams};

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
    ime_allowed: bool,
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
            ime_allowed: false,
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

    fn dispatch_event_to_node_chain(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
    ) -> bool {
        let services_ptr: *mut dyn UiServices = services;

        let (active_roots, _barrier_root) = self.active_input_layers();
        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (invalidations, requested_focus, requested_capture, stop_propagation) = self
                    .with_widget_mut(node_id, |widget, tree| {
                        let (children, bounds) = tree
                            .nodes
                            .get(node_id)
                            .map(|n| (n.children.as_slice(), n.bounds))
                            .unwrap_or((&[][..], Rect::default()));
                        let mut cx = EventCx {
                            app,
                            services: unsafe { &mut *services_ptr },
                            node: node_id,
                            window: tree.window,
                            input_ctx: input_ctx.clone(),
                            children,
                            focus: tree.focus,
                            captured: tree.captured,
                            bounds,
                            invalidations: Vec::new(),
                            requested_focus: None,
                            requested_capture: None,
                            requested_cursor: None,
                            stop_propagation: false,
                        };
                        widget.event(&mut cx, &event_for_node);
                        (
                            cx.invalidations,
                            cx.requested_focus,
                            cx.requested_capture,
                            cx.stop_propagation,
                        )
                    });

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus != Some(focus)
                    && self.node_in_any_layer(focus, &active_roots)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture
                    && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
                {
                    self.captured = capture;
                }

                if self.captured.is_some() || stop_propagation {
                    return true;
                }
            }
            return false;
        }

        let mut node_id = start;
        loop {
            let (invalidations, requested_focus, requested_capture, stop_propagation, parent) =
                self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
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

            if let Some(focus) = requested_focus
                && self.focus != Some(focus)
                && self.node_in_any_layer(focus, &active_roots)
            {
                if let Some(prev) = self.focus {
                    self.mark_invalidation(prev, Invalidation::Paint);
                }
                self.focus = Some(focus);
                self.mark_invalidation(focus, Invalidation::Paint);
            }

            if let Some(capture) = requested_capture
                && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
            {
                self.captured = capture;
            }

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
        let Some(old_children) = self.nodes.get(parent).map(|n| n.children.clone()) else {
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
            let focusable = if let Some(record) =
                crate::declarative::element_record_for_node(app, window, id)
            {
                match record.instance {
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

            if let Some(node) = self.nodes.get(id) {
                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        None
    }

    pub fn dispatch_event(&mut self, app: &mut H, services: &mut dyn UiServices, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        self.enforce_modal_barrier_scope(&active_layers);

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

        let focus_is_text_input = self.focus_is_text_input();
        self.set_ime_allowed(app, focus_is_text_input);

        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            focus_is_text_input,
            dispatch_phase: InputDispatchPhase::Normal,
        };

        // ADR 0012: when a text input is focused, reserve common IME/navigation keys for the
        // text/IME path first, and only fall back to shortcut matching if the widget doesn't
        // consume the event.
        let defer_keydown_shortcuts_until_after_dispatch = !self.replaying_pending_shortcut
            && self.focus.is_some()
            && match event {
                Event::KeyDown { key, modifiers, .. } => {
                    Self::should_defer_keydown_shortcut_matching_to_text_input(
                        *key,
                        *modifiers,
                        focus_is_text_input,
                    )
                }
                _ => false,
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
        let mut stop_propagation_requested = false;

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && !defer_keydown_shortcuts_until_after_dispatch
            && self.handle_keydown_shortcuts(
                app,
                services,
                KeydownShortcutParams {
                    input_ctx: &input_ctx,
                    barrier_root,
                    focus_is_text_input,
                    key: *key,
                    modifiers: *modifiers,
                    repeat: *repeat,
                },
            )
        {
            return;
        }

        let default_root = barrier_root.unwrap_or(base_root);

        // Pointer capture only affects pointer events. Drag-and-drop style events
        // (external/internal) must continue to follow the cursor for correct cross-window UX.
        let captured = match event {
            Event::Pointer(_) => self.captured,
            _ => None,
        };

        // Internal drag overrides may need to route events to a stable "anchor" node, even if
        // hit-testing fails or the cursor is over an unrelated widget (e.g. docking tear-off).
        let internal_drag_target = (|| {
            if !matches!(event, Event::InternalDrag(_)) {
                return None;
            }
            let window = self.window?;
            let drag = app.drag()?;
            if !drag.cross_window_hover {
                return None;
            }
            let routes = app.global::<crate::drag_route::InternalDragRouteService>()?;
            let target = routes.route(window, drag.kind)?;
            self.node_in_any_layer(target, &active_layers)
                .then_some(target)
        })();

        if let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
            && let Some(pos) = event_position(event)
        {
            let hit = self.hit_test_layers(&active_layers, pos);

            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) && captured.is_none() {
                self.dispatch_pointer_down_outside(
                    app,
                    services,
                    PointerDownOutsideParams {
                        input_ctx: &input_ctx,
                        active_layer_roots: &active_layers,
                        base_root,
                        hit,
                        event,
                    },
                );
            }
            let hovered_pressable: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let window_frame = window_frame?;
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

            let (prev_element, prev_node, next_element, next_node) =
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

            if let Some(element) = prev_element
                && prev_node.is_some()
            {
                let hook = crate::elements::with_element_state(
                    app,
                    window,
                    element,
                    crate::action::PressableHoverActionHooks::default,
                    |hooks| hooks.on_hover_change.clone(),
                );

                if let Some(h) = hook {
                    let mut host = crate::action::UiActionHostAdapter { app };
                    h(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: element,
                        },
                        false,
                    );
                }
            }

            if let Some(element) = next_element
                && next_node.is_some()
            {
                let hook = crate::elements::with_element_state(
                    app,
                    window,
                    element,
                    crate::action::PressableHoverActionHooks::default,
                    |hooks| hooks.on_hover_change.clone(),
                );

                if let Some(h) = hook {
                    let mut host = crate::action::UiActionHostAdapter { app };
                    h(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: element,
                        },
                        true,
                    );
                }
            }

            let hovered_hover_region: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let window_frame = window_frame?;
                    let mut node = hit;
                    while let Some(id) = node {
                        if let Some(record) = window_frame.instances.get(&id)
                            && matches!(
                                record.instance,
                                declarative::ElementInstance::HoverRegion(_)
                            )
                        {
                            return Some(record.element);
                        }
                        node = self.nodes.get(id).and_then(|n| n.parent);
                    }
                    None
                });

            let (_prev_element, prev_node, _next_element, next_node) =
                crate::elements::update_hovered_hover_region(app, window, hovered_hover_region);
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
        } else if let Some(target) = internal_drag_target {
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

        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(node_id, event);
            for (node_id, event_for_node) in chain {
                let (
                    invalidations,
                    requested_focus,
                    requested_capture,
                    requested_cursor,
                    stop_propagation,
                ) = self.with_widget_mut(node_id, |widget, tree| {
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, &event_for_node);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.requested_cursor,
                        cx.stop_propagation,
                    )
                });

                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus != Some(focus)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture {
                    self.captured = capture;
                }

                if requested_cursor.is_some() && cursor_choice.is_none() {
                    cursor_choice = requested_cursor;
                }

                if stop_propagation {
                    stop_propagation_requested = true;
                }

                if self.captured.is_some() || stop_propagation {
                    break;
                }
            }
        } else {
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
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
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

                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus != Some(focus)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture {
                    self.captured = capture;
                };

                if requested_cursor.is_some() && cursor_choice.is_none() {
                    cursor_choice = requested_cursor;
                }

                if stop_propagation {
                    stop_propagation_requested = true;
                }

                if self.captured.is_some() || stop_propagation {
                    break;
                }

                node_id = match parent {
                    Some(parent) => parent,
                    None => break,
                };
            }
        }

        if defer_keydown_shortcuts_until_after_dispatch
            && !stop_propagation_requested
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
        {
            let focus_is_text_input = self.focus_is_text_input();
            let input_ctx_for_shortcuts = InputContext {
                focus_is_text_input,
                ..input_ctx.clone()
            };

            if self.handle_keydown_shortcuts(
                app,
                services,
                KeydownShortcutParams {
                    input_ctx: &input_ctx_for_shortcuts,
                    barrier_root,
                    focus_is_text_input,
                    key: *key,
                    modifiers: *modifiers,
                    repeat: *repeat,
                },
            ) {
                if needs_redraw && let Some(window) = self.window {
                    app.request_redraw(window);
                }
                return;
            }
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

        // Keep IME enable/disable tightly coupled to focus changes caused by the event itself.
        let focus_is_text_input = self.focus_is_text_input();
        self.set_ime_allowed(app, focus_is_text_input);
    }

    fn dispatch_pointer_down_outside(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        params: PointerDownOutsideParams<'_>,
    ) {
        let hit_root = params.hit.and_then(|n| self.node_root(n));

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

            if !layer.wants_pointer_down_outside_events {
                continue;
            }

            self.dispatch_event_to_node_chain_observer(
                app,
                services,
                params.input_ctx,
                layer.root,
                params.event,
            );
            break;
        }
    }

    fn dispatch_event_to_node_chain_observer(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
    ) {
        let services_ptr: *mut dyn UiServices = services;

        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (invalidations, _parent) = self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut observer_ctx = input_ctx.clone();
                    observer_ctx.dispatch_phase = InputDispatchPhase::Observer;
                    let mut cx = EventCx {
                        app,
                        services: unsafe { &mut *services_ptr },
                        node: node_id,
                        window: tree.window,
                        input_ctx: observer_ctx,
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, &event_for_node);

                    // Observer dispatch must not mutate routing state (capture/focus/propagation). It
                    // exists to allow click-through outside-press policies, not to intercept input.
                    (cx.invalidations, parent)
                });

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }
            }
            return;
        }

        let mut node_id = start;
        loop {
            let (invalidations, parent) = self.with_widget_mut(node_id, |widget, tree| {
                let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                let (children, bounds) = tree
                    .nodes
                    .get(node_id)
                    .map(|n| (n.children.as_slice(), n.bounds))
                    .unwrap_or((&[][..], Rect::default()));
                let mut observer_ctx = input_ctx.clone();
                observer_ctx.dispatch_phase = InputDispatchPhase::Observer;
                let mut cx = EventCx {
                    app,
                    services: unsafe { &mut *services_ptr },
                    node: node_id,
                    window: tree.window,
                    input_ctx: observer_ctx,
                    children,
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

                // Observer dispatch must not mutate routing state (capture/focus/propagation). It
                // exists to allow click-through outside-press policies, not to intercept input.
                (cx.invalidations, parent)
            });

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
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
            dispatch_phase: InputDispatchPhase::Normal,
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

            if let Some(focus) = requested_focus
                && self.focus != Some(focus)
            {
                if let Some(prev) = self.focus {
                    self.mark_invalidation(prev, Invalidation::Paint);
                }
                self.focus = Some(focus);
                self.mark_invalidation(focus, Invalidation::Paint);
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

        let _ = base_root;
        self.focus_traverse_in_roots(app, active_layers, forward, barrier_root)
    }

    /// Focus traversal mechanism used by both the runtime default and component-owned focus scopes.
    ///
    /// Notes:
    /// - `roots` are treated as candidates; only focusables that are in the current active input layers
    ///   (modal-aware) and intersect the modal scope bounds are included.
    /// - This is intentionally conservative until we formalize a scroll-into-view contract (ADR 0068).
    pub fn focus_traverse_in_roots(
        &mut self,
        app: &mut H,
        roots: &[NodeId],
        forward: bool,
        scope_root: Option<NodeId>,
    ) -> bool {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return true;
        };
        let (active_layers, barrier_root) = self.active_input_layers();

        let scope_root = scope_root.or(barrier_root).unwrap_or(base_root);
        let scope_bounds = self
            .nodes
            .get(scope_root)
            .map(|n| n.bounds)
            .unwrap_or_default();

        let mut focusables: Vec<NodeId> = Vec::new();
        for &root in roots {
            self.collect_focusables(root, &active_layers, scope_bounds, &mut focusables);
        }
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
        let position = if let Some(w) = widget
            && let Some(t) = w.render_transform(n.bounds)
            && let Some(inv) = t.inverse()
        {
            inv.apply_point(position)
        } else {
            position
        };
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

    fn node_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.render_transform(n.bounds)?;
        t.inverse().is_some().then_some(t)
    }

    fn apply_vector(t: Transform2D, v: Point) -> Point {
        Point::new(Px(t.a * v.x.0 + t.c * v.y.0), Px(t.b * v.x.0 + t.d * v.y.0))
    }

    fn event_with_mapped_position(event: &Event, position: Point, delta: Option<Point>) -> Event {
        match event {
            Event::Pointer(e) => {
                let e = match e {
                    PointerEvent::Move {
                        buttons, modifiers, ..
                    } => PointerEvent::Move {
                        position,
                        buttons: *buttons,
                        modifiers: *modifiers,
                    },
                    PointerEvent::Down {
                        button, modifiers, ..
                    } => PointerEvent::Down {
                        position,
                        button: *button,
                        modifiers: *modifiers,
                    },
                    PointerEvent::Up {
                        button, modifiers, ..
                    } => PointerEvent::Up {
                        position,
                        button: *button,
                        modifiers: *modifiers,
                    },
                    PointerEvent::Wheel { modifiers, .. } => PointerEvent::Wheel {
                        position,
                        delta: delta.unwrap_or(Point::new(Px(0.0), Px(0.0))),
                        modifiers: *modifiers,
                    },
                };
                Event::Pointer(e)
            }
            Event::ExternalDrag(e) => Event::ExternalDrag(fret_core::ExternalDragEvent {
                position,
                kind: e.kind.clone(),
            }),
            Event::InternalDrag(e) => Event::InternalDrag(fret_core::InternalDragEvent {
                position,
                kind: e.kind.clone(),
                modifiers: e.modifiers,
            }),
            _ => event.clone(),
        }
    }

    fn build_mapped_event_chain(&self, start: NodeId, event: &Event) -> Vec<(NodeId, Event)> {
        let Some(pos) = event_position(event) else {
            return vec![(start, event.clone())];
        };

        let mut chain: Vec<NodeId> = Vec::new();
        let mut cur = Some(start);
        while let Some(id) = cur {
            chain.push(id);
            cur = self.nodes.get(id).and_then(|n| n.parent);
        }

        let mut nodes_root_to_leaf = chain.clone();
        nodes_root_to_leaf.reverse();

        let mut mapped_pos = pos;
        let mut mapped_delta = match event {
            Event::Pointer(PointerEvent::Wheel { delta, .. }) => Some(*delta),
            _ => None,
        };

        let mut out: Vec<(NodeId, Event)> = Vec::with_capacity(chain.len());
        for &node in &nodes_root_to_leaf {
            if let Some(t) = self.node_render_transform(node)
                && let Some(inv) = t.inverse()
            {
                mapped_pos = inv.apply_point(mapped_pos);
                if let Some(d) = mapped_delta {
                    mapped_delta = Some(Self::apply_vector(inv, d));
                }
            }
            out.push((
                node,
                Self::event_with_mapped_position(event, mapped_pos, mapped_delta),
            ));
        }

        out.reverse();
        out
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
                let mut actions = fret_core::SemanticsActions {
                    focus: is_focusable || is_text_input,
                    invoke: false,
                    set_value: is_text_input,
                    set_text_selection: is_text_input,
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
                        label: &mut label,
                        value: &mut value,
                        text_selection: &mut text_selection,
                        text_composition: &mut text_composition,
                        actions: &mut actions,
                        active_descendant: &mut active_descendant,
                        pos_in_set: &mut pos_in_set,
                        set_size: &mut set_size,
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
mod tests {
    use super::*;
    use fret_core::{
        Color, Corners, DrawOrder, Edges, Px, Scene, SceneOp, TextConstraints, TextMetrics,
        TextService, TextStyle, TextWrap,
    };
    use fret_runtime::{BindingV1, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Model};
    use slotmap::KeyData;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Default)]
    struct TestStack;

    impl<H: UiHost> Widget<H> for TestStack {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                } else {
                    cx.paint(child, cx.bounds);
                }
            }
        }
    }

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

    struct PaintObservingWidget {
        model: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for PaintObservingWidget {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.observe_model(self.model, Invalidation::Paint);
        }
    }

    struct HitTestObservingWidget {
        model: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for HitTestObservingWidget {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.observe_model(self.model, Invalidation::HitTest);
        }
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
    fn model_change_invalidates_observers_across_windows() {
        let mut app = crate::test_host::TestHost::new();
        let model = app.models_mut().insert(0u32);

        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );

        let mut ui_a = UiTree::new();
        ui_a.set_window(window_a);
        let node_a = ui_a.create_node(ObservingWidget { model });
        ui_a.set_root(node_a);
        ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene_a = Scene::default();
        ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
        ui_a.nodes.get_mut(node_a).unwrap().invalidation.clear();

        let mut ui_b = UiTree::new();
        ui_b.set_window(window_b);
        let node_b = ui_b.create_node(ObservingWidget { model });
        ui_b.set_root(node_b);
        ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene_b = Scene::default();
        ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
        ui_b.nodes.get_mut(node_b).unwrap().invalidation.clear();

        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        assert!(changed.contains(&model.id()));

        ui_a.propagate_model_changes(&mut app, &changed);
        ui_b.propagate_model_changes(&mut app, &changed);

        let na = ui_a.nodes.get(node_a).unwrap();
        assert!(na.invalidation.layout);
        assert!(na.invalidation.paint);

        let nb = ui_b.nodes.get(node_b).unwrap();
        assert!(nb.invalidation.layout);
        assert!(nb.invalidation.paint);
    }

    #[test]
    fn paint_observation_only_invalidates_paint() {
        let mut app = crate::test_host::TestHost::new();
        let model = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let node = ui.create_node(PaintObservingWidget { model });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        ui.nodes.get_mut(node).unwrap().invalidation.clear();

        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        assert!(ui.propagate_model_changes(&mut app, &changed));

        let n = ui.nodes.get(node).unwrap();
        assert!(!n.invalidation.layout);
        assert!(n.invalidation.paint);
        assert!(!n.invalidation.hit_test);
    }

    #[test]
    fn hit_test_observation_escalates_to_layout_and_paint() {
        let mut app = crate::test_host::TestHost::new();
        let model = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let node = ui.create_node(HitTestObservingWidget { model });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        ui.nodes.get_mut(node).unwrap().invalidation.clear();

        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();
        assert!(ui.propagate_model_changes(&mut app, &changed));

        let n = ui.nodes.get(node).unwrap();
        assert!(n.invalidation.hit_test);
        assert!(n.invalidation.layout);
        assert!(n.invalidation.paint);
    }

    #[test]
    fn model_change_requests_redraw_for_each_invalidated_window() {
        let mut app = crate::test_host::TestHost::new();
        let model = app.models_mut().insert(0u32);

        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );

        let mut ui_a = UiTree::new();
        ui_a.set_window(window_a);
        let node_a = ui_a.create_node(PaintObservingWidget { model });
        ui_a.set_root(node_a);
        ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene_a = Scene::default();
        ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
        ui_a.nodes.get_mut(node_a).unwrap().invalidation.clear();

        let mut ui_b = UiTree::new();
        ui_b.set_window(window_b);
        let node_b = ui_b.create_node(PaintObservingWidget { model });
        ui_b.set_root(node_b);
        ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene_b = Scene::default();
        ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
        ui_b.nodes.get_mut(node_b).unwrap().invalidation.clear();

        let _ = model.update(&mut app, |v, _cx| *v += 1);
        let changed = app.take_changed_models();

        ui_a.propagate_model_changes(&mut app, &changed);
        ui_b.propagate_model_changes(&mut app, &changed);

        let effects = app.flush_effects();
        let redraws: std::collections::HashSet<AppWindowId> = effects
            .into_iter()
            .filter_map(|e| match e {
                Effect::Redraw(w) => Some(w),
                _ => None,
            })
            .collect();
        let expected: std::collections::HashSet<AppWindowId> =
            [window_a, window_b].into_iter().collect();

        assert_eq!(redraws, expected);
    }

    #[test]
    fn paint_all_sets_ime_allowed_for_focused_text_input() {
        #[derive(Default)]
        struct FakeTextInput;

        impl<H: UiHost> Widget<H> for FakeTextInput {
            fn is_text_input(&self) -> bool {
                true
            }

            fn is_focusable(&self) -> bool {
                true
            }
        }

        let mut app = crate::test_host::TestHost::new();
        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let node = ui.create_node(FakeTextInput);
        ui.set_root(node);
        ui.set_focus(Some(node));

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, fret_runtime::Effect::ImeAllow { enabled: true, .. }))
        );
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
        assert_eq!(
            ui.hit_test(node, Point::new(Px(25.0), Px(25.0))),
            Some(node)
        );
    }

    #[test]
    fn hit_test_respects_rounded_overflow_clip_under_render_transform() {
        struct RoundedClipTranslatedWidget {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for RoundedClipTranslatedWidget {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::translation(self.delta))
            }

            fn clips_hit_test(&self, _bounds: Rect) -> bool {
                true
            }

            fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
                Some(Corners::all(Px(20.0)))
            }
        }

        let mut app = crate::test_host::TestHost::new();
        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let node = ui.create_node(RoundedClipTranslatedWidget {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        ui.set_root(node);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Inside the visual bounds, but outside the rounded corner arc (after inverse mapping).
        assert_eq!(ui.hit_test(node, Point::new(Px(41.0), Px(1.0))), None);

        // Inside the rounded rectangle (after inverse mapping).
        assert_eq!(
            ui.hit_test(node, Point::new(Px(65.0), Px(25.0))),
            Some(node)
        );
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
    fn layer_hit_testable_flag_can_make_overlay_pointer_transparent() {
        let window = AppWindowId::default();

        let mut app = crate::test_host::TestHost::new();
        let clicks = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(ClickCounter { clicks });
        ui.set_root(base);

        let overlay = ui.create_node(ClickCounter { clicks });
        let layer = ui.push_overlay_root_ex(overlay, false, true);
        ui.set_layer_hit_testable(layer, false);

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
    fn overlay_render_transform_affects_hit_testing_and_event_coordinates() {
        struct TransformOverlayRoot {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for TransformOverlayRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::translation(self.delta))
            }

            fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
                false
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                cx.paint(child, child_bounds);
            }
        }

        struct RecordOverlayClicks {
            clicks: Model<u32>,
            last_pos: Model<Point>,
        }

        impl<H: UiHost> Widget<H> for RecordOverlayClicks {
            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                match event {
                    Event::Pointer(PointerEvent::Down { position, .. }) => {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(self.last_pos, |p| *p = *position);
                        cx.stop_propagation();
                    }
                    Event::Pointer(PointerEvent::Up { .. }) => {
                        let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
        }

        let window = AppWindowId::default();
        let mut app = crate::test_host::TestHost::new();
        let underlay_clicks = app.models_mut().insert(0u32);
        let overlay_clicks = app.models_mut().insert(0u32);
        let overlay_last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

        let mut ui = UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(ClickCounter {
            clicks: underlay_clicks,
        });
        ui.set_root(base);

        let overlay_root = ui.create_node(TransformOverlayRoot {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        let overlay_leaf = ui.create_node(RecordOverlayClicks {
            clicks: overlay_clicks,
            last_pos: overlay_last_pos,
        });
        ui.add_child(overlay_root, overlay_leaf);
        let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click inside the overlay leaf (after overlay transform).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(45.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(45.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(overlay_clicks).copied(), Some(1));
        assert_eq!(
            app.models().get(overlay_last_pos).copied(),
            Some(Point::new(Px(5.0), Px(5.0)))
        );
        assert_eq!(
            app.models().get(underlay_clicks).copied(),
            Some(0),
            "expected underlay to not receive clicks when overlay leaf handles them"
        );

        // Click outside the overlay leaf should reach the underlay.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(5.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(5.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(underlay_clicks).copied(), Some(1));
    }

    #[test]
    fn render_transform_affects_hit_testing_and_pointer_event_coordinates() {
        struct TransformRoot {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for TransformRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::translation(self.delta))
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                cx.paint(child, child_bounds);
            }
        }

        struct RecordPointerPos {
            clicks: Model<u32>,
            last_pos: Model<Point>,
        }

        impl<H: UiHost> Widget<H> for RecordPointerPos {
            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                match event {
                    Event::Pointer(PointerEvent::Down { position, .. }) => {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(self.last_pos, |p| *p = *position);
                        cx.stop_propagation();
                    }
                    Event::Pointer(PointerEvent::Up { .. }) => {
                        let _ = cx.app.models_mut().update(self.clicks, |v| *v += 1);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let window = AppWindowId::default();
        let mut app = crate::test_host::TestHost::new();
        let clicks = app.models_mut().insert(0u32);
        let last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TransformRoot {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        let child = ui.create_node(RecordPointerPos { clicks, last_pos });
        ui.add_child(root, child);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(45.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(45.0), Px(5.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get(clicks).copied(), Some(1));
        assert_eq!(
            app.models().get(last_pos).copied(),
            Some(Point::new(Px(5.0), Px(5.0)))
        );
    }

    #[test]
    fn nested_render_transforms_compose_for_pointer_event_coordinates() {
        struct TranslateRoot {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for TranslateRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::translation(self.delta))
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, cx.bounds.size);
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, cx.bounds.size);
                cx.paint(child, child_bounds);
            }
        }

        struct ScaleRoot {
            scale: f32,
        }

        impl<H: UiHost> Widget<H> for ScaleRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::scale_uniform(self.scale))
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                cx.paint(child, child_bounds);
            }
        }

        struct RecordPointerPos {
            last_pos: Model<Point>,
        }

        impl<H: UiHost> Widget<H> for RecordPointerPos {
            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if let Event::Pointer(PointerEvent::Down { position, .. }) = event {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(self.last_pos, |p| *p = *position);
                    cx.stop_propagation();
                }
            }
        }

        let window = AppWindowId::default();
        let mut app = crate::test_host::TestHost::new();
        let last_pos = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TranslateRoot {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        let scale = ui.create_node(ScaleRoot { scale: 2.0 });
        let leaf = ui.create_node(RecordPointerPos { last_pos });
        ui.add_child(root, scale);
        ui.add_child(scale, leaf);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Leaf local (5,5) -> Scale(2x) -> (10,10) -> Translate(+40,0) -> (50,10).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(50.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(
            app.models().get(last_pos).copied(),
            Some(Point::new(Px(5.0), Px(5.0)))
        );
    }

    #[test]
    fn visual_bounds_for_element_includes_ancestor_render_transform() {
        struct TransformRoot {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for TransformRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                Some(Transform2D::translation(self.delta))
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                cx.paint(child, child_bounds);
            }
        }

        struct ElementLeaf;

        impl<H: UiHost> Widget<H> for ElementLeaf {
            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let window = AppWindowId::default();
        let mut app = crate::test_host::TestHost::new();
        let mut ui = UiTree::new();
        ui.set_window(window);

        let element = crate::elements::GlobalElementId(123);

        let root = ui.create_node(TransformRoot {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        let leaf = ui.create_node_for_element(element, ElementLeaf);
        ui.add_child(root, leaf);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        // `visual_bounds_for_element` is defined as a cross-frame query: the "last frame" value is
        // made visible after `prepare_window_for_frame` advances the window element state.
        app.advance_frame();

        let visual = crate::elements::visual_bounds_for_element(&mut app, window, element)
            .expect("expected visual bounds to be recorded during paint");
        assert_eq!(visual.origin, Point::new(Px(40.0), Px(0.0)));
        assert_eq!(visual.size, Size::new(Px(10.0), Px(10.0)));
    }

    #[test]
    fn non_invertible_render_transform_is_ignored_for_paint_and_visual_bounds() {
        struct NonInvertibleRoot {
            delta: Point,
        }

        impl<H: UiHost> Widget<H> for NonInvertibleRoot {
            fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
                let t = Transform2D::translation(self.delta);
                // A singular scale makes the transform non-invertible; ADR 0083 requires treating
                // such transforms as `None` to keep paint/hit-testing consistent.
                let s = Transform2D::scale_uniform(0.0);
                Some(t * s)
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                let Some(&child) = cx.children.first() else {
                    return cx.available;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                let _ = cx.layout_in(child, child_bounds);
                cx.available
            }

            fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
                let Some(&child) = cx.children.first() else {
                    return;
                };
                let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
                cx.paint(child, child_bounds);
            }
        }

        struct ElementLeaf;

        impl<H: UiHost> Widget<H> for ElementLeaf {
            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let window = AppWindowId::default();
        let mut app = crate::test_host::TestHost::new();
        let mut ui = UiTree::new();
        ui.set_window(window);

        let element = crate::elements::GlobalElementId(456);

        let root = ui.create_node(NonInvertibleRoot {
            delta: Point::new(Px(40.0), Px(0.0)),
        });
        let leaf = ui.create_node_for_element(element, ElementLeaf);
        ui.add_child(root, leaf);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert!(
            !scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushTransform { .. })),
            "non-invertible render transforms must not emit scene transform ops"
        );

        // `visual_bounds_for_element` is defined as a cross-frame query: the "last frame" value is
        // made visible after `prepare_window_for_frame` advances the window element state.
        app.advance_frame();

        let visual = crate::elements::visual_bounds_for_element(&mut app, window, element)
            .expect("expected visual bounds to be recorded during paint");
        assert_eq!(visual.origin, Point::new(Px(0.0), Px(0.0)));
        assert_eq!(visual.size, Size::new(Px(10.0), Px(10.0)));
    }

    #[test]
    fn outside_press_observer_must_not_capture_pointer_or_break_click_through() {
        struct CaptureOnPointerDownOutside;

        impl<H: UiHost> Widget<H> for CaptureOnPointerDownOutside {
            fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
                false
            }

            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                    cx.capture_pointer(cx.node);
                }
            }
        }

        let window = AppWindowId::default();

        let mut app = crate::test_host::TestHost::new();
        let clicks = app.models_mut().insert(0u32);

        let mut ui = UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(ClickCounter { clicks });
        ui.set_root(base);

        let overlay = ui.create_node(CaptureOnPointerDownOutside);
        let layer = ui.push_overlay_root_ex(overlay, false, true);
        ui.set_layer_wants_pointer_down_outside_events(layer, true);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let value = app.models().get(clicks).copied().unwrap_or(0);
        assert_eq!(
            value, 1,
            "expected click-through dispatch to reach underlay"
        );
        assert_eq!(
            ui.captured(),
            None,
            "observer pass must not capture pointer"
        );
    }

    #[test]
    fn outside_press_observer_dispatch_sets_input_context_phase() {
        struct RecordObserverPhase {
            phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
        }

        impl<H: UiHost> Widget<H> for RecordObserverPhase {
            fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
                false
            }

            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(self.phase, |v| *v = cx.input_ctx.dispatch_phase);
                }
            }
        }

        struct RecordNormalPhase {
            phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
        }

        impl<H: UiHost> Widget<H> for RecordNormalPhase {
            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(self.phase, |v| *v = cx.input_ctx.dispatch_phase);
                }
            }
        }

        let window = AppWindowId::default();

        let mut app = crate::test_host::TestHost::new();
        let observer_phase = app
            .models_mut()
            .insert(fret_runtime::InputDispatchPhase::Normal);
        let normal_phase = app
            .models_mut()
            .insert(fret_runtime::InputDispatchPhase::Observer);

        let mut ui = UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(RecordNormalPhase {
            phase: normal_phase,
        });
        ui.set_root(base);

        let overlay = ui.create_node(RecordObserverPhase {
            phase: observer_phase,
        });
        let layer = ui.push_overlay_root_ex(overlay, false, true);
        ui.set_layer_wants_pointer_down_outside_events(layer, true);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(
            app.models().get(observer_phase).copied(),
            Some(fret_runtime::InputDispatchPhase::Observer),
            "observer pass should tag InputContext as Observer"
        );
        assert_eq!(
            app.models().get(normal_phase).copied(),
            Some(fret_runtime::InputDispatchPhase::Normal),
            "normal hit-tested dispatch should tag InputContext as Normal"
        );
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
        assert_eq!(scene.ops_len(), 3);

        match (scene.ops()[0], scene.ops()[1], scene.ops()[2]) {
            (
                SceneOp::PushTransform { transform },
                SceneOp::Quad { rect, .. },
                SceneOp::PopTransform,
            ) => {
                assert_eq!(transform.tx, bounds_b.origin.x.0 - bounds_a.origin.x.0);
                assert_eq!(transform.ty, bounds_b.origin.y.0 - bounds_a.origin.y.0);
                assert_eq!(rect, bounds_a);
            }
            _ => panic!("expected push-transform + quad + pop-transform ops"),
        }
    }

    #[test]
    fn semantics_snapshot_includes_visible_roots_and_barrier() {
        let mut app = crate::test_host::TestHost::new();

        let mut ui = UiTree::new();
        ui.set_window(AppWindowId::default());

        let base = ui.create_node(TestStack);
        ui.set_root(base);
        let base_child = ui.create_node(TestStack);
        ui.add_child(base, base_child);

        let overlay_root = ui.create_node(TestStack);
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
    fn modal_barrier_clears_focus_and_capture_in_underlay() {
        struct CaptureOnDown;

        impl<H: UiHost> Widget<H> for CaptureOnDown {
            fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
                true
            }

            fn is_focusable(&self) -> bool {
                true
            }

            fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
                if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                }
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestStack);
        let underlay = ui.create_node(CaptureOnDown);
        ui.add_child(root, underlay);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(ui.focus(), Some(underlay));
        assert_eq!(ui.captured(), Some(underlay));

        let overlay_root = ui.create_node(TestStack);
        let _layer = ui.push_overlay_root(overlay_root, true);

        assert_eq!(ui.focus(), None);
        assert_eq!(ui.captured(), None);
    }

    #[test]
    fn focus_traversal_includes_roots_above_modal_barrier() {
        #[derive(Default)]
        struct Focusable;

        impl<H: UiHost> Widget<H> for Focusable {
            fn is_focusable(&self) -> bool {
                true
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let base_root = ui.create_node(TestStack);
        let underlay_focusable = ui.create_node(Focusable);
        ui.add_child(base_root, underlay_focusable);
        ui.set_root(base_root);

        let modal_root = ui.create_node(TestStack);
        let modal_focusable = ui.create_node(Focusable);
        ui.add_child(modal_root, modal_focusable);
        ui.push_overlay_root(modal_root, true);

        // Simulate a nested "portal" overlay that lives above the modal barrier (e.g. combobox popover
        // inside a dialog).
        let popup_root = ui.create_node(TestStack);
        let popup_focusable = ui.create_node(Focusable);
        ui.add_child(popup_root, popup_focusable);
        ui.push_overlay_root(popup_root, false);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Under a modal barrier, traversal must not reach underlay focusables.
        ui.set_focus(Some(modal_focusable));
        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(popup_focusable));

        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(modal_focusable));

        // Reverse direction should also wrap within the active layers set.
        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.previous"));
        assert_eq!(ui.focus(), Some(popup_focusable));
    }

    #[test]
    fn focus_traversal_prefers_topmost_overlay_root() {
        #[derive(Default)]
        struct Focusable;

        impl<H: UiHost> Widget<H> for Focusable {
            fn is_focusable(&self) -> bool {
                true
            }

            fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
                cx.available
            }
        }

        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let base_root = ui.create_node(TestStack);
        let base_focusable = ui.create_node(Focusable);
        ui.add_child(base_root, base_focusable);
        ui.set_root(base_root);

        let overlay_root = ui.create_node(TestStack);
        let overlay_focusable = ui.create_node(Focusable);
        ui.add_child(overlay_root, overlay_focusable);
        ui.push_overlay_root(overlay_root, false);

        let mut services = FakeUiServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.set_focus(Some(base_focusable));
        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(overlay_focusable));

        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(base_focusable));
    }

    #[test]
    fn tab_focus_next_runs_when_text_input_not_composing() {
        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(KeymapService {
            keymap: Keymap::from_v1(KeymapFileV1 {
                keymap_version: 1,
                bindings: vec![BindingV1 {
                    command: Some("focus.next".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                }],
            })
            .expect("valid keymap"),
        });

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestStack);
        let text_input = ui.create_node(crate::text_input::TextInput::new());
        ui.add_child(root, text_input);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.set_focus(Some(text_input));

        let _ = app.take_effects();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        let effects = app.take_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. } if *command == CommandId::from("focus.next")
            )),
            "expected focus traversal command effect"
        );
    }

    #[test]
    fn tab_focus_next_is_suppressed_during_ime_composition() {
        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(KeymapService {
            keymap: Keymap::from_v1(KeymapFileV1 {
                keymap_version: 1,
                bindings: vec![BindingV1 {
                    command: Some("focus.next".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                }],
            })
            .expect("valid keymap"),
        });

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestStack);
        let text_input = ui.create_node(crate::text_input::TextInput::new());
        ui.add_child(root, text_input);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.set_focus(Some(text_input));

        let _ = app.take_effects();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Ime(fret_core::ImeEvent::Preedit {
                text: "toukyou".into(),
                cursor: Some((0, 0)),
            }),
        );
        let _ = app.take_effects();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        let effects = app.take_effects();
        assert!(
            !effects.iter().any(|e| matches!(
                e,
                Effect::Command { command, .. } if *command == CommandId::from("focus.next")
            )),
            "did not expect focus traversal command effect during IME composition"
        );
    }

    #[test]
    fn reserved_shortcuts_are_suppressed_during_ime_composition() {
        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(KeymapService {
            keymap: Keymap::from_v1(KeymapFileV1 {
                keymap_version: 1,
                bindings: vec![
                    BindingV1 {
                        command: Some("test.tab".into()),
                        platform: None,
                        when: None,
                        keys: KeySpecV1 {
                            mods: vec![],
                            key: "Tab".into(),
                        },
                    },
                    BindingV1 {
                        command: Some("test.enter".into()),
                        platform: None,
                        when: None,
                        keys: KeySpecV1 {
                            mods: vec![],
                            key: "Enter".into(),
                        },
                    },
                    BindingV1 {
                        command: Some("test.numpad_enter".into()),
                        platform: None,
                        when: None,
                        keys: KeySpecV1 {
                            mods: vec![],
                            key: "NumpadEnter".into(),
                        },
                    },
                    BindingV1 {
                        command: Some("test.space".into()),
                        platform: None,
                        when: None,
                        keys: KeySpecV1 {
                            mods: vec![],
                            key: "Space".into(),
                        },
                    },
                    BindingV1 {
                        command: Some("test.escape".into()),
                        platform: None,
                        when: None,
                        keys: KeySpecV1 {
                            mods: vec![],
                            key: "Escape".into(),
                        },
                    },
                ],
            })
            .expect("valid keymap"),
        });

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestStack);
        let text_input = ui.create_node(crate::text_input::TextInput::new());
        ui.add_child(root, text_input);
        ui.set_root(root);

        let mut services = FakeUiServices;
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.set_focus(Some(text_input));

        let _ = app.take_effects();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Ime(fret_core::ImeEvent::Preedit {
                text: "toukyou".into(),
                cursor: Some((0, 0)),
            }),
        );
        let _ = app.take_effects();

        for key in [
            KeyCode::Tab,
            KeyCode::Enter,
            KeyCode::NumpadEnter,
            KeyCode::Space,
            KeyCode::Escape,
        ] {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &Event::KeyDown {
                    key,
                    modifiers: fret_core::Modifiers::default(),
                    repeat: false,
                },
            );
        }

        let effects = app.take_effects();
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::Command { .. })),
            "did not expect any shortcut commands during IME composition"
        );
    }

    #[test]
    fn remove_layer_uninstalls_overlay_and_removes_subtree() {
        let mut app = crate::test_host::TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let window = AppWindowId::default();
        let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestStack);
        ui.set_root(root);

        let overlay_root = ui.create_node(TestStack);
        let overlay_child = ui.create_node(TestStack);
        ui.add_child(overlay_root, overlay_child);
        let layer = ui.push_overlay_root(overlay_root, true);

        // Pretend an overlay widget captured focus/pointer.
        ui.focus = Some(overlay_child);
        ui.captured = Some(overlay_child);

        let mut services = FakeUiServices;
        let removed_root = ui.remove_layer(&mut services, layer);

        assert_eq!(removed_root, Some(overlay_root));
        assert!(ui.layers.get(layer).is_none());
        assert!(!ui.layer_order.contains(&layer));
        assert!(!ui.root_to_layer.contains_key(&overlay_root));

        assert!(ui.nodes.get(overlay_root).is_none());
        assert!(ui.nodes.get(overlay_child).is_none());
        assert_eq!(ui.focus(), None);
        assert_eq!(ui.captured(), None);
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

        let root = ui.create_node(TestStack);
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
