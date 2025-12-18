use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_app::App;
use fret_core::{AppWindowId, Event, NodeId, Point, PointerEvent, Rect, Scene, Size};
use slotmap::SlotMap;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InvalidationFlags {
    pub layout: bool,
    pub paint: bool,
    pub hit_test: bool,
}

impl InvalidationFlags {
    pub fn mark(&mut self, inv: Invalidation) {
        match inv {
            Invalidation::Layout => self.layout = true,
            Invalidation::Paint => self.paint = true,
            Invalidation::HitTest => self.hit_test = true,
        }
    }

    pub fn clear(&mut self) {
        self.layout = false;
        self.paint = false;
        self.hit_test = false;
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
}

pub struct Node {
    pub widget: Option<Box<dyn Widget>>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub bounds: Rect,
    pub measured_size: Size,
    pub invalidation: InvalidationFlags,
}

impl Node {
    pub fn new(widget: impl Widget + 'static) -> Self {
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
        }
    }
}

#[derive(Default)]
pub struct UiTree {
    nodes: SlotMap<NodeId, Node>,
    layers: SlotMap<UiLayerId, UiLayer>,
    layer_order: Vec<UiLayerId>,
    root_to_layer: HashMap<NodeId, UiLayerId>,
    base_layer: Option<UiLayerId>,
    focus: Option<NodeId>,
    captured: Option<NodeId>,
    window: Option<AppWindowId>,
}

impl UiTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_window(&mut self, window: AppWindowId) {
        self.window = Some(window);
    }

    pub fn create_node(&mut self, widget: impl Widget + 'static) -> NodeId {
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
        });
        self.root_to_layer.insert(root, id);
        self.layer_order.insert(0, id);
        self.base_layer = Some(id);
        id
    }

    pub fn push_overlay_root(&mut self, root: NodeId, blocks_underlay_input: bool) -> UiLayerId {
        let id = self.layers.insert(UiLayer {
            root,
            visible: true,
            blocks_underlay_input,
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

    pub fn layout_all(&mut self, app: &mut App, bounds: Rect) {
        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            let _ = self.layout_in(app, root, bounds);
        }
    }

    pub fn paint_all(&mut self, app: &mut App, bounds: Rect, scene: &mut Scene) {
        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            self.paint(app, root, bounds, scene);
        }
    }

    pub fn dispatch_event(&mut self, app: &mut App, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        let (active_layers, barrier_root) = self.active_input_layers();

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

        let target = if let Some(captured) = self.captured {
            Some(captured)
        } else if let Event::Pointer(pe) = event {
            let pos = pointer_position(pe);
            self.hit_test_layers(&active_layers, pos)
                .or(barrier_root)
                .or(Some(default_root))
        } else {
            self.focus.or(Some(default_root))
        };

        let Some(mut node_id) = target else {
            return;
        };

        loop {
            let (invalidations, requested_focus, requested_capture, stop_propagation, parent) =
                self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let children: Vec<NodeId> = tree
                        .nodes
                        .get(node_id)
                        .map(|n| n.children.clone())
                        .unwrap_or_default();
                    let mut cx = EventCx {
                        app,
                        node: node_id,
                        window: tree.window,
                        children: &children,
                        focus: tree.focus,
                        captured: tree.captured,
                        invalidations: Vec::new(),
                        commands: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
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
                self.focus = Some(focus);
            }

            if let Some(capture) = requested_capture {
                self.captured = capture;
            };

            if self.captured.is_some() || stop_propagation {
                break;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }
    }

    pub fn layout(&mut self, app: &mut App, root: NodeId, available: Size) -> Size {
        let bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            available,
        );
        self.layout_in(app, root, bounds)
    }

    pub fn layout_in(&mut self, app: &mut App, root: NodeId, bounds: Rect) -> Size {
        self.layout_node(app, root, bounds)
    }

    pub fn paint(&mut self, app: &mut App, root: NodeId, bounds: Rect, scene: &mut Scene) {
        self.paint_node(app, root, bounds, scene);
    }

    fn with_widget_mut<R>(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut dyn Widget, &mut UiTree) -> R,
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

    fn layout_node(&mut self, app: &mut App, node: NodeId, bounds: Rect) -> Size {
        let (prev_bounds, measured, invalidated) = match self.nodes.get(node) {
            Some(n) => (n.bounds, n.measured_size, n.invalidation.layout),
            None => return Size::default(),
        };

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

        let needs_layout = invalidated || prev_bounds != bounds;
        if !needs_layout {
            return measured;
        }

        let tree_ptr: *mut UiTree = self;
        let app_ptr: *mut App = app;
        let mut layout_child = move |child: NodeId, bounds: Rect| -> Size {
            unsafe { (&mut *tree_ptr).layout_node(&mut *app_ptr, child, bounds) }
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
                children: &children,
                bounds,
                available: bounds.size,
                layout_child: &mut layout_child,
            };
            widget.layout(&mut cx)
        });

        if let Some(n) = self.nodes.get_mut(node) {
            n.measured_size = size;
            n.invalidation.layout = false;
        }

        size
    }

    fn paint_node(&mut self, app: &mut App, node: NodeId, bounds: Rect, scene: &mut Scene) {
        let tree_ref: *const UiTree = self as *const UiTree;
        let tree_ptr: *mut UiTree = self;
        let app_ptr: *mut App = app;
        let scene_ptr: *mut Scene = scene;
        let mut paint_child = move |child: NodeId, bounds: Rect| {
            unsafe { (&mut *tree_ptr).paint_node(&mut *app_ptr, child, bounds, &mut *scene_ptr) };
        };
        let child_bounds = move |child: NodeId| -> Option<Rect> {
            unsafe { (&*tree_ref).nodes.get(child).map(|n| n.bounds) }
        };

        if let Some(n) = self.nodes.get_mut(node) {
            n.bounds = bounds;
        }

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
                children: &children,
                bounds,
                scene,
                paint_child: &mut paint_child,
                child_bounds: &child_bounds,
            };
            widget.paint(&mut cx);
        });

        if let Some(n) = self.nodes.get_mut(node) {
            n.invalidation.paint = false;
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
        if !n.bounds.contains(position) {
            return None;
        }

        for &child in n.children.iter().rev() {
            if let Some(hit) = self.hit_test_node(child, position) {
                return Some(hit);
            }
        }

        Some(node)
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
            roots.push(self.layers[*layer].root);
        }

        let barrier_root = barrier_index.map(|idx| self.layers[visible[idx]].root);
        (roots, barrier_root)
    }

    fn node_in_any_layer(&self, node: NodeId, layer_roots: &[NodeId]) -> bool {
        let Some(node_root) = self.node_root(node) else {
            return false;
        };
        layer_roots.iter().any(|r| *r == node_root)
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
