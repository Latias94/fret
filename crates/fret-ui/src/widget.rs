use crate::{Theme, UiHost};
use fret_core::{
    AppWindowId, Corners, Event, NodeId, Point, Rect, Scene, SemanticsFlags, SemanticsRole, Size,
    Transform2D, UiServices,
};
use fret_runtime::{
    CommandId, DefaultAction, DefaultActionSet, Effect, InputContext, Model, ModelId,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::layout_constraints::LayoutConstraints;
use crate::layout_pass::LayoutPassKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalidation {
    Layout,
    Paint,
    HitTest,
    /// Recompute hit-testing and repaint, without forcing a layout pass.
    ///
    /// This is intended for state changes that affect coordinate mapping (e.g. scrolling) but do
    /// not change layout geometry.
    HitTestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSourceLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

pub struct EventCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
    pub node: NodeId,
    pub layer_root: Option<NodeId>,
    pub window: Option<AppWindowId>,
    pub pointer_id: Option<fret_core::PointerId>,
    pub input_ctx: InputContext,
    pub prevented_default_actions: &'a mut DefaultActionSet,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub bounds: Rect,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub requested_focus: Option<NodeId>,
    pub requested_capture: Option<Option<NodeId>>,
    pub requested_cursor: Option<fret_core::CursorIcon>,
    pub notify_requested: bool,
    pub notify_requested_location: Option<UiSourceLocation>,
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
        if self.pointer_id.is_none() {
            return;
        }
        self.requested_capture = Some(Some(node));
    }

    pub fn release_pointer_capture(&mut self) {
        if self.pointer_id.is_none() {
            return;
        }
        self.requested_capture = Some(None);
    }

    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }

    pub fn prevent_default(&mut self, action: DefaultAction) {
        self.prevented_default_actions.insert(action);
    }

    pub fn default_prevented(&self, action: DefaultAction) -> bool {
        self.prevented_default_actions.contains(action)
    }

    /// Request a window redraw (one-shot).
    ///
    /// Use this for one-shot updates after state changes (e.g. responding to input).
    ///
    /// Notes:
    /// - A redraw does not necessarily imply a fresh widget `paint()` pass if the UI tree can
    ///   replay a valid paint cache entry. If you need frame-driven updates, prefer
    ///   `request_animation_frame()` (from `LayoutCx`/`PaintCx`/`MeasureCx`) which also ensures
    ///   `Invalidation::Paint` is set.
    /// - `request_redraw()` is not a timer. If you need continuous progression without input
    ///   (animations, progressive rendering), you must request the next frame via
    ///   `request_animation_frame()` (or a higher-level continuous-frames helper).
    /// - A redraw request may be coalesced and does not necessarily wake a sleeping event loop on
    ///   all platforms. Prefer `request_animation_frame()` for frame-driven progression.
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Mark the current view as dirty and schedule a redraw.
    ///
    /// In view-cache mode, this forces the nearest cache root to rerender (skip view-cache reuse)
    /// and prevents paint replay of stale recorded ranges.
    #[track_caller]
    pub fn notify(&mut self) {
        self.notify_requested = true;
        if self.notify_requested_location.is_none() {
            let caller = std::panic::Location::caller();
            self.notify_requested_location = Some(UiSourceLocation {
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            });
        }
    }

    pub fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
        if !self.input_ctx.caps.ui.cursor_icons {
            return;
        }
        self.requested_cursor = Some(icon);
    }
}

/// Observer-only event context for the `InputDispatchPhase::Preview` pass.
///
/// This pass exists to support "click-through outside-press" policies (ADR 0069) without allowing
/// widgets to mutate input routing state (focus / capture / propagation / default actions).
pub struct ObserverCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub pointer_id: Option<fret_core::PointerId>,
    pub input_ctx: InputContext,
    pub children: &'a [NodeId],
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
    pub bounds: Rect,
    pub invalidations: Vec<(NodeId, Invalidation)>,
    pub notify_requested: bool,
    pub notify_requested_location: Option<UiSourceLocation>,
}

impl<'a, H: UiHost> ObserverCx<'a, H> {
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

    /// Request a window redraw (one-shot).
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Mark the current view as dirty and schedule a redraw.
    ///
    /// In view-cache mode, this forces the nearest cache root to rerender (skip view-cache reuse)
    /// and prevents paint replay of stale recorded ranges.
    #[track_caller]
    pub fn notify(&mut self) {
        self.notify_requested = true;
        if self.notify_requested_location.is_none() {
            let caller = std::panic::Location::caller();
            self.notify_requested_location = Some(UiSourceLocation {
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            });
        }
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

    /// Request a window redraw.
    ///
    /// Use this for one-shot updates after state changes. For frame-driven updates (animations,
    /// progressive rendering), prefer `request_animation_frame()` when available.
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }
}

