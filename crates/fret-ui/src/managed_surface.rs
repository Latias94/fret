use std::any::Any;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, Rect, Size, UiServices};
use fret_runtime::CommandId;

use crate::layout_pass::LayoutPassKind;
use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, Invalidation, LayoutCx,
    PaintCx, PrepaintCx,
};
use crate::{Theme, UiHost};

pub type OnManagedSurfaceLayout<H> =
    Arc<dyn for<'a, 'b> Fn(&mut ManagedSurfaceLayoutCx<'a, 'b, H>) + 'static>;
pub type OnManagedSurfacePrepaint<H> =
    Arc<dyn for<'a, 'b> Fn(&mut ManagedSurfacePrepaintCx<'a, 'b, H>) + 'static>;
pub type OnManagedSurfacePaint<H> =
    Arc<dyn for<'a, 'b> Fn(&mut ManagedSurfacePaintCx<'a, 'b, H>) + 'static>;
pub type OnManagedSurfaceEvent<H> =
    Arc<dyn for<'a, 'b> Fn(&mut ManagedSurfaceEventCx<'a, 'b, H>, &fret_core::Event) + 'static>;
pub type OnManagedSurfaceCommand<H> =
    Arc<dyn for<'a, 'b> Fn(&mut ManagedSurfaceCommandCx<'a, 'b, H>, &CommandId) -> bool + 'static>;
pub type OnManagedSurfaceCommandAvailability<H> = Arc<
    dyn for<'a, 'b> Fn(
            &mut ManagedSurfaceCommandAvailabilityCx<'a, 'b, H>,
            &CommandId,
        ) -> CommandAvailability
        + 'static,
>;

pub(crate) struct ManagedSurfaceHooks<H: UiHost> {
    pub on_layout: Option<OnManagedSurfaceLayout<H>>,
    pub on_prepaint: Option<OnManagedSurfacePrepaint<H>>,
    pub on_paint: Option<OnManagedSurfacePaint<H>>,
    pub on_event: Option<OnManagedSurfaceEvent<H>>,
    pub on_command: Option<OnManagedSurfaceCommand<H>>,
    pub on_command_availability: Option<OnManagedSurfaceCommandAvailability<H>>,
}

impl<H: UiHost> Default for ManagedSurfaceHooks<H> {
    fn default() -> Self {
        Self {
            on_layout: None,
            on_prepaint: None,
            on_paint: None,
            on_event: None,
            on_command: None,
            on_command_availability: None,
        }
    }
}

pub struct ManagedSurfaceLayoutCx<'a, 'b, H: UiHost> {
    cx: &'a mut LayoutCx<'b, H>,
    laid_out: Vec<NodeId>,
}

impl<'a, 'b, H: UiHost> ManagedSurfaceLayoutCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut LayoutCx<'b, H>) -> Self {
        Self {
            cx,
            laid_out: Vec::new(),
        }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    pub fn available(&self) -> Size {
        self.cx.available
    }

    pub fn children(&self) -> &[NodeId] {
        self.cx.children
    }

    pub fn theme(&mut self) -> &Theme {
        self.cx.theme()
    }

    pub fn layout_child(&mut self, child: NodeId, bounds: Rect) -> Size {
        let size = self.cx.layout_in(child, bounds);
        self.laid_out.push(child);
        size
    }

    pub fn layout_child_root(&mut self, child: NodeId, bounds: Rect) -> Size {
        let size = self.cx.layout_viewport_root(child, bounds);
        self.laid_out.push(child);
        size
    }

    pub fn layout_unplaced_children(&mut self, bounds: Rect) {
        let children: Vec<NodeId> = self
            .cx
            .children
            .iter()
            .copied()
            .filter(|child| !self.laid_out.contains(child))
            .collect();
        for child in children {
            let _ = self.layout_child(child, bounds);
        }
    }

    pub fn set_output<T: Any>(&mut self, value: T) {
        if self.cx.pass_kind == LayoutPassKind::Final {
            self.cx.tree.set_prepaint_output(self.cx.node, value);
        }
    }

    pub fn output<T: Any>(&mut self) -> Option<&T> {
        self.cx.tree.prepaint_output(self.cx.node)
    }

    pub fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.cx.tree.prepaint_output_mut(self.cx.node)
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        self.cx.observe_global::<T>(invalidation);
    }

    pub fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }
}

pub struct ManagedSurfacePrepaintCx<'a, 'b, H: UiHost> {
    cx: &'a mut PrepaintCx<'b, H>,
}

impl<'a, 'b, H: UiHost> ManagedSurfacePrepaintCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut PrepaintCx<'b, H>) -> Self {
        Self { cx }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    pub fn theme(&self) -> &Theme {
        self.cx.theme()
    }

    pub fn set_output<T: Any>(&mut self, value: T) {
        self.cx.set_output(value);
    }

    pub fn output<T: Any>(&mut self) -> Option<&T> {
        self.cx.output()
    }

    pub fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.cx.output_mut()
    }

    pub fn invalidate_self(&mut self, invalidation: Invalidation) {
        self.cx.invalidate_self(invalidation);
    }

    pub fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }
}

