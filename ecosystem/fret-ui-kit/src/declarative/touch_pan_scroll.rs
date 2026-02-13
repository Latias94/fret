use std::sync::Arc;

use fret_core::{Point, Px};
use fret_runtime::Model;
use fret_ui::action::{OnPointerDown, OnPointerMove, OnPointerUp};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui::{action::OnPointerCancel, element::ScrollAxis};

use super::controllable_state::use_controllable_model;

#[derive(Debug, Clone, Copy)]
pub struct TouchPanToScrollOptions {
    pub drag_threshold: Px,
}

impl Default for TouchPanToScrollOptions {
    fn default() -> Self {
        Self {
            drag_threshold: Px(6.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TouchPanToScrollState {
    pointer_id: Option<fret_core::PointerId>,
    start: Option<Point>,
    last: Option<Point>,
    panning: bool,
}

fn axis_scrollable(handle: &ScrollHandle, axis: ScrollAxis) -> bool {
    let max = handle.max_offset();
    match axis {
        ScrollAxis::X => max.x.0 > 0.01,
        ScrollAxis::Y => max.y.0 > 0.01,
        ScrollAxis::Both => max.x.0 > 0.01 || max.y.0 > 0.01,
    }
}

fn apply_pan_delta(handle: &ScrollHandle, axis: ScrollAxis, delta: Point) {
    let prev = handle.offset();
    let next = match axis {
        ScrollAxis::X => Point::new(Px(prev.x.0 - delta.x.0), prev.y),
        ScrollAxis::Y => Point::new(prev.x, Px(prev.y.0 - delta.y.0)),
        ScrollAxis::Both => Point::new(Px(prev.x.0 - delta.x.0), Px(prev.y.0 - delta.y.0)),
    };
    handle.set_offset(next);
}

/// Installs a minimal touch "pan to scroll" policy onto the current `PointerRegion` element.
///
/// Notes:
/// - This is intentionally *touch-only* and does not attempt to implement a full gesture arena.
/// - Press/tap activation should already be gated by `PointerEvent::Up.is_click` for touch
///   pressables, so a pan that exceeds click slop will not activate descendants.
#[track_caller]
pub fn install_touch_pan_to_scroll<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    axis: ScrollAxis,
    scroll_handle: ScrollHandle,
    options: TouchPanToScrollOptions,
) {
    let state: Model<TouchPanToScrollState> = use_controllable_model(
        cx,
        None::<Model<TouchPanToScrollState>>,
        TouchPanToScrollState::default,
    )
    .model();

    let state_c = state.clone();
    let handle_c = scroll_handle.clone();
    let on_down: OnPointerDown = Arc::new(move |host, _action_cx, down| {
        if down.pointer_type != fret_core::PointerType::Touch {
            return false;
        }
        if !axis_scrollable(&handle_c, axis) {
            return false;
        }

        let _ = host.models_mut().update(&state_c, |st| {
            st.pointer_id = Some(down.pointer_id);
            st.start = Some(down.position);
            st.last = Some(down.position);
            st.panning = false;
        });
        false
    });

    let state_c = state.clone();
    let handle_c = scroll_handle.clone();
    let on_move: OnPointerMove = Arc::new(move |host, action_cx, mv| {
        if mv.pointer_type != fret_core::PointerType::Touch {
            return false;
        }
        if !axis_scrollable(&handle_c, axis) {
            return false;
        }

        let mut out = false;
        let _ = host.models_mut().update(&state_c, |st| {
            if st.pointer_id != Some(mv.pointer_id) {
                return;
            }
            let Some(prev) = st.last else {
                st.last = Some(mv.position);
                return;
            };
            let Some(start) = st.start else {
                st.start = Some(mv.position);
                st.last = Some(mv.position);
                return;
            };

            let total_dx = mv.position.x.0 - start.x.0;
            let total_dy = mv.position.y.0 - start.y.0;
            if !st.panning {
                let dist = (total_dx * total_dx + total_dy * total_dy).sqrt();
                if dist > options.drag_threshold.0.max(0.0) {
                    st.panning = true;
                }
            }

            let dx = mv.position.x.0 - prev.x.0;
            let dy = mv.position.y.0 - prev.y.0;
            st.last = Some(mv.position);

            if st.panning {
                apply_pan_delta(&handle_c, axis, Point::new(Px(dx), Px(dy)));
                out = true;
            }
        });

        if out {
            host.invalidate(Invalidation::HitTestOnly);
            host.request_redraw(action_cx.window);
        }
        out
    });

    let state_c = state.clone();
    let on_up: OnPointerUp = Arc::new(move |host, _action_cx, up| {
        if up.pointer_type != fret_core::PointerType::Touch {
            return false;
        }
        let _ = host.models_mut().update(&state_c, |st| {
            if st.pointer_id == Some(up.pointer_id) {
                *st = TouchPanToScrollState::default();
            }
        });
        false
    });

    let state_c = state.clone();
    let on_cancel: OnPointerCancel = Arc::new(move |host, _action_cx, cancel| {
        if cancel.pointer_type != fret_core::PointerType::Touch {
            return false;
        }
        let _ = host.models_mut().update(&state_c, |st| {
            if st.pointer_id == Some(cancel.pointer_id) {
                *st = TouchPanToScrollState::default();
            }
        });
        false
    });

    cx.pointer_region_add_on_pointer_down(on_down);
    cx.pointer_region_add_on_pointer_move(on_move);
    cx.pointer_region_add_on_pointer_up(on_up);
    cx.pointer_region_on_pointer_cancel(on_cancel);
}
