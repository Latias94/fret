use crate::{Theme, UiHost};
use fret_core::{
    AppWindowId, Event, NodeId, Rect, Scene, SemanticsFlags, SemanticsRole, Size, TextService,
};
use fret_runtime::{CommandId, Effect, InputContext, Model, ModelId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalidation {
    Layout,
    Paint,
    HitTest,
}

pub struct EventCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub text: &'a mut dyn TextService,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub input_ctx: InputContext,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub requested_focus: Option<NodeId>,
    pub requested_capture: Option<Option<NodeId>>,
    pub stop_propagation: bool,
}

impl<'a, H: UiHost> EventCx<'a, H> {
    pub fn theme(&self) -> &Theme {
        Theme::global(&*self.app)
    }

    pub fn invalidate(&mut self, node: NodeId, kind: Invalidation) {
        self.invalidations.push((node, kind));
    }

    pub fn invalidate_self(&mut self, kind: Invalidation) {
        self.invalidate(self.node, kind);
    }

    pub fn dispatch_command(&mut self, command: CommandId) {
        self.app.push_effect(Effect::Command {
            window: self.window,
            command,
        });
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

pub struct CommandCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub text: &'a mut dyn TextService,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub input_ctx: InputContext,
    pub focus: Option<NodeId>,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub requested_focus: Option<NodeId>,
    pub stop_propagation: bool,
}

impl<'a, H: UiHost> CommandCx<'a, H> {
    pub fn theme(&self) -> &Theme {
        Theme::global(&*self.app)
    }

    pub fn invalidate(&mut self, node: NodeId, kind: Invalidation) {
        self.invalidations.push((node, kind));
    }

    pub fn invalidate_self(&mut self, kind: Invalidation) {
        self.invalidate(self.node, kind);
    }

    pub fn request_focus(&mut self, node: NodeId) {
        self.requested_focus = Some(node);
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

pub struct LayoutCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub focus: Option<NodeId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub available: Size,
    pub scale_factor: f32,
    pub text: &'a mut dyn TextService,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub layout_child: &'a mut dyn FnMut(NodeId, Rect) -> Size,
}

impl<'a, H: UiHost> LayoutCx<'a, H> {
    pub fn theme(&self) -> &Theme {
        Theme::global(&*self.app)
    }

    pub fn observe_model<T>(&mut self, model: Model<T>, invalidation: Invalidation) {
        (self.observe_model)(model.id(), invalidation);
    }

    pub fn layout(&mut self, child: NodeId, available: Size) -> Size {
        let rect = Rect::new(self.bounds.origin, available);
        (self.layout_child)(child, rect)
    }

    pub fn layout_in(&mut self, child: NodeId, bounds: Rect) -> Size {
        (self.layout_child)(child, bounds)
    }
}

pub struct PaintCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub focus: Option<NodeId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub scale_factor: f32,
    pub text: &'a mut dyn TextService,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub scene: &'a mut Scene,
    pub paint_child: &'a mut dyn FnMut(NodeId, Rect),
    pub child_bounds: &'a dyn Fn(NodeId) -> Option<Rect>,
}

impl<'a, H: UiHost> PaintCx<'a, H> {
    pub fn theme(&self) -> &Theme {
        Theme::global(&*self.app)
    }

    pub fn observe_model<T>(&mut self, model: Model<T>, invalidation: Invalidation) {
        (self.observe_model)(model.id(), invalidation);
    }

    pub fn paint(&mut self, child: NodeId, bounds: Rect) {
        (self.paint_child)(child, bounds);
    }

    pub fn child_bounds(&self, child: NodeId) -> Option<Rect> {
        (self.child_bounds)(child)
    }
}

pub struct SemanticsCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub bounds: Rect,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub(crate) role: &'a mut SemanticsRole,
    pub(crate) flags: &'a mut SemanticsFlags,
}

impl<'a, H: UiHost> SemanticsCx<'a, H> {
    pub fn set_role(&mut self, role: SemanticsRole) {
        *self.role = role;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.flags.disabled = disabled;
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.flags.selected = selected;
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.flags.expanded = expanded;
    }
}

pub trait Widget<H: UiHost> {
    fn event(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}
    fn command(&mut self, _cx: &mut CommandCx<'_, H>, _command: &CommandId) -> bool {
        false
    }
    fn is_focusable(&self) -> bool {
        false
    }
    fn is_text_input(&self) -> bool {
        false
    }
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::default()
    }
    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    fn semantics(&mut self, _cx: &mut SemanticsCx<'_, H>) {}
}