/// Command availability query result used by `UiTree::is_command_available` (ADR 1157).
///
/// This is a pure query signal (no side effects). Consumers typically interpret:
/// - `Available`: command should be treated as enabled for the current dispatch path.
/// - `Blocked`: command must not bubble further to ancestors for availability purposes.
/// - `NotHandled`: this node does not participate in availability for this command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAvailability {
    NotHandled,
    Available,
    Blocked,
}

/// Context passed to `Widget::command_availability`.
///
/// This is intentionally read-only (no `UiServices`, no invalidations) to keep availability a pure
/// query.
pub struct CommandAvailabilityCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub tree: &'a crate::tree::UiTree<H>,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub input_ctx: InputContext,
    pub focus: Option<NodeId>,
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
    pub pass_kind: LayoutPassKind,
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

    /// Request a window redraw (one-shot).
    ///
    /// This schedules a paint of the current UI state. If you need continuous frame progression
    /// (e.g. animations or progressive rendering without input), use `request_animation_frame()`.
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Request the next animation frame for this window.
    ///
    /// Use this for frame-driven behaviors (animations, progress indicators, progressive
    /// rendering) where the UI must keep repainting even if there are no incoming events.
    ///
    /// This is a one-shot request. Code that animates should re-issue
    /// `request_animation_frame()` each frame while it remains active.
    ///
    /// This method also ensures `Invalidation::Paint` is set for the calling node so paint caching
    /// cannot short-circuit the widget `paint()` pass on the next frame.
    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate_with_source_and_detail(
            self.node,
            Invalidation::Paint,
            crate::tree::UiDebugInvalidationSource::Notify,
            crate::tree::UiDebugInvalidationDetail::AnimationFrameRequest,
        );
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
        self.tree.layout_in_with_pass_kind(
            self.app,
            self.services,
            child,
            bounds,
            self.scale_factor,
            self.pass_kind,
        )
    }

    pub fn layout_in_probe(&mut self, child: NodeId, bounds: Rect) -> Size {
        self.tree.layout_in_with_pass_kind(
            self.app,
            self.services,
            child,
            bounds,
            self.scale_factor,
            LayoutPassKind::Probe,
        )
    }

    pub fn layout_engine_child_bounds(&mut self, child: NodeId) -> Option<Rect> {
        let local = self.tree.layout_engine_child_local_rect(self.node, child)?;
        Some(Rect::new(
            Point::new(
                fret_core::Px(self.bounds.origin.x.0 + local.origin.x.0),
                fret_core::Px(self.bounds.origin.y.0 + local.origin.y.0),
            ),
            local.size,
        ))
    }

    pub fn layout_viewport_root(&mut self, child: NodeId, bounds: Rect) -> Size {
        if self.pass_kind == LayoutPassKind::Probe {
            return bounds.size;
        }
        self.tree.register_viewport_root(child, bounds);
        bounds.size
    }

    pub fn solve_barrier_child_root(&mut self, child: NodeId, bounds: Rect) {
        if self.pass_kind != LayoutPassKind::Final {
            return;
        }
        self.tree.solve_barrier_flow_root(
            self.app,
            self.services,
            child,
            bounds,
            self.scale_factor,
        );
    }

    pub fn solve_barrier_child_root_if_needed(&mut self, child: NodeId, bounds: Rect) {
        if self.pass_kind != LayoutPassKind::Final {
            return;
        }
        self.tree.solve_barrier_flow_root_if_needed(
            self.app,
            self.services,
            child,
            bounds,
            self.scale_factor,
        );
    }

    pub fn solve_barrier_child_roots_if_needed(&mut self, roots: &[(NodeId, Rect)]) {
        if self.pass_kind != LayoutPassKind::Final {
            return;
        }
        self.tree.solve_barrier_flow_roots_if_needed(
            self.app,
            self.services,
            roots,
            self.scale_factor,
        );
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

    /// Request a window redraw (one-shot).
    ///
    /// This is typically used after mutating model/state in response to user input. For
    /// frame-driven updates, use `request_animation_frame()`.
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Request the next animation frame for this window.
    ///
    /// Use this for animations/progressive rendering that must advance without input events.
    ///
    /// This is a one-shot request. Callers should re-issue `request_animation_frame()` each frame
    /// while it remains active.
    /// This also sets `Invalidation::Paint` for the current node so paint caching cannot skip
    /// widget `paint()` on the next frame.
    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate_with_source_and_detail(
            self.node,
            Invalidation::Paint,
            crate::tree::UiDebugInvalidationSource::Notify,
            crate::tree::UiDebugInvalidationDetail::AnimationFrameRequest,
        );
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
        if !self.tree.debug_enabled() {
            return self.tree.measure_in(
                self.app,
                self.services,
                child,
                constraints,
                self.scale_factor,
            );
        }

        let started = fret_core::time::Instant::now();
        let size = self.tree.measure_in(
            self.app,
            self.services,
            child,
            constraints,
            self.scale_factor,
        );
        let elapsed = started.elapsed();
        self.tree
            .debug_record_measure_child(self.node, child, elapsed);
        size
    }
}

