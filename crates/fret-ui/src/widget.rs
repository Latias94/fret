use crate::{Theme, UiHost};
use fret_core::{
    AppWindowId, Corners, Event, NodeId, Point, Rect, Scene, SemanticsFlags, SemanticsRole, Size,
    Transform2D, UiServices,
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
    pub services: &'a mut dyn UiServices,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub input_ctx: InputContext,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub bounds: Rect,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub requested_focus: Option<NodeId>,
    pub requested_capture: Option<Option<NodeId>>,
    pub requested_cursor: Option<fret_core::CursorIcon>,
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

    pub fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
        if !self.input_ctx.caps.ui.cursor_icons {
            return;
        }
        self.requested_cursor = Some(icon);
    }
}

pub struct CommandCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
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
    pub services: &'a mut dyn UiServices,
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
    pub services: &'a mut dyn UiServices,
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
    pub(crate) label: &'a mut Option<String>,
    pub(crate) value: &'a mut Option<String>,
    pub(crate) actions: &'a mut fret_core::SemanticsActions,
    pub(crate) active_descendant: &'a mut Option<NodeId>,
}

impl<'a, H: UiHost> SemanticsCx<'a, H> {
    pub fn set_role(&mut self, role: SemanticsRole) {
        *self.role = role;
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        *self.label = Some(label.into());
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        *self.value = Some(value.into());
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.actions.focus = focusable;
    }

    pub fn set_invokable(&mut self, invokable: bool) {
        self.actions.invoke = invokable;
    }

    pub fn set_value_editable(&mut self, editable: bool) {
        self.actions.set_value = editable;
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

    pub fn set_checked(&mut self, checked: Option<bool>) {
        self.flags.checked = checked;
    }

    pub fn set_active_descendant(&mut self, node: Option<NodeId>) {
        *self.active_descendant = node;
    }
}

pub trait Widget<H: UiHost> {
    fn event(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}
    fn command(&mut self, _cx: &mut CommandCx<'_, H>, _command: &CommandId) -> bool {
        false
    }
    fn cleanup_resources(&mut self, _services: &mut dyn UiServices) {}
    /// Optional affine transform applied to both paint and input for the subtree rooted at this node.
    ///
    /// This is a "render transform" (not a layout transform):
    /// - Layout bounds remain authoritative for measurement and positioning.
    /// - The transform is expressed in the same coordinate space as `bounds` (logical px, window-local).
    /// - Hit-testing and pointer event positions are mapped through the inverse transform so input stays
    ///   consistent with the rendered output.
    ///
    /// Notes:
    /// - If the transform is not invertible, hit-testing and pointer event mapping fall back to the
    ///   untransformed behavior.
    /// - Paint caching may be disabled for nodes that return a transform, depending on runtime policy.
    fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        None
    }
    /// Whether hit-testing should be clipped to `bounds`.
    ///
    /// When `false`, children can receive pointer input even if they are positioned outside the
    /// parent's bounds (useful for `overflow: visible` + absolute-positioned badges/icons).
    ///
    /// Default: `true`.
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        true
    }
    /// Optional rounded-rectangle clip shape for hit-testing.
    ///
    /// When provided and `clips_hit_test(...)` is `true`, the runtime additionally clips pointer
    /// targeting to the rounded-rectangle defined by `bounds` + these corner radii. This keeps
    /// hit-testing consistent with `overflow: clip` + rounded corners.
    ///
    /// Default: `None` (rectangular clipping only).
    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
        None
    }
    /// Hit-test predicate for pointer input targeting.
    ///
    /// Returning `false` makes the node "transparent" to hit-testing (events fall through to
    /// underlay layers / widgets).
    ///
    /// Default: `true` (bounds-based hit testing).
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }
    /// Whether the node's children participate in hit-testing.
    ///
    /// When `false`, the entire subtree behaves like CSS `pointer-events: none` (useful for
    /// disabled controls that must not intercept events).
    ///
    /// Default: `true`.
    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        true
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
