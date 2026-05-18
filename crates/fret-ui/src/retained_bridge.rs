//! Unstable retained-widget bridge for policy-heavy UI (e.g. docking migration).
//!
//! This module is intentionally feature-gated (`unstable-retained-bridge`) and is **not** part of
//! the stable `fret-ui` runtime contract surface (ADR 0066).

use crate::{UiHost, UiTree};
use fret_core::NodeId;
use std::any::Any;
use std::sync::Arc;

pub use crate::text_input::{BoundTextInput, TextInput};
pub use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, Invalidation, LayoutCx,
    MeasureCx, PaintCx, PrepaintCx, SemanticsCx, Widget,
};

type RetainedSubtreeBuildFn<H> = dyn Fn(&mut UiTree<H>) -> NodeId;

/// Extension trait that exposes a feature-gated node creation API for retained widgets.
pub trait UiTreeRetainedExt<H: UiHost> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId;
}

impl<H: UiHost> UiTreeRetainedExt<H> for UiTree<H> {
    fn create_node_retained(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        self.create_node(widget)
    }
}

/// Unstable declarative bridge for hosting retained subtrees inside the element runtime.
///
/// This is intended as a migration aid for policy-heavy ecosystems (docking, node graphs, charts)
/// while the primary authoring direction remains declarative (ADR 0028 / ADR 0039).
#[derive(Clone)]
pub struct RetainedSubtreeFactory {
    inner: Arc<dyn Any>,
}

impl std::fmt::Debug for RetainedSubtreeFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedSubtreeFactory")
            .finish_non_exhaustive()
    }
}

impl RetainedSubtreeFactory {
    pub fn new<H: UiHost + 'static>(f: impl Fn(&mut UiTree<H>) -> NodeId + 'static) -> Self {
        let f: Arc<RetainedSubtreeBuildFn<H>> = Arc::new(f);
        Self { inner: Arc::new(f) }
    }

    pub(crate) fn build<H: UiHost + 'static>(&self, ui: &mut UiTree<H>) -> NodeId {
        let Some(f) = self.inner.downcast_ref::<Arc<RetainedSubtreeBuildFn<H>>>() else {
            if crate::strict_runtime::strict_runtime_enabled() {
                panic!("retained subtree factory type mismatch (host type changed?)");
            }

            tracing::error!(
                "retained subtree factory type mismatch (host type changed?); returning fallback empty widget node"
            );

            struct FallbackWidget;
            impl<H2: UiHost> Widget<H2> for FallbackWidget {}

            return ui.create_node(FallbackWidget);
        };

        (f)(ui)
    }
}

#[derive(Debug, Clone)]
pub struct RetainedSubtreeProps {
    pub layout: crate::element::LayoutStyle,
    pub factory: RetainedSubtreeFactory,
}

impl RetainedSubtreeProps {
    pub fn new<H: UiHost + 'static>(f: impl Fn(&mut UiTree<H>) -> NodeId + 'static) -> Self {
        let mut layout = crate::element::LayoutStyle::default();
        layout.size.width = crate::element::Length::Fill;
        layout.size.height = crate::element::Length::Fill;
        Self {
            layout,
            factory: RetainedSubtreeFactory::new(f),
        }
    }

    pub fn with_layout(mut self, layout: crate::element::LayoutStyle) -> Self {
        self.layout = layout;
        self
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
