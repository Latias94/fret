//! Unstable retained-widget bridge for policy-heavy UI (e.g. docking migration).
//!
//! This module is intentionally feature-gated (`unstable-retained-bridge`) and is **not** part of
//! the stable `fret-ui` runtime contract surface (ADR 0066).

use crate::{UiHost, UiTree};
use fret_core::NodeId;

pub use crate::resizable_panel_group::{ResizablePanelGroupLayout, ResizablePanelGroupStyle};
pub use crate::resize_handle::ResizeHandle;
pub use crate::text_input::{BoundTextInput, TextInput};
pub use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, Invalidation, LayoutCx,
    MeasureCx, PaintCx, SemanticsCx, Widget,
};

/// Extension trait that exposes a feature-gated node creation API for retained widgets.
pub trait UiTreeRetainedExt<H: UiHost> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId;
}

impl<H: UiHost> UiTreeRetainedExt<H> for UiTree<H> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.create_node(widget)
    }
}

/// Unstable mechanism helpers for splitter / panel-group sizing.
pub mod resizable_panel_group {
    use fret_core::{Axis, Point, Px, Rect};

    use crate::resizable_panel_group::{
        ResizablePanelGroupLayout, apply_handle_delta, compute_resizable_panel_group_layout,
        fractions_from_sizes,
    };