/// Prepaint context invoked after layout, before paint.
///
/// This is intentionally narrow: it exists to support GPUI-aligned "ephemeral prepaint items"
/// workflows (ADR 0182 / ADR 0190) without forcing a full rerender/relayout of a cache root.
///
/// Notes:
/// - Prepaint runs after layout bounds are known.
/// - Prepaint may request redraw/animation frames, but should avoid structural tree mutations.
pub struct PrepaintCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub tree: &'a mut crate::tree::UiTree<H>,
    pub node: NodeId,
    pub window: Option<AppWindowId>,
    pub bounds: Rect,
    pub scale_factor: f32,
}

impl<'a, H: UiHost> PrepaintCx<'a, H> {
    pub fn set_output<T: std::any::Any>(&mut self, value: T) {
        self.tree.set_prepaint_output(self.node, value);
    }

    pub fn output<T: std::any::Any>(&mut self) -> Option<&T> {
        self.tree.prepaint_output(self.node)
    }

    pub fn output_mut<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.tree.prepaint_output_mut(self.node)
    }

    /// Mark an invalidation on `node` for the next frame.
    ///
    /// Prefer `Invalidation::Paint` / `Invalidation::HitTest` here. Invalidating `Layout` from
    /// prepaint is allowed but can easily introduce avoidable churn.
    pub fn invalidate(&mut self, node: NodeId, kind: Invalidation) {
        self.tree
            .debug_record_prepaint_action(crate::tree::UiDebugPrepaintAction {
                node: self.node,
                target: Some(node),
                kind: crate::tree::UiDebugPrepaintActionKind::Invalidate,
                invalidation: Some(kind),
                element: None,
                virtual_list_window_shift_kind: None,
                virtual_list_window_shift_reason: None,
                frame_id: self.app.frame_id(),
            });
        self.tree.invalidate_with_detail(
            node,
            kind,
            crate::tree::UiDebugInvalidationDetail::Unknown,
        );
    }

    /// Mark an invalidation on the current node for the next frame.
    pub fn invalidate_self(&mut self, kind: Invalidation) {
        self.invalidate(self.node, kind);
    }

    /// Request a window redraw (one-shot).
    ///
    /// Use this for one-shot updates after prepaint-driven state changes.
    pub fn request_redraw(&mut self) {
        self.tree
            .debug_record_prepaint_action(crate::tree::UiDebugPrepaintAction {
                node: self.node,
                target: None,
                kind: crate::tree::UiDebugPrepaintActionKind::RequestRedraw,
                invalidation: None,
                element: None,
                virtual_list_window_shift_kind: None,
                virtual_list_window_shift_reason: None,
                frame_id: self.app.frame_id(),
            });
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Request the next animation frame for this window.
    ///
    /// Prefer this over `request_redraw()` when you need frame-driven progression (animations,
    /// progressive rendering). This also sets `Invalidation::Paint` for the current node so paint
    /// caching cannot skip widget `paint()` on the next frame.
    pub fn request_animation_frame(&mut self) {
        self.tree
            .debug_record_prepaint_action(crate::tree::UiDebugPrepaintAction {
                node: self.node,
                target: Some(self.node),
                kind: crate::tree::UiDebugPrepaintActionKind::RequestAnimationFrame,
                invalidation: Some(Invalidation::Paint),
                element: None,
                virtual_list_window_shift_kind: None,
                virtual_list_window_shift_reason: None,
                frame_id: self.app.frame_id(),
            });
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate_with_source_and_detail(
            self.node,
            Invalidation::Paint,
            crate::tree::UiDebugInvalidationSource::Notify,
            crate::tree::UiDebugInvalidationDetail::AnimationFrameRequest,
        );
        let Some(window) = self.window else {
            return;
        };
        self.app.push_effect(Effect::RequestAnimationFrame(window));
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
    pub children_render_transform: Option<Transform2D>,
    pub services: &'a mut dyn UiServices,
    pub observe_model: &'a mut dyn FnMut(ModelId, Invalidation),
    pub observe_global: &'a mut dyn FnMut(TypeId, Invalidation),
    pub scene: &'a mut Scene,
}

impl<'a, H: UiHost> PaintCx<'a, H> {
    pub fn prepaint_output<T: std::any::Any>(&mut self) -> Option<&T> {
        self.tree.prepaint_output(self.node)
    }

    pub fn prepaint_output_mut<T: std::any::Any>(&mut self) -> Option<&mut T> {
        self.tree.prepaint_output_mut(self.node)
    }

    pub fn theme(&mut self) -> &Theme {
        self.observe_global::<Theme>(Invalidation::Paint);
        Theme::global(&*self.app)
    }

    /// Request a window redraw (one-shot).
    ///
    /// Use this for one-shot updates. For frame-driven updates that must repaint continuously,
    /// use `request_animation_frame()`.
    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.app.request_redraw(window);
    }

    /// Request the next animation frame for this window.
    ///
    /// Prefer this over `request_redraw()` when you need frame-driven progression (animations,
    /// progressive rendering). This also sets `Invalidation::Paint` for the current node so paint
    /// caching cannot skip widget `paint()` on the next frame.
    ///
    /// This is a one-shot request. Callers should re-issue `request_animation_frame()` each frame
    /// while it remains active.
    pub fn request_animation_frame(&mut self) {
        // Ensure animation-frame requests trigger a paint pass even when paint caching is enabled.
        self.tree.invalidate_with_source_and_detail(
            self.node,
            Invalidation::Paint,
            crate::tree::UiDebugInvalidationSource::Notify,
            crate::tree::UiDebugInvalidationDetail::AnimationFrameRequest,
        );
        let Some(window) = self.window else {
            return;
        };
        self.app.push_effect(Effect::RequestAnimationFrame(window));
    }

    /// Request the next animation frame for this window without marking the nearest cache root as
    /// dirty.
    ///
    /// This is intended for paint-only chrome (hover fades, drag indicators, caret blink) that
    /// must repaint every frame but should remain structurally reusable under view caching.
    pub fn request_animation_frame_paint_only(&mut self) {
        self.tree.invalidate_with_source_and_detail(
            self.node,
            Invalidation::Paint,
            crate::tree::UiDebugInvalidationSource::Other,
            crate::tree::UiDebugInvalidationDetail::AnimationFrameRequest,
        );
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
        let child_transform = self.children_render_transform;
        if let Some(transform) = child_transform {
            self.scene
                .push(fret_core::SceneOp::PushTransform { transform });
        }

        let accumulated = child_transform
            .map(|t| self.accumulated_transform.compose(t))
            .unwrap_or(self.accumulated_transform);

        self.tree.paint_node(
            self.app,
            self.services,
            child,
            bounds,
            self.scene,
            self.scale_factor,
            accumulated,
        );

        if child_transform.is_some() {
            self.scene.push(fret_core::SceneOp::PopTransform);
        }
    }

    /// Paint all child nodes using their last computed layout bounds.
    ///
    /// This is the default behavior of `Widget::paint()`.
    pub fn paint_children(&mut self) {
        for &child in self.children {
            if let Some(bounds) = self.child_bounds(child) {
                self.paint(child, bounds);
            } else {
                self.paint(child, self.bounds);
            }
        }
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
    pub(crate) test_id: &'a mut Option<String>,
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

    pub fn set_test_id(&mut self, id: impl Into<String>) {
        *self.test_id = Some(id.into());
    }

    pub fn clear_test_id(&mut self) {
        *self.test_id = None;
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
    /// Capture-phase event dispatch (root → target).
    ///
    /// Default is no-op so existing widgets keep their current bubble-only behavior.
    fn event_capture(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}

    /// Observer-phase event dispatch (`InputDispatchPhase::Preview`).
    ///
    /// This pass must not mutate input routing state (focus / capture / propagation / default
    /// actions). It exists to support outside-press dismissal and click-through overlay policies
    /// (ADR 0069).
    fn event_observer(&mut self, _cx: &mut ObserverCx<'_, H>, _event: &Event) {}

    fn debug_type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn event(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}
    fn command(&mut self, _cx: &mut CommandCx<'_, H>, _command: &CommandId) -> bool {
        false
    }

    /// Pure query: does this node participate in availability for `command`?
    fn command_availability(
        &self,
        _cx: &mut CommandAvailabilityCx<'_, H>,
        _command: &CommandId,
    ) -> CommandAvailability {
        CommandAvailability::NotHandled
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
    /// Optional affine transform applied to children only (not to this node's own bounds).
    ///
    /// This is intended for behaviors like scrolling where the viewport bounds are fixed, but the
    /// content subtree is translated.
    ///
    /// The transform is expressed in the same coordinate space as `bounds` (logical px,
    /// window-local).
    fn children_render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        None
    }
    /// Optional cursor icon request for a pointer position.
    ///
    /// This is a pure query used to build an interaction stream that can be reused on cache-hit
    /// frames (ADR 0182). Prefer this over setting cursor icons via pointer-move event handlers
    /// when the cursor choice is a function of the current input state only.
    ///
    /// The provided `position` is already mapped into this node's coordinate space (including
    /// ancestor `render_transform` and `children_render_transform`), matching what the widget sees
    /// during pointer event dispatch.
    fn cursor_icon_at(
        &self,
        _bounds: Rect,
        _position: Point,
        _input_ctx: &fret_runtime::InputContext,
    ) -> Option<fret_core::CursorIcon> {
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
    /// Optional synchronization hook for declarative `InteractivityGate` nodes.
    ///
    /// Declarative `InteractivityGate` is allowed to short-circuit layout when `present == false`
    /// (display-none behavior). In those frames the layout engine may skip calling `layout()` for
    /// the gate node, leaving cached widget gates stale. Declarative host widgets can override
    /// this hook so the mount pipeline can keep semantics/hit-test traversal consistent even when
    /// layout is skipped.
    fn sync_interactivity_gate(&mut self, _present: bool, _interactive: bool) {}
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

    /// Optional platform-facing text input snapshot for the focused widget.
    ///
    /// This exists to support editor-grade IME and accessibility bridges that need UTF-16 ranges
    /// and an IME cursor anchor, without depending on widget internals.
    ///
    /// Coordinate model: UTF-16 code units over the widget's "composed view" (base text with the
    /// active preedit spliced at the caret).
    fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
        None
    }

    /// Returns the focused selection range (UTF-16 code units over the composed view).
    fn platform_text_input_selected_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        None
    }

    /// Returns the marked (preedit) range (UTF-16 code units over the composed view).
    fn platform_text_input_marked_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        None
    }

    fn platform_text_input_text_for_range_utf16(
        &self,
        _range: fret_runtime::Utf16Range,
    ) -> Option<String> {
        None
    }

    fn platform_text_input_bounds_for_range_utf16(
        &mut self,
        _cx: &mut PlatformTextInputCx<'_, H>,
        _range: fret_runtime::Utf16Range,
    ) -> Option<Rect> {
        None
    }

    fn platform_text_input_character_index_for_point_utf16(
        &mut self,
        _cx: &mut PlatformTextInputCx<'_, H>,
        _point: Point,
    ) -> Option<u32> {
        None
    }

    fn platform_text_input_replace_text_in_range_utf16(
        &mut self,
        _cx: &mut PlatformTextInputCx<'_, H>,
        _range: fret_runtime::Utf16Range,
        _text: &str,
    ) -> bool {
        false
    }

    fn platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut self,
        _cx: &mut PlatformTextInputCx<'_, H>,
        _range: fret_runtime::Utf16Range,
        _text: &str,
        _marked: Option<fret_runtime::Utf16Range>,
    ) -> bool {
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
    /// Prepaint hook invoked after layout, before paint.
    ///
    /// Default is no-op so existing widgets keep their current behavior.
    fn prepaint(&mut self, _cx: &mut PrepaintCx<'_, H>) {}
    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.paint_children();
    }
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

pub struct PlatformTextInputCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
    pub window: Option<AppWindowId>,
    pub node: NodeId,
    pub bounds: Rect,
    pub scale_factor: f32,
}

impl<'a, H: UiHost> PlatformTextInputCx<'a, H> {
    pub fn theme(&self) -> &Theme {
        Theme::global(&*self.app)
    }
}
