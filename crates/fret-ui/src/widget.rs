use fret_app::{App, CommandId};
use fret_core::{Event, NodeId, Rect, Scene, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalidation {
    Layout,
    Paint,
    HitTest,
}

pub struct EventCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub commands: Vec<CommandId>,
}

impl<'a> EventCx<'a> {
    pub fn invalidate(&mut self, node: NodeId, kind: Invalidation) {
        self.invalidations.push((node, kind));
    }

    pub fn dispatch_command(&mut self, command: CommandId) {
        self.commands.push(command);
    }
}

pub struct LayoutCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub available: Size,
}

pub struct PaintCx<'a> {
    pub app: &'a mut App,
    pub node: NodeId,
    pub bounds: Rect,
    pub scene: &'a mut Scene,
}

pub trait Widget {
    fn event(&mut self, _cx: &mut EventCx<'_>, _event: &Event) {}
    fn layout(&mut self, _cx: &mut LayoutCx<'_>) -> Size {
        Size::default()
    }
    fn paint(&mut self, _cx: &mut PaintCx<'_>) {}
}
