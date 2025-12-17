use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_app::App;
use fret_core::{Event, NodeId, Point, PointerEvent, Rect, Scene, Size};
use slotmap::SlotMap;

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
    roots: Vec<NodeId>,
    focus: Option<NodeId>,
    captured: Option<NodeId>,
}

impl UiTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&mut self, widget: impl Widget + 'static) -> NodeId {
        self.nodes.insert(Node::new(widget))
    }

    pub fn set_root(&mut self, root: NodeId) {
        if !self.roots.contains(&root) {
            self.roots.push(root);
        }
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

    pub fn dispatch_event(&mut self, app: &mut App, event: &Event) {
        let root = self.roots.first().copied();
        let Some(root) = root else {
            return;
        };

        let target = if let Some(captured) = self.captured {
            Some(captured)
        } else if let Event::Pointer(pe) = event {
            self.hit_test(root, pointer_position(pe)).or(Some(root))
        } else {
            self.focus.or(Some(root))
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
        self.layout_node(app, root, available)
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

    fn layout_node(&mut self, app: &mut App, node: NodeId, available: Size) -> Size {
        let tree_ptr: *mut UiTree = self;
        let app_ptr: *mut App = app;
        let mut layout_child = move |child: NodeId, available: Size| -> Size {
            unsafe { (&mut *tree_ptr).layout_node(&mut *app_ptr, child, available) }
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
                children: &children,
                available,
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

    fn paint_node(
        &mut self,
        app: &mut App,
        node: NodeId,
        bounds: Rect,
        scene: &mut Scene,
    ) {
        let tree_ptr: *mut UiTree = self;
        let app_ptr: *mut App = app;
        let scene_ptr: *mut Scene = scene;
        let mut paint_child = move |child: NodeId, bounds: Rect| {
            unsafe { (&mut *tree_ptr).paint_node(&mut *app_ptr, child, bounds, &mut *scene_ptr) };
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
                children: &children,
                bounds,
                scene,
                paint_child: &mut paint_child,
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
}

fn pointer_position(pe: &PointerEvent) -> Point {
    match pe {
        PointerEvent::Move { position }
        | PointerEvent::Down { position, .. }
        | PointerEvent::Up { position, .. }
        | PointerEvent::Wheel { position, .. } => *position,
    }
}
