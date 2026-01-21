use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, Point};
use fret_runtime::Model;
use fret_ui::action::{OnPinchGesture, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::ui::canvas_surface::{CanvasSurfacePanelProps, canvas_surface_panel};
use crate::ui::use_controllable_model;
use crate::view::{DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D, wheel_zoom_factor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanZoomInputPreset {
    /// Safe-by-default mapping intended for embedding inside scroll views.
    ///
    /// - Does not consume plain wheel.
    /// - Zooms only when `ctrl || meta` is held.
    /// - Pans via middle-drag.
    DefaultSafe,
    /// Canvas-first mapping for editor/CAD surfaces.
    ///
    /// - Wheel zooms (consumed).
    /// - Pans via middle-drag (by default).
    DesktopCanvasCad,
}

impl Default for PanZoomInputPreset {
    fn default() -> Self {
        Self::DefaultSafe
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PanZoomWheelZoomConfig {
    /// Base zoom factor per `step` of wheel delta.
    pub base: f32,
    /// Wheel delta step that maps to `base` (desktop defaults are often around 120).
    pub step: f32,
    /// Extra speed multiplier applied to the exponent.
    pub speed: f32,
}

impl Default for PanZoomWheelZoomConfig {
    fn default() -> Self {
        Self {
            base: DEFAULT_WHEEL_ZOOM_BASE,
            step: DEFAULT_WHEEL_ZOOM_STEP,
            speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PanZoomCanvasPaintCx {
    pub view: PanZoom2D,
    /// Suggested scale factor for hosted raster caches (text shaping, tessellation).
    pub raster_scale_factor: f32,
}

#[derive(Clone)]
pub struct PanZoomCanvasSurfacePanelProps {
    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,

    pub preset: PanZoomInputPreset,

    /// Optional externally-owned view model (controlled). When `None`, an internal model is used.
    pub view: Option<Model<PanZoom2D>>,
    pub default_view: PanZoom2D,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub wheel_zoom: PanZoomWheelZoomConfig,
    pub pinch_zoom_speed: f32,
    pub pan_button: MouseButton,

    /// Optional extra handlers invoked when the pan/zoom policy does not consume the event.
    pub on_pointer_down: Option<OnPointerDown>,
    pub on_pointer_move: Option<OnPointerMove>,
    pub on_pointer_up: Option<OnPointerUp>,
    pub on_wheel: Option<OnWheel>,
    pub on_pinch_gesture: Option<OnPinchGesture>,
}

impl Default for PanZoomCanvasSurfacePanelProps {
    fn default() -> Self {
        let mut pointer_region = PointerRegionProps::default();
        pointer_region.layout.size.width = Length::Fill;
        pointer_region.layout.size.height = Length::Fill;

        Self {
            pointer_region,
            canvas: CanvasProps::default(),
            preset: PanZoomInputPreset::DefaultSafe,
            view: None,
            default_view: PanZoom2D::default(),
            min_zoom: 0.05,
            max_zoom: 64.0,
            wheel_zoom: PanZoomWheelZoomConfig::default(),
            pinch_zoom_speed: 1.0,
            pan_button: MouseButton::Middle,
            on_pointer_down: None,
            on_pointer_move: None,
            on_pointer_up: None,
            on_wheel: None,
            on_pinch_gesture: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DragState {
    button: MouseButton,
    last_pos: Point,
}

fn zoom_modifier_active(preset: PanZoomInputPreset, modifiers: Modifiers) -> bool {
    match preset {
        PanZoomInputPreset::DefaultSafe => modifiers.ctrl || modifiers.meta,
        PanZoomInputPreset::DesktopCanvasCad => true,
    }
}

#[track_caller]
pub fn pan_zoom_canvas_surface_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: PanZoomCanvasSurfacePanelProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
) -> AnyElement {
    let view = use_controllable_model(cx, props.view.take(), || props.default_view).model();
    props.view = Some(view.clone());

    let view_value = cx
        .get_model_copied(&view, Invalidation::Paint)
        .unwrap_or(props.default_view);

    let drag: Model<Option<DragState>> = use_controllable_model(cx, None, || None).model();

    let pan_button = props.pan_button;
    let drag_c = drag.clone();
    let on_pointer_down_pan: OnPointerDown = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              down: fret_ui::action::PointerDownCx| {
            if down.button != pan_button {
                return false;
            }

            host.capture_pointer();
            let _ = host.models_mut().update(&drag_c, |st| {
                *st = Some(DragState {
                    button: down.button,
                    last_pos: down.position,
                });
            });
            host.request_redraw(action_cx.window);
            true
        },
    );

    let view_c = view.clone();
    let drag_c = drag.clone();
    let on_pointer_move_pan: OnPointerMove = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              mv: fret_ui::action::PointerMoveCx| {
            let drag = host.models_mut().read(&drag_c, |st| *st).ok().flatten();
            let Some(drag) = drag else {
                return false;
            };

            let dx = mv.position.x.0 - drag.last_pos.x.0;
            let dy = mv.position.y.0 - drag.last_pos.y.0;

            let _ = host.models_mut().update(&view_c, |view| {
                view.pan_by_screen_delta(dx, dy);
            });
            let _ = host.models_mut().update(&drag_c, |st| {
                if let Some(st) = st.as_mut() {
                    st.last_pos = mv.position;
                }
            });

            host.request_redraw(action_cx.window);
            true
        },
    );

    let drag_c = drag.clone();
    let on_pointer_up_pan: OnPointerUp = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              up: fret_ui::action::PointerUpCx| {
            let drag = host.models_mut().read(&drag_c, |st| *st).ok().flatten();
            let Some(drag) = drag else {
                return false;
            };
            if up.button != drag.button {
                return false;
            }

            host.release_pointer_capture();
            let _ = host.models_mut().update(&drag_c, |st| *st = None);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let preset = props.preset;
    let wheel_zoom = props.wheel_zoom;
    let min_zoom = props.min_zoom;
    let max_zoom = props.max_zoom;
    let view_c = view.clone();
    let on_wheel_zoom: OnWheel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              wheel: fret_ui::action::WheelCx| {
            if !zoom_modifier_active(preset, wheel.modifiers) {
                return false;
            }

            let Some(factor) = wheel_zoom_factor(
                wheel.delta.y.0,
                wheel_zoom.base,
                wheel_zoom.step,
                wheel_zoom.speed,
            ) else {
                return false;
            };

            let zoom = host
                .models_mut()
                .read(&view_c, |view| view.zoom)
                .ok()
                .unwrap_or(1.0);
            let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
            let new_zoom = (zoom * factor).clamp(min_zoom, max_zoom);
            if (new_zoom - zoom).abs() <= 1.0e-9 {
                return false;
            }

            let bounds = host.bounds();
            let _ = host.models_mut().update(&view_c, |view| {
                let mut tmp = *view;
                tmp.zoom_about_screen_point(bounds, wheel.position, new_zoom);
                *view = tmp;
            });

            host.request_redraw(action_cx.window);
            true
        },
    );

    let pinch_zoom_speed = props.pinch_zoom_speed;
    let view_c = view.clone();
    let on_pinch_zoom: OnPinchGesture = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              pinch: fret_ui::action::PinchGestureCx| {
            if !pinch.delta.is_finite() {
                return false;
            }
            let delta = pinch.delta * pinch_zoom_speed;
            if delta.abs() <= 1.0e-9 {
                return false;
            }

            let zoom = host
                .models_mut()
                .read(&view_c, |view| view.zoom)
                .ok()
                .unwrap_or(1.0);
            let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
            let factor = (1.0 + delta).max(1.0e-6);
            let new_zoom = (zoom * factor).clamp(min_zoom, max_zoom);
            if (new_zoom - zoom).abs() <= 1.0e-9 {
                return false;
            }

            let bounds = host.bounds();
            let _ = host.models_mut().update(&view_c, |view| {
                let mut tmp = *view;
                tmp.zoom_about_screen_point(bounds, pinch.position, new_zoom);
                *view = tmp;
            });

            host.request_redraw(action_cx.window);
            true
        },
    );

    let mut surface = CanvasSurfacePanelProps {
        pointer_region: props.pointer_region,
        canvas: props.canvas,
        on_pointer_down: Some(on_pointer_down_pan),
        on_pointer_move: Some(on_pointer_move_pan),
        on_pointer_up: Some(on_pointer_up_pan),
        on_wheel: Some(on_wheel_zoom),
        on_pinch_gesture: Some(on_pinch_zoom),
    };

    if let Some(on_pointer_down) = props.on_pointer_down.take() {
        let inner = surface
            .on_pointer_down
            .take()
            .expect("default pointer down");
        surface.on_pointer_down = Some(Arc::new(move |host, cx, down| {
            let used = inner(host, cx, down);
            used || on_pointer_down(host, cx, down)
        }));
    }
    if let Some(on_pointer_move) = props.on_pointer_move.take() {
        let inner = surface
            .on_pointer_move
            .take()
            .expect("default pointer move");
        surface.on_pointer_move = Some(Arc::new(move |host, cx, mv| {
            let used = inner(host, cx, mv);
            used || on_pointer_move(host, cx, mv)
        }));
    }
    if let Some(on_pointer_up) = props.on_pointer_up.take() {
        let inner = surface.on_pointer_up.take().expect("default pointer up");
        surface.on_pointer_up = Some(Arc::new(move |host, cx, up| {
            let used = inner(host, cx, up);
            used || on_pointer_up(host, cx, up)
        }));
    }
    if let Some(on_wheel) = props.on_wheel.take() {
        let inner = surface.on_wheel.take().expect("default wheel");
        surface.on_wheel = Some(Arc::new(move |host, cx, wheel| {
            let used = inner(host, cx, wheel);
            used || on_wheel(host, cx, wheel)
        }));
    }
    if let Some(on_pinch) = props.on_pinch_gesture.take() {
        let inner = surface
            .on_pinch_gesture
            .take()
            .expect("default pinch gesture");
        surface.on_pinch_gesture = Some(Arc::new(move |host, cx, pinch| {
            let used = inner(host, cx, pinch);
            used || on_pinch(host, cx, pinch)
        }));
    }

    canvas_surface_panel(cx, surface, move |p| {
        let paint_cx = PanZoomCanvasPaintCx {
            view: view_value,
            raster_scale_factor: view_value.zoom.max(1.0e-6) * p.scale_factor(),
        };
        paint(p, paint_cx);
    })
}

/// A common editor-friendly preset:
/// - Wheel pans (consumed).
/// - Ctrl/Cmd + wheel zooms about the pointer (consumed by the base pan/zoom policy).
/// - Middle-drag pans.
///
/// This is intentionally conservative for embedding:
/// - plain wheel does not zoom (to avoid fighting scroll containers),
/// - zoom stays behind the modifier gate.
#[track_caller]
pub fn editor_pan_zoom_canvas_surface_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: PanZoomCanvasSurfacePanelProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
) -> AnyElement {
    let view = use_controllable_model(cx, props.view.take(), || props.default_view).model();
    props.view = Some(view.clone());

    props.preset = PanZoomInputPreset::DefaultSafe;
    props.pan_button = MouseButton::Middle;

    let view_pan = view.clone();
    let on_wheel_pan: OnWheel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              wheel: fret_ui::action::WheelCx| {
            if wheel.modifiers.ctrl || wheel.modifiers.meta {
                return false;
            }

            let dx = wheel.delta.x.0;
            let dy = wheel.delta.y.0;
            if !dx.is_finite() || !dy.is_finite() || (dx.abs() <= 1.0e-9 && dy.abs() <= 1.0e-9) {
                return false;
            }

            let _ = host.models_mut().update(&view_pan, |view| {
                let mut tmp = *view;
                tmp.pan_by_screen_delta(-dx, -dy);
                *view = tmp;
            });

            host.request_redraw(action_cx.window);
            true
        },
    );

    if let Some(extra) = props.on_wheel.take() {
        let inner = on_wheel_pan.clone();
        props.on_wheel = Some(Arc::new(move |host, cx, wheel| {
            let used = inner(host, cx, wheel);
            used || extra(host, cx, wheel)
        }));
    } else {
        props.on_wheel = Some(on_wheel_pan);
    }

    pan_zoom_canvas_surface_panel(cx, props, paint)
}
