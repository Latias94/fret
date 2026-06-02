use std::sync::{Arc, Mutex};

use fret_core::{Color, MouseButton, PointerId, Px};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, CanvasProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

mod paint;

use paint::{GradientPreviewPaintInput, gradient_preview_paint};

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

            let on_paint = gradient_preview_paint(GradientPreviewPaintInput {
                angle_deg,
                stops: stops.clone(),
                active_stop,
                preview_state: state_for_paint.clone(),
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
