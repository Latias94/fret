use crate::{Theme, UiHost};
use fret_core::{
    AppWindowId, Corners, Event, NodeId, Point, Rect, Scene, SemanticsFlags, SemanticsRole, Size,
    Transform2D, UiServices,
};
use fret_runtime::{CommandId, Effect, InputContext, Model, ModelId};
use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::layout_constraints::LayoutConstraints;

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
    pub tree: &'a mut crate::tree::UiTree<H>,
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
    pub tree: &'a mut crate::tree::UiTree<H>,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub focus: Option<NodeId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub available: Size,
    pub scale_factor: f32,
    pub services: &'a mut dyn UiServices,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub observe_global: &'a mut dyn FnMut(TypeId, Invalidation),
}

impl<'a, H: UiHost> LayoutCx<'a, H> {
    pub fn theme(&mut self) -> &Theme {
        self.observe_global::<Theme>(Invalidation::Layout);
        Theme::global(&*self.app)
    }

    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate(self.node, Invalidation::Paint);
        let Some(window) = self.window else {
            return;
        };
        self.app.push_effect(Effect::RequestAnimationFrame(window));
    }

    pub fn observe_model<T>(&mut self, model: &Model<T>, invalidation: Invalidation) {
        (self.observe_model)(model.id(), invalidation);
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        (self.observe_global)(TypeId::of::<T>(), invalidation);
    }

    pub fn layout(&mut self, child: NodeId, available: Size) -> Size {
        let rect = Rect::new(self.bounds.origin, available);
        self.layout_in(child, rect)
    }

    pub fn layout_in(&mut self, child: NodeId, bounds: Rect) -> Size {
        self.tree
            .layout_in(self.app, self.services, child, bounds, self.scale_factor)
    }

    pub fn layout_engine_child_bounds(&mut self, child: NodeId) -> Option<Rect> {
        #[cfg(feature = "layout-engine-v2")]
        {
            let local = self.tree.layout_engine_child_local_rect(self.node, child)?;
            Some(Rect::new(
                Point::new(
                    fret_core::Px(self.bounds.origin.x.0 + local.origin.x.0),
                    fret_core::Px(self.bounds.origin.y.0 + local.origin.y.0),
                ),
                local.size,
            ))
        }

        #[cfg(not(feature = "layout-engine-v2"))]
        {
            let _ = child;
            None
        }
    }

    pub fn layout_viewport_root(&mut self, child: NodeId, bounds: Rect) -> Size {
        #[cfg(feature = "layout-engine-v2")]
        {
            self.tree.register_viewport_root(child, bounds);
            bounds.size
        }

        #[cfg(not(feature = "layout-engine-v2"))]
        {
            self.layout_in(child, bounds)
        }
    }
    pub fn measure_in(&mut self, child: NodeId, constraints: LayoutConstraints) -> Size {
        self.tree.measure_in(
            self.app,
            self.services,
            child,
            constraints,
            self.scale_factor,
        )
    }
}

pub struct MeasureCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub tree: &'a mut crate::tree::UiTree<H>,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub focus: Option<NodeId>,
    pub children: &'a [NodeId],
    pub constraints: LayoutConstraints,
    pub scale_factor: f32,
    pub services: &'a mut dyn UiServices,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub observe_global: &'a mut dyn FnMut(TypeId, Invalidation),
}

impl<'a, H: UiHost> MeasureCx<'a, H> {
    pub fn theme(&mut self) -> &Theme {
        self.observe_global::<Theme>(Invalidation::Layout);
        Theme::global(&*self.app)
    }

    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate(self.node, Invalidation::Paint);
        let Some(window) = self.window else {
            return;
        };
        self.app.push_effect(Effect::RequestAnimationFrame(window));
    }

    pub fn observe_model<T>(&mut self, model: &Model<T>, invalidation: Invalidation) {
        (self.observe_model)(model.id(), invalidation);
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        (self.observe_global)(TypeId::of::<T>(), invalidation);
    }

    pub fn measure_in(&mut self, child: NodeId, constraints: LayoutConstraints) -> Size {
        self.tree.measure_in(
            self.app,
            self.services,
            child,
            constraints,
            self.scale_factor,
        )
    }
}

pub struct PaintCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub tree: &'a mut crate::tree::UiTree<H>,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub focus: Option<NodeId>,
    pub children: &'a [NodeId],
    pub bounds: Rect,
    pub scale_factor: f32,
    pub accumulated_transform: Transform2D,
    pub services: &'a mut dyn UiServices,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub observe_global: &'a mut dyn FnMut(TypeId, Invalidation),
    pub scene: &'a mut Scene,
}

impl<'a, H: UiHost> PaintCx<'a, H> {
    pub fn theme(&mut self) -> &Theme {
        self.observe_global::<Theme>(Invalidation::Layout);
        Theme::global(&*self.app)
    }

    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate(self.node, Invalidation::Paint);
        let Some(window) = self.window else {
            return;
        };
        self.app.push_effect(Effect::RequestAnimationFrame(window));
    }

    pub fn observe_model<T>(&mut self, model: &Model<T>, invalidation: Invalidation) {
        (self.observe_model)(model.id(), invalidation);
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        (self.observe_global)(TypeId::of::<T>(), invalidation);
    }

    pub fn paint(&mut self, child: NodeId, bounds: Rect) {
        self.tree.paint_node(
            self.app,
            self.services,
            child,
            bounds,
            self.scene,
            self.scale_factor,
            self.accumulated_transform,
        )
    }

    pub fn child_bounds(&self, child: NodeId) -> Option<Rect> {
        self.tree.node_bounds(child)
    }
}