    pub fn compute_layout(
        axis: Axis,
        bounds: Rect,
        children_len: usize,
        fractions: &[f32],
        gap: Px,
        hit_thickness: Px,
        min_px: &[Px],
    ) -> ResizablePanelGroupLayout {
        compute_resizable_panel_group_layout(
            axis,
            bounds,
            children_len,
            fractions.to_vec(),
            gap,
            hit_thickness,
            min_px,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn drag_update_fractions(
        axis: Axis,
        bounds: Rect,
        children_len: usize,
        fractions: &[f32],
        handle_ix: usize,
        gap: Px,
        hit_thickness: Px,
        min_px: &[Px],
        grab_offset: f32,
        position: Point,
    ) -> Option<Vec<f32>> {
        if children_len < 2 || handle_ix + 1 >= children_len {
            return None;
        }

        let layout = compute_layout(
            axis,
            bounds,
            children_len,
            fractions,
            gap,
            hit_thickness,
            min_px,
        );
        let old_center = *layout.handle_centers.get(handle_ix)?;

        let axis_pos = match axis {
            Axis::Horizontal => position.x.0,
            Axis::Vertical => position.y.0,
        };

        let desired_center = axis_pos - grab_offset;
        let desired_delta = desired_center - old_center;
        if !desired_delta.is_finite() {
            return None;
        }

        let mut sizes = layout.sizes.clone();
        let actual = apply_handle_delta(handle_ix, desired_delta, &mut sizes, &layout.mins);
        if actual.abs() <= 1.0e-6 {
            return None;
        }
        Some(fractions_from_sizes(&sizes, layout.avail))
    }
}

/// Unstable retained helpers for viewport surfaces (Tier A embedding).
pub mod viewport_surface {
    use fret_core::{
        AppWindowId, Event, MouseButton, PointerEvent, RenderTargetId, ViewportInputEvent,
        ViewportInputKind, ViewportMapping, WindowMetricsService,
    };
    use fret_runtime::Effect;

    use crate::widget::EventCx;
    use crate::{UiHost, widget::Invalidation};

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ViewportInputCapture {
        pub window: AppWindowId,
        pub target: RenderTargetId,
        pub mapping: ViewportMapping,
        pub button: MouseButton,
        pub last_cursor_px: fret_core::Point,
    }

    /// Forwards pointer + wheel events into a viewport surface using `ViewportMapping`.
    ///
    /// This helper mirrors the "capture on pointer down, then clamp moves/up while captured"
    /// pattern used by viewport panels (game views, editor canvases).
    pub fn handle_viewport_surface_input<H: UiHost>(
        cx: &mut EventCx<'_, H>,
        event: &Event,
        target: RenderTargetId,
        mapping: ViewportMapping,
        capture: &mut Option<ViewportInputCapture>,
        focus_on_down: bool,
    ) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let pixels_per_point = cx
            .app
            .global::<WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(window))
            .unwrap_or(1.0);

        match event {
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                pointer_id,
                pointer_type,
                ..
            }) => {
                let kind = ViewportInputKind::PointerDown {
                    button: *button,
                    modifiers: *modifiers,
                    click_count: *click_count,
                };
                let Some(evt) = ViewportInputEvent::from_mapping_window_point(
                    window,
                    target,
                    &mapping,
                    pixels_per_point,
                    *pointer_id,
                    *pointer_type,
                    *position,
                    kind,
                ) else {
                    return false;
                };

                cx.app.push_effect(Effect::ViewportInput(evt));
                if focus_on_down {
                    cx.request_focus(cx.node);
                }
                *capture = Some(ViewportInputCapture {
                    window,
                    target,
                    mapping,
                    button: *button,
                    last_cursor_px: *position,
                });
                cx.capture_pointer(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
                true
            }
            Event::Pointer(PointerEvent::Move {
                position,
                buttons,
                modifiers,
                pointer_id,
                pointer_type,
                ..
            }) => {
                if let Some(c) = capture
                    && c.window == window
                    && cx.captured == Some(cx.node)
                {
                    c.last_cursor_px = *position;
                    let pixels_per_point = cx
                        .app
                        .global::<WindowMetricsService>()
                        .and_then(|svc| svc.scale_factor(c.window))
                        .unwrap_or(1.0);
                    let evt = ViewportInputEvent::from_mapping_window_point_clamped(
                        c.window,
                        c.target,
                        &c.mapping,
                        pixels_per_point,
                        *pointer_id,
                        *pointer_type,
                        *position,
                        ViewportInputKind::PointerMove {
                            buttons: *buttons,
                            modifiers: *modifiers,
                        },
                    );
                    cx.app.push_effect(Effect::ViewportInput(evt));
                    cx.stop_propagation();
                    return true;
                }

                let Some(evt) = ViewportInputEvent::from_mapping_window_point(
                    window,
                    target,
                    &mapping,
                    pixels_per_point,
                    *pointer_id,
                    *pointer_type,
                    *position,
                    ViewportInputKind::PointerMove {
                        buttons: *buttons,
                        modifiers: *modifiers,
                    },
                ) else {
                    return false;
                };
                if let Some(c) = capture {
                    c.last_cursor_px = *position;
                }
                cx.app.push_effect(Effect::ViewportInput(evt));
                cx.stop_propagation();
                true
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                is_click,
                click_count,
                pointer_id,
                pointer_type,
                ..
            }) => {
                let Some(c) = *capture else {
                    return false;
                };
                if c.window != window || c.button != *button {
                    return false;
                }

                let pixels_per_point = cx
                    .app
                    .global::<WindowMetricsService>()
                    .and_then(|svc| svc.scale_factor(c.window))
                    .unwrap_or(1.0);
                let evt = ViewportInputEvent::from_mapping_window_point_clamped(
                    c.window,
                    c.target,
                    &c.mapping,
                    pixels_per_point,
                    *pointer_id,
                    *pointer_type,
                    *position,
                    ViewportInputKind::PointerUp {
                        button: *button,
                        modifiers: *modifiers,
                        is_click: *is_click,
                        click_count: *click_count,
                    },
                );
                cx.app.push_effect(Effect::ViewportInput(evt));

                *capture = None;
                if cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
                true
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                pointer_id,
                pointer_type,
                ..
            }) => {
                let Some(evt) = ViewportInputEvent::from_mapping_window_point(
                    window,
                    target,
                    &mapping,
                    pixels_per_point,
                    *pointer_id,
                    *pointer_type,
                    *position,
                    ViewportInputKind::Wheel {
                        delta: *delta,
                        modifiers: *modifiers,
                    },
                ) else {
                    return false;
                };
                if let Some(c) = capture {
                    c.last_cursor_px = *position;
                }
                cx.app.push_effect(Effect::ViewportInput(evt));
                cx.stop_propagation();
                true
            }
            Event::PointerCancel(e) => {
                let position = e
                    .position
                    .or_else(|| capture.as_ref().map(|c| c.last_cursor_px))
                    .unwrap_or_else(|| mapping.map().draw_rect.origin);
                let evt = ViewportInputEvent::from_mapping_window_point_clamped(
                    window,
                    target,
                    &mapping,
                    pixels_per_point,
                    e.pointer_id,
                    e.pointer_type,
                    position,
                    ViewportInputKind::PointerCancel {
                        buttons: e.buttons,
                        modifiers: e.modifiers,
                        reason: e.reason,
                    },
                );
                cx.app.push_effect(Effect::ViewportInput(evt));

                *capture = None;
                if cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }
}
