use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_app::App;
use fret_core::{Event, NodeId, Rect, Scene, Size};
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
    pub widget: Box<dyn Widget>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub bounds: Rect,
    pub invalidation: InvalidationFlags,
}

impl Node {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            parent: None,
            children: Vec::new(),
            bounds: Rect::default(),
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
        let target = self
            .captured
            .or(self.focus)
            .or_else(|| self.roots.first().copied());
        let Some(mut node_id) = target else {
            return;
        };

        loop {
            let (invalidations, parent) = {
                let Some(node) = self.nodes.get_mut(node_id) else {
                    break;
                };

                let mut cx = EventCx {
                    app,
                    node: node_id,
                    focus: self.focus,
                    captured: self.captured,
                    invalidations: Vec::new(),
                    commands: Vec::new(),
                };
                node.widget.event(&mut cx, event);
                (cx.invalidations, node.parent)
            };

            for (id, inv) in invalidations {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.invalidation.mark(inv);
                }
            }

            if self.captured.is_some() {
                break;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }
    }

    pub fn layout(&mut self, app: &mut App, root: NodeId, available: Size) -> Size {
        let Some(node) = self.nodes.get_mut(root) else {
            return Size::default();
        };
        let mut cx = LayoutCx {
            app,
            node: root,
            available,
        };
        let size = node.widget.layout(&mut cx);
        node.invalidation.layout = false;
        size
    }

    pub fn paint(&mut self, app: &mut App, root: NodeId, bounds: Rect, scene: &mut Scene) {
        let Some(node) = self.nodes.get_mut(root) else {
            return;
        };
        node.bounds = bounds;
        let mut cx = PaintCx {
            app,
            node: root,
            bounds,
            scene,
        };
        node.widget.paint(&mut cx);
        node.invalidation.paint = false;
    }
}
