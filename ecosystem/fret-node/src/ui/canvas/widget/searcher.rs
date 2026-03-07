use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::searcher_activation::searcher_pointer_hit;
use super::searcher_input::SearcherStepDirection;
use super::searcher_ui::{finish_searcher_event, invalidate_searcher_paint};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_searcher_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.searcher.is_some() {
        canvas.dismiss_searcher_overlay(cx);
        return finish_searcher_event(cx);
    }
    false
}

pub(super) fn handle_searcher_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
    modifiers: Modifiers,
) -> bool {
    if matches!(
        key,
        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter
    ) && canvas.try_activate_active_searcher_row(cx)
    {
        return finish_searcher_event(cx);
    }

    if canvas.interaction.searcher.is_none() {
        return false;
    }

    match key {
        fret_core::KeyCode::ArrowDown => {
            let _ = canvas.step_searcher_active_row(SearcherStepDirection::Forward);
            return finish_searcher_event(cx);
        }
        fret_core::KeyCode::ArrowUp => {
            let _ = canvas.step_searcher_active_row(SearcherStepDirection::Backward);
            return finish_searcher_event(cx);
        }
        _ => {}
    }

    if canvas.update_searcher_query_from_key(key, modifiers) {
        return finish_searcher_event(cx);
    }

    false
}

pub(super) fn handle_searcher_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    let hit = searcher_pointer_hit(canvas, position, zoom);

    match button {
        MouseButton::Left => {
            if let Some(row_ix) = hit.row_ix {
                let _ = canvas.arm_searcher_row_drag(cx, row_ix, position);
            } else if !hit.inside {
                canvas.dismiss_searcher_overlay(cx);
            }
            finish_searcher_event(cx)
        }
        MouseButton::Right => {
            canvas.dismiss_searcher_overlay(cx);
            finish_searcher_event(cx)
        }
        _ => {
            canvas.dismiss_searcher_overlay(cx);
            finish_searcher_event(cx)
        }
    }
}

pub(super) fn handle_searcher_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }
    if canvas.interaction.searcher.is_none() {
        canvas.interaction.pending_insert_node_drag = None;
        return false;
    }

    let hit = searcher_pointer_hit(canvas, position, zoom);

    if canvas.interaction.pending_insert_node_drag.take().is_some() {
        cx.release_pointer_capture();
        canvas.activate_searcher_hit_or_dismiss(cx, hit);
        return finish_searcher_event(cx);
    }

    false
}

pub(super) fn handle_searcher_pointer_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.update_searcher_hover_from_position(position, zoom) {
        invalidate_searcher_paint(cx);
    }
    true
}

pub(super) fn handle_searcher_wheel<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    delta: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.scroll_searcher_from_wheel(delta, modifiers) {
        invalidate_searcher_paint(cx);
        return true;
    }

    !modifiers.ctrl && !modifiers.meta
}