pub struct ManagedSurfaceEventCx<'a, 'b, H: UiHost> {
    cx: &'a mut EventCx<'b, H>,
}

impl<'a, 'b, H: UiHost> ManagedSurfaceEventCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut EventCx<'b, H>) -> Self {
        Self { cx }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn pointer_id(&self) -> Option<fret_core::PointerId> {
        self.cx.pointer_id
    }

    pub fn pointer_position_window(&self, event: &fret_core::Event) -> Option<fret_core::Point> {
        self.cx.pointer_position_window(event)
    }

    pub fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    pub fn children(&self) -> &[NodeId] {
        self.cx.children
    }

    pub fn theme(&self) -> &Theme {
        self.cx.theme()
    }

    pub fn request_focus(&mut self, node: NodeId) {
        self.cx.request_focus(node);
    }

    pub fn capture_pointer(&mut self, node: NodeId) {
        self.cx.capture_pointer(node);
    }

    pub fn release_pointer_capture(&mut self) {
        self.cx.release_pointer_capture();
    }

    pub fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
        self.cx.set_cursor_icon(icon);
    }

    pub fn invalidate_self(&mut self, invalidation: Invalidation) {
        self.cx.invalidate_self(invalidation);
    }

    pub fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    pub fn push_effect(&mut self, effect: fret_runtime::Effect) {
        self.cx.app.push_effect(effect);
    }

    pub fn stop_propagation(&mut self) {
        self.cx.stop_propagation();
    }
}

pub struct ManagedSurfaceCommandCx<'a, 'b, H: UiHost> {
    cx: &'a mut CommandCx<'b, H>,
}

impl<'a, 'b, H: UiHost> ManagedSurfaceCommandCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut CommandCx<'b, H>) -> Self {
        Self { cx }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.cx.focus
    }

    pub fn request_focus(&mut self, node: NodeId) {
        self.cx.request_focus(node);
    }

    pub fn invalidate_self(&mut self, invalidation: Invalidation) {
        self.cx.invalidate_self(invalidation);
    }

    pub fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    pub fn notify(&mut self) {
        self.cx.notify();
    }

    pub fn stop_propagation(&mut self) {
        self.cx.stop_propagation();
    }
}

pub struct ManagedSurfaceCommandAvailabilityCx<'a, 'b, H: UiHost> {
    cx: &'a mut CommandAvailabilityCx<'b, H>,
}

impl<'a, 'b, H: UiHost> ManagedSurfaceCommandAvailabilityCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut CommandAvailabilityCx<'b, H>) -> Self {
        Self { cx }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.cx.focus
    }

    pub fn focus_in_subtree(&self) -> bool {
        self.cx
            .focus
            .map(|focus| self.cx.tree.is_descendant(self.cx.node, focus))
            .unwrap_or(false)
    }
}

pub struct ManagedSurfacePaintCx<'a, 'b, H: UiHost> {
    cx: &'a mut PaintCx<'b, H>,
}

impl<'a, 'b, H: UiHost> ManagedSurfacePaintCx<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut PaintCx<'b, H>) -> Self {
        Self { cx }
    }

    pub fn app(&mut self) -> &mut H {
        self.cx.app
    }

    pub fn node(&self) -> NodeId {
        self.cx.node
    }

    pub fn window(&self) -> Option<AppWindowId> {
        self.cx.window
    }

    pub fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    pub fn scale_factor(&self) -> f32 {
        self.cx.scale_factor
    }

    pub fn children(&self) -> &[NodeId] {
        self.cx.children
    }

    pub fn theme(&mut self) -> &Theme {
        self.cx.theme()
    }

    pub fn services(&mut self) -> &mut dyn UiServices {
        self.cx.services
    }

    /// Keep a paint-time text blob alive until this managed surface is repainted or cleaned up.
    ///
    /// This is for transient text emitted directly by managed-surface paint hooks. The blob remains
    /// valid for the scene that references it, but the hook does not need to store resource ids in
    /// its own cross-frame output.
    pub fn release_text_blob_on_next_paint(&mut self, blob: fret_core::TextBlobId) {
        self.cx.release_text_blob_on_next_paint(blob);
    }

    pub fn child_bounds(&self, child: NodeId) -> Option<Rect> {
        self.cx.child_bounds(child)
    }

    pub fn scene(&mut self) -> &mut fret_core::Scene {
        self.cx.scene
    }

    pub fn paint_child(&mut self, child: NodeId, bounds: Rect) {
        self.cx.paint(child, bounds);
    }

    pub fn output<T: Any>(&mut self) -> Option<&T> {
        self.cx.prepaint_output()
    }

    pub fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.cx.prepaint_output_mut()
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        self.cx.observe_global::<T>(invalidation);
    }

    pub fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }

    pub fn request_animation_frame_paint_only(&mut self) {
        self.cx.request_animation_frame_paint_only();
    }
}
