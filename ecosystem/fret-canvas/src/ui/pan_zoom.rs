use std::sync::Arc;

use fret_core::scene::Paint;
use fret_core::{
    Corners, DrawOrder, Edges, Modifiers, MouseButton, Point, Px, Rect, SceneOp, Size,
};
use fret_runtime::Model;
use fret_ui::action::{OnPinchGesture, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::ui::canvas_surface::{CanvasSurfacePanelProps, canvas_surface_panel};
use crate::ui::use_controllable_model;
use crate::view::{DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D, wheel_zoom_factor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanZoomInputPreset {
    /// Safe-by-default mapping intended for embedding inside scroll views.
    ///
    /// - Does not consume plain wheel.
    /// - Zooms only when `ctrl || meta` is held.
    /// - Pans via middle-drag.
    #[default]
    DefaultSafe,
    /// Canvas-first mapping for editor/CAD surfaces.
    ///
    /// - Wheel zooms (consumed).
    /// - Pans via middle-drag (by default).
    DesktopCanvasCad,
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct MarqueeDragState {
    button: MouseButton,
    start: Point,
    current: Point,
    modifiers: Modifiers,
    active: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CanvasMarqueeCommitCx {
    /// Marquee rectangle in screen space (window logical px).
    pub rect_screen: Rect,
    /// Marquee rectangle in canvas space (PanZoom canvas units).
    pub rect_canvas: Rect,
    pub modifiers: Modifiers,
}

pub type OnCanvasMarqueeCommit = Arc<
    dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, CanvasMarqueeCommitCx)
        + 'static,
>;

/// Optional filter invoked on pointer down before starting a marquee drag.
///
/// Return `true` to allow starting selection-on-drag, or `false` to defer to other handlers.
///
/// This is useful for XYFlow-style "background-only" marquee semantics:
/// selection should only start when the down event is not within a node subtree.
pub type OnCanvasMarqueeStart = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            fret_ui::action::PointerDownCx,
        ) -> bool
        + 'static,
>;

#[derive(Debug, Clone, Copy)]
pub struct CanvasMarqueeStyle {
    pub fill: fret_core::Color,
    pub border: fret_core::Color,
    pub border_width_px: f32,
}

impl Default for CanvasMarqueeStyle {
    fn default() -> Self {
        Self {
            fill: fret_core::Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.15,
            },
            border: fret_core::Color {
                r: 0.20,
                g: 0.55,
                b: 0.95,
                a: 0.85,
            },
            border_width_px: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct CanvasMarqueeSelectionProps {
    pub enabled: bool,
    /// Start a marquee drag on pointer down + drag (XyFlow `selectionOnDrag` mental model).
    pub selection_on_drag: bool,
    pub button: MouseButton,
    pub min_drag_distance_px: f32,
    pub style: CanvasMarqueeStyle,
    /// Optional predicate that decides whether the marquee gesture should start.
    ///
    /// When this returns `false`, the canvas surface does not capture the pointer and does not
    /// start marquee selection.
    pub start_filter: Option<OnCanvasMarqueeStart>,
    pub on_commit: Option<OnCanvasMarqueeCommit>,
}

impl Default for CanvasMarqueeSelectionProps {
    fn default() -> Self {
        Self {
            enabled: true,
            selection_on_drag: true,
            button: MouseButton::Left,
            min_drag_distance_px: 3.0,
            style: CanvasMarqueeStyle::default(),
            start_filter: None,
            on_commit: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct EditorPanZoomCanvasWithMarqueeProps {
    pub pan_zoom: PanZoomCanvasSurfacePanelProps,
    pub marquee: CanvasMarqueeSelectionProps,
}

fn rect_from_points(a: Point, b: Point) -> Rect {
    let x0 = a.x.0.min(b.x.0);
    let y0 = a.y.0.min(b.y.0);
    let x1 = a.x.0.max(b.x.0);
    let y1 = a.y.0.max(b.y.0);
    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    )
}

/// Editor preset + XyFlow-style selection-on-drag (marquee) overlay.
///
/// This is a policy-light recipe that:
/// - reuses the existing editor pan/zoom mapping (wheel pan + ctrl/cmd wheel zoom + MMB drag pan),
/// - optionally emits a marquee rectangle while dragging, and
/// - reports commit payloads (screen + canvas rect) via an explicit callback.
///
/// Selection semantics remain app-owned: callers decide how to interpret the committed rect.
#[track_caller]
pub fn editor_pan_zoom_canvas_surface_panel_with_marquee_selection<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: EditorPanZoomCanvasWithMarqueeProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
) -> AnyElement {
    let marquee_enabled = props.marquee.enabled && props.marquee.selection_on_drag;
    let marquee_button = props.marquee.button;
    let min_drag = props.marquee.min_drag_distance_px.max(0.0);
    let marquee_style = props.marquee.style;
    let start_filter = props.marquee.start_filter.clone();
    let on_commit = props.marquee.on_commit.take();

    let view_model = use_controllable_model(cx, props.pan_zoom.view.take(), || {
        props.pan_zoom.default_view
    })
    .model();
    props.pan_zoom.view = Some(view_model.clone());

    let drag_state: Model<Option<MarqueeDragState>> =
        use_controllable_model(cx, None, || None).model();
    let drag_value = cx
        .get_model_copied(&drag_state, Invalidation::Paint)
        .unwrap_or(None);

    if marquee_enabled {
        let drag_c = drag_state.clone();
        let on_down: OnPointerDown = Arc::new(
            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  down: fret_ui::action::PointerDownCx| {
                if down.button != marquee_button {
                    return false;
                }

                if let Some(start_filter) = start_filter.as_ref()
                    && !start_filter(host, action_cx, down)
                {
                    return false;
                }

                host.capture_pointer();
                let _ = host.models_mut().update(&drag_c, |st| {
                    *st = Some(MarqueeDragState {
                        button: down.button,
                        start: down.position,
                        current: down.position,
                        modifiers: down.modifiers,
                        active: false,
                    });
                });
                host.request_redraw(action_cx.window);
                true
            },
        );
        props.pan_zoom.on_pointer_down = Some(match props.pan_zoom.on_pointer_down.take() {
            None => on_down,
            Some(prev) => {
                Arc::new(move |host, cx, down| prev(host, cx, down) || on_down(host, cx, down))
            }
        });

        let drag_c = drag_state.clone();
        let on_move: OnPointerMove = Arc::new(
            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  mv: fret_ui::action::PointerMoveCx| {
                let mut drag = host.models_mut().read(&drag_c, |st| *st).ok().flatten();
                let Some(mut drag_state) = drag.take() else {
                    return false;
                };

                drag_state.current = mv.position;
                if !drag_state.active {
                    let dx = drag_state.current.x.0 - drag_state.start.x.0;
                    let dy = drag_state.current.y.0 - drag_state.start.y.0;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist >= min_drag {
                        drag_state.active = true;
                    }
                }

                let _ = host
                    .models_mut()
                    .update(&drag_c, |st| *st = Some(drag_state));
                host.request_redraw(action_cx.window);
                true
            },
        );
        props.pan_zoom.on_pointer_move = Some(match props.pan_zoom.on_pointer_move.take() {
            None => on_move,
            Some(prev) => Arc::new(move |host, cx, mv| prev(host, cx, mv) || on_move(host, cx, mv)),
        });

        let drag_c = drag_state.clone();
        let view_c = view_model.clone();
        let on_commit = on_commit.clone();
        let on_up: OnPointerUp = Arc::new(
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

                if drag.active
                    && let Some(on_commit) = on_commit.as_ref()
                {
                    let bounds = host.bounds();
                    let view = host
                        .models_mut()
                        .read(&view_c, |v| *v)
                        .ok()
                        .unwrap_or_default();
                    let rect_screen = rect_from_points(drag.start, drag.current);
                    let c0 = view.screen_to_canvas(bounds, rect_screen.origin);
                    let c1 = view.screen_to_canvas(
                        bounds,
                        Point::new(
                            Px(rect_screen.origin.x.0 + rect_screen.size.width.0),
                            Px(rect_screen.origin.y.0 + rect_screen.size.height.0),
                        ),
                    );
                    let rect_canvas = rect_from_points(c0, c1);
                    on_commit(
                        host,
                        action_cx,
                        CanvasMarqueeCommitCx {
                            rect_screen,
                            rect_canvas,
                            modifiers: drag.modifiers,
                        },
                    );
                }

                host.request_redraw(action_cx.window);
                true
            },
        );
        props.pan_zoom.on_pointer_up = Some(match props.pan_zoom.on_pointer_up.take() {
            None => on_up,
            Some(prev) => Arc::new(move |host, cx, up| prev(host, cx, up) || on_up(host, cx, up)),
        });
    }

    let paint_drag = drag_value;
    let paint = move |p: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
        paint(p, paint_cx);

        if !marquee_enabled {
            return;
        }

        let drag = paint_drag;
        let Some(drag) = drag else {
            return;
        };
        if !drag.active {
            return;
        }

        let rect = rect_from_points(drag.start, drag.current);
        if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
            return;
        }

        p.scene().push(SceneOp::Quad {
            order: DrawOrder(1000),
            rect,
            background: Paint::Solid(marquee_style.fill),
            border: Edges::all(Px(marquee_style.border_width_px)),
            border_paint: Paint::Solid(marquee_style.border),
            corner_radii: Corners::all(Px(0.0)),
        });
    };

    editor_pan_zoom_canvas_surface_panel(cx, props.pan_zoom, paint)
}