pub struct SemanticsCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub element_id_map: Option<&'a HashMap<u64, NodeId>>,
    pub bounds: Rect,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub(crate) role: &'a mut SemanticsRole,
    pub(crate) flags: &'a mut SemanticsFlags,
    pub(crate) label: &'a mut Option<String>,
    pub(crate) value: &'a mut Option<String>,
    pub(crate) text_selection: &'a mut Option<(u32, u32)>,
    pub(crate) text_composition: &'a mut Option<(u32, u32)>,
    pub(crate) actions: &'a mut fret_core::SemanticsActions,
    pub(crate) active_descendant: &'a mut Option<NodeId>,
    pub(crate) pos_in_set: &'a mut Option<u32>,
    pub(crate) set_size: &'a mut Option<u32>,
    pub(crate) labelled_by: &'a mut Vec<NodeId>,
    pub(crate) described_by: &'a mut Vec<NodeId>,
    pub(crate) controls: &'a mut Vec<NodeId>,
}

impl<'a, H: UiHost> SemanticsCx<'a, H> {
    pub fn resolve_declarative_element(&self, element: u64) -> Option<NodeId> {
        self.element_id_map.and_then(|m| m.get(&element).copied())
    }

    pub fn set_role(&mut self, role: SemanticsRole) {
        *self.role = role;
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        *self.label = Some(label.into());
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        *self.value = Some(value.into());
    }

    pub fn set_text_selection(&mut self, anchor: u32, focus: u32) {
        *self.text_selection = Some((anchor, focus));
    }

    pub fn clear_text_selection(&mut self) {
        *self.text_selection = None;
    }

    pub fn set_text_composition(&mut self, start: u32, end: u32) {
        *self.text_composition = Some((start, end));
    }

    pub fn clear_text_composition(&mut self) {
        *self.text_composition = None;
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

    pub fn set_text_selection_supported(&mut self, supported: bool) {
        self.actions.set_text_selection = supported;
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

    pub fn set_pos_in_set(&mut self, pos_in_set: Option<u32>) {
        *self.pos_in_set = pos_in_set;
    }

    pub fn set_set_size(&mut self, set_size: Option<u32>) {
        *self.set_size = set_size;
    }

    pub fn set_collection_position(&mut self, pos_in_set: Option<u32>, set_size: Option<u32>) {
        *self.pos_in_set = pos_in_set;
        *self.set_size = set_size;
    }

    pub fn push_labelled_by(&mut self, node: NodeId) {
        if self.labelled_by.contains(&node) {
            return;
        }
        self.labelled_by.push(node);
    }

    pub fn clear_labelled_by(&mut self) {
        self.labelled_by.clear();
    }

    pub fn push_described_by(&mut self, node: NodeId) {
        if self.described_by.contains(&node) {
            return;
        }
        self.described_by.push(node);
    }

    pub fn clear_described_by(&mut self) {
        self.described_by.clear();
    }

    pub fn push_controlled(&mut self, node: NodeId) {
        if self.controls.contains(&node) {
            return;
        }
        self.controls.push(node);
    }

    pub fn clear_controls(&mut self) {
        self.controls.clear();
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
    /// Whether this node should be included in the semantics snapshot.
    ///
    /// This is a mechanism-only gate used to model `present=false` (display-none) subtrees that
    /// should not be exposed to assistive tech, while still keeping element state alive (e.g.
    /// Radix-style `forceMount`).
    ///
    /// Default: `true`.
    fn semantics_present(&self) -> bool {
        true
    }
    /// Whether semantics snapshot traversal should recurse into this node's children.
    ///
    /// Default: `true`.
    fn semantics_children(&self) -> bool {
        true
    }
    /// Whether focus traversal should recurse into this node's children.
    ///
    /// This is a mechanism-only gate used by `UiTree` to model "inert" subtrees during
    /// transitions (e.g. `present=true` but `interactive=false`), without requiring every focusable
    /// widget to thread an "interactive" flag into its own `is_focusable()` logic.
    ///
    /// Default: `true`.
    fn focus_traversal_children(&self) -> bool {
        true
    }
    fn is_focusable(&self) -> bool {
        false
    }
    fn is_text_input(&self) -> bool {
        false
    }
    /// Whether this node can scroll a focused descendant into view.
    ///
    /// This is a mechanism-only capability used by `UiTree` to implement a minimal
    /// "scroll-into-view" contract for focus traversal (ADR 0068) without coupling focus traversal
    /// policy into component crates.
    fn can_scroll_descendant_into_view(&self) -> bool {
        false
    }
    fn scroll_descendant_into_view(
        &mut self,
        _cx: &mut ScrollIntoViewCx<'_, H>,
        _descendant_bounds: Rect,
    ) -> ScrollIntoViewResult {
        ScrollIntoViewResult::NotHandled
    }
    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        Size::default()
    }
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::default()
    }
    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    fn semantics(&mut self, _cx: &mut SemanticsCx<'_, H>) {}
}

pub enum ScrollIntoViewResult {
    NotHandled,
    Handled { did_scroll: bool },
}

pub struct ScrollIntoViewCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub bounds: Rect,
}
