use std::sync::Arc;

use fret_canvas::view::{
    DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D, wheel_zoom_factor,
};
use fret_core::{Modifiers, MouseButton, Point, Px};
use fret_runtime::Model;
use fret_ui::action::{OnPinchGesture, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use crate::declarative::canvas_surface::{CanvasSurfacePanelProps, canvas_surface_panel};
use crate::declarative::controllable_state::use_controllable_model;
use crate::declarative::model_watch::ModelWatchExt;

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
    props: PanZoomCanvasSurfacePanelProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
) -> AnyElement {
    let PanZoomCanvasSurfacePanelProps {
        pointer_region,
        canvas,
        preset,
        view,
        default_view,
        min_zoom,
        max_zoom,
        wheel_zoom,
        pinch_zoom_speed,
        pan_button,
        on_pointer_down,
        on_pointer_move,
        on_pointer_up,
        on_wheel,
        on_pinch_gesture,
    } = props;

    let view = use_controllable_model(cx, view, || default_view).model();
    let view_value = cx
        .watch_model(&view)
        .paint()
        .copied()
        .unwrap_or(default_view);

    let drag: Model<Option<DragState>> =
        use_controllable_model(cx, None::<Model<Option<DragState>>>, || None).model();

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
                let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0);
                view.pan.x = Px(view.pan.x.0 + dx / zoom);
                view.pan.y = Px(view.pan.y.0 + dy / zoom);
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

    let view_c = view.clone();
    let on_pinch_zoom: OnPinchGesture = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              pinch: fret_ui::action::PinchGestureCx| {
            if !pinch.delta.is_finite() {
                return false;
            }

            let speed = if pinch_zoom_speed.is_finite() {
                pinch_zoom_speed.max(0.0)
            } else {
                1.0
            };

            // Match existing ecosystem behavior (e.g. node graph): factor = 1 + delta*speed.
            let delta = pinch.delta.clamp(-0.95, 10.0);
            let factor = (1.0 + delta * speed).max(0.01);

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
                tmp.zoom_about_screen_point(bounds, pinch.position, new_zoom);
                *view = tmp;
            });

            host.request_redraw(action_cx.window);
            true
        },
    );

    let paint_view = view_value;
    let paint = move |painter: &mut CanvasPainter<'_>| {
        let raster_scale_factor =
            painter.scale_factor() * PanZoom2D::sanitize_zoom(paint_view.zoom, 1.0);
        let cx2 = PanZoomCanvasPaintCx {
            view: paint_view,
            raster_scale_factor,
        };

        if let Some(transform) = paint_view.render_transform(painter.bounds()) {
            painter.with_transform(transform, |painter| paint(painter, cx2));
        } else {
            paint(painter, cx2);
        }
    };

    let mut surface = CanvasSurfacePanelProps::default();
    surface.pointer_region = pointer_region;
    surface.canvas = canvas;
    surface.on_pointer_down = Some(on_pointer_down_pan);
    surface.on_pointer_move = Some(on_pointer_move_pan);
    surface.on_pointer_up = Some(on_pointer_up_pan);
    surface.on_wheel = Some(on_wheel_zoom);
    surface.on_pinch_gesture = Some(on_pinch_zoom);

    // Forward any extra handlers as a fallback when pan/zoom does not consume.
    //
    // This is intentionally simple: callers that need full tool-mode routing should use the
    // policy-free `canvas_surface_panel` substrate directly.
    if on_pointer_down.is_some()
        || on_pointer_move.is_some()
        || on_pointer_up.is_some()
        || on_wheel.is_some()
        || on_pinch_gesture.is_some()
    {
        let mut next = surface;

        let maybe_down = on_pointer_down.clone();
        if let Some(existing) = next.on_pointer_down.take() {
            next.on_pointer_down = Some(Arc::new(move |host, cx, down| {
                if existing(host, cx, down) {
                    return true;
                }
                maybe_down.as_ref().is_some_and(|h| h(host, cx, down))
            }));
        }

        let maybe_move = on_pointer_move.clone();
        if let Some(existing) = next.on_pointer_move.take() {
            next.on_pointer_move = Some(Arc::new(move |host, cx, mv| {
                if existing(host, cx, mv) {
                    return true;
                }
                maybe_move.as_ref().is_some_and(|h| h(host, cx, mv))
            }));
        }

        let maybe_up = on_pointer_up.clone();
        if let Some(existing) = next.on_pointer_up.take() {
            next.on_pointer_up = Some(Arc::new(move |host, cx, up| {
                if existing(host, cx, up) {
                    return true;
                }
                maybe_up.as_ref().is_some_and(|h| h(host, cx, up))
            }));
        }

        let maybe_wheel = on_wheel.clone();
        if let Some(existing) = next.on_wheel.take() {
            next.on_wheel = Some(Arc::new(move |host, cx, wheel| {
                if existing(host, cx, wheel) {
                    return true;
                }
                maybe_wheel.as_ref().is_some_and(|h| h(host, cx, wheel))
            }));
        }

        let maybe_pinch = on_pinch_gesture.clone();
        if let Some(existing) = next.on_pinch_gesture.take() {
            next.on_pinch_gesture = Some(Arc::new(move |host, cx, pinch| {
                if existing(host, cx, pinch) {
                    return true;
                }
                maybe_pinch.as_ref().is_some_and(|h| h(host, cx, pinch))
            }));
        }

        return canvas_surface_panel(cx, next, paint);
    }

    canvas_surface_panel(cx, surface, paint)
}
