use std::sync::{Arc, Mutex};

use fret_core::scene::{ColorSpace, GradientStop, LinearGradient, MAX_STOPS, Paint, TileMode};
use fret_core::{Color, Corners, Edges, MouseButton, Point, PointerId, Px, Rect};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::canvas::OnCanvasPaint;
use fret_ui::element::{
    AnyElement, CanvasProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy)]
pub(super) struct PreviewStop {
    pub(super) id: fret_ui::ItemKey,
    pub(super) position: f32,
    pub(super) color: Color,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct GradientPreviewState {
    pub(super) dragging: bool,
    pub(super) pointer_id: Option<PointerId>,
    pub(super) active_stop: Option<fret_ui::ItemKey>,
}

pub(super) fn gradient_preview_canvas<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    angle_deg: f64,
    stops: Vec<PreviewStop>,
    height: Px,
    active_stop: Option<fret_ui::ItemKey>,
    preview_state: Arc<Mutex<GradientPreviewState>>,
    stop_models: Arc<[(fret_ui::ItemKey, Model<f64>)]>,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);

    let state_for_down = preview_state.clone();
    let state_for_move = preview_state.clone();
    let state_for_paint = preview_state.clone();

    cx.pressable(
        PressableProps {
            enabled,
            layout,
            a11y: PressableA11y {
                label: Some(Arc::from("Gradient preview")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, _st| {
            let stops_for_hit = stops.clone();
            let stop_models_for_hit = stop_models.clone();

            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if !enabled {
                    return PressablePointerDownResult::Continue;
                }
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }

                let bounds = host.bounds();
                let width = bounds.size.width.0.max(1.0);
                let x = down.position_local.x.0;
                let x = x.clamp(0.0, width);

                let mut best: Option<(f32, fret_ui::ItemKey)> = None;
                for s in stops_for_hit.iter() {
                    let sx = s.position.clamp(0.0, 1.0) * width;
                    let d = (sx - x).abs();
                    if best.is_none() || d < best.unwrap().0 {
                        best = Some((d, s.id));
                    }
                }

                let Some((dist, stop_id)) = best else {
                    return PressablePointerDownResult::Continue;
                };

                // Keep the hit target forgiving; preview is a compact strip.
                if dist > 12.0 {
                    return PressablePointerDownResult::Continue;
                }

                let t = (x / width).clamp(0.0, 1.0) as f64;
                if let Some((_id, model)) =
                    stop_models_for_hit.iter().find(|(id, _)| *id == stop_id)
                {
                    let _ = host.models_mut().update(model, |v| *v = t);
                }

                let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                st.dragging = true;
                st.pointer_id = Some(down.pointer_id);
                st.active_stop = Some(stop_id);

                host.request_redraw(action_cx.window);
                PressablePointerDownResult::Continue
            }));

            let stops_for_drag = stop_models.clone();
            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                let mut st = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                if !st.dragging || st.pointer_id != Some(mv.pointer_id) {
                    return false;
                }

                if !mv.buttons.left {
                    st.dragging = false;
                    st.pointer_id = None;
                    return false;
                }

                let Some(stop_id) = st.active_stop else {
                    return false;
                };

                let bounds = host.bounds();
                let width = bounds.size.width.0.max(1.0) as f64;
                let x = (mv.position_local.x.0 as f64).clamp(0.0, width);
                let t = (x / width).clamp(0.0, 1.0);

                if let Some((_id, model)) = stops_for_drag.iter().find(|(id, _)| *id == stop_id) {
                    let _ = host.models_mut().update(model, |v| *v = t);
                    host.request_redraw(action_cx.window);
                    return true;
                }
                false
            }));

            let state_for_up = preview_state.clone();
            cx.pressable_add_on_pointer_up(Arc::new(move |_host, _action_cx, up| {
                let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
                if st.pointer_id == Some(up.pointer_id) {
                    st.dragging = false;
                    st.pointer_id = None;
                }
                PressablePointerUpResult::Continue
            }));

            let on_paint: OnCanvasPaint = Arc::new(move |p| {
                let bounds = p.bounds();
                let rect = Rect {
                    origin: bounds.origin,
                    size: bounds.size,
                };

                let muted = p.theme().color_token("muted");
                let border = p.theme().color_token("border");
                let accent = p.theme().color_token("accent");

                let angle = (angle_deg as f32).to_radians();
                let dx = angle.cos();
                let dy = angle.sin();

                let len = (rect.size.width.0.powi(2) + rect.size.height.0.powi(2))
                    .sqrt()
                    .max(1.0);
                let half = len * 0.5;
                let cx0 = rect.origin.x.0 + rect.size.width.0 * 0.5;
                let cy0 = rect.origin.y.0 + rect.size.height.0 * 0.5;
                let start = Point::new(Px(cx0 - dx * half), Px(cy0 - dy * half));
                let end = Point::new(Px(cx0 + dx * half), Px(cy0 + dy * half));

                let mut fixed = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
                let mut n: u8 = 0;
                for (i, s) in stops.iter().take(MAX_STOPS).enumerate() {
                    fixed[i] = GradientStop::new(s.position.clamp(0.0, 1.0), s.color);
                    n = (i as u8) + 1;
                }
                if n == 0 {
                    fixed[0] = GradientStop::new(0.0, muted);
                    fixed[1] = GradientStop::new(1.0, muted);
                    n = 2;
                }

                let gradient = LinearGradient {
                    start,
                    end,
                    tile_mode: TileMode::Clamp,
                    color_space: ColorSpace::Srgb,
                    stop_count: n,
                    stops: fixed,
                };

                p.scene().push(fret_core::SceneOp::Quad {
                    order: fret_core::DrawOrder(0),
                    rect,
                    background: Paint::LinearGradient(gradient).into(),
                    border: Edges::all(Px(1.0)),
                    border_paint: Paint::Solid(border).into(),
                    corner_radii: Corners::all(Px(6.0)),
                });

                let w = rect.size.width.0.max(1.0);
                let h = rect.size.height.0.max(1.0);

                let marker_d = (h * 0.55).min(12.0).max(6.0);
                let marker_y = rect.origin.y.0 + h - marker_d * 0.5 - 1.0;
                let marker_radius = Px(marker_d * 0.5);

                let active = state_for_paint
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .active_stop
                    .or(active_stop);

                for s in stops.iter() {
                    let x = rect.origin.x.0 + w * s.position.clamp(0.0, 1.0);
                    let marker_rect = Rect {
                        origin: Point::new(Px(x - marker_d * 0.5), Px(marker_y - marker_d * 0.5)),
                        size: fret_core::Size::new(Px(marker_d), Px(marker_d)),
                    };

                    let outline = if Some(s.id) == active {
                        Paint::Solid(accent)
                    } else {
                        Paint::Solid(border)
                    };
                    let stroke_w = if Some(s.id) == active {
                        Px(2.0)
                    } else {
                        Px(1.0)
                    };

                    p.scene().push(fret_core::SceneOp::Quad {
                        order: fret_core::DrawOrder(1),
                        rect: marker_rect,
                        background: Paint::Solid(s.color).into(),
                        border: Edges::all(stroke_w),
                        border_paint: outline.into(),
                        corner_radii: Corners::all(marker_radius),
                    });
                }
            });

            let props = CanvasProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            vec![cx.canvas(props, move |p| on_paint(p))]
        },
    )
}
