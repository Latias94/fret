use fret_app::{App, CommandId};
use fret_core::{AppWindowId, Event, NodeId, Rect, Scene, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalidation {
    Layout,
    Paint,
    HitTest,
}

pub struct EventCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub commands: Vec<CommandId>,
    pub requested_focus: Option<NodeId>,
    pub requested_capture: Option<Option<NodeId>>,
    pub stop_propagation: bool,
}

impl<'a> EventCx<'a> {
    pub fn invalidate(&mut self, node: NodeId, kind: Invalidation) {
        self.invalidations.push((node, kind));
    }

    pub fn invalidate_self(&mut self, kind: Invalidation) {
        self.invalidate(self.node, kind);
    }

    pub fn dispatch_command(&mut self, command: CommandId) {
        self.commands.push(command);
    }

    pub fn request_focus(&mut self, node: NodeId) {
        self.requested_focus = Some(node);
    }

    pub fn capture_pointer(&mut self, node: NodeId) {
        self.requested_capture = Some(Some(node));
    }

    pub fn release_pointer_capture(&mut self) {
        self.requested_capture = Some(None);
    }

    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }

    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }
}

pub struct LayoutCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub available: Size,
    pub layout_child: &'a mut dyn FnMut(NodeId, Rect) -> Size,
}

impl<'a> LayoutCx<'a> {
    pub fn layout(&mut self, child: NodeId, available: Size) -> Size {
        let rect = Rect::new(self.bounds.origin, available);
        (self.layout_child)(child, rect)
    }

    pub fn layout_in(&mut self, child: NodeId, bounds: Rect) -> Size {
        (self.layout_child)(child, bounds)
    }
}

pub struct PaintCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub scene: &'a mut Scene,
    pub paint_child: &'a mut dyn FnMut(NodeId, Rect),
    pub child_bounds: &'a dyn Fn(NodeId) -> Option<Rect>,
}

impl<'a> PaintCx<'a> {
    pub fn paint(&mut self, child: NodeId, bounds: Rect) {
        (self.paint_child)(child, bounds);
    }

    pub fn child_bounds(&self, child: NodeId) -> Option<Rect> {
        (self.child_bounds)(child)
    }
}

pub trait Widget {
    fn event(&mut self, _cx: &mut EventCx<'_>, _event: &Event) {}
    fn layout(&mut self, _cx: &mut LayoutCx<'_>) -> Size {
        Size::default()
    }
    fn paint(&mut self, _cx: &mut PaintCx<'_>) {}
}
