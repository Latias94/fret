use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::searcher_activation::searcher_pointer_hit;
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
    ) && canvas.interaction.searcher.is_some()
    {
        let row_ix = canvas
            .interaction
            .searcher
            .as_ref()
            .map(|s| s.active_row)
            .unwrap_or(0);
        if canvas.try_activate_searcher_row(cx, row_ix) {
            return finish_searcher_event(cx);
        }
    }

    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };

    match key {
        fret_core::KeyCode::ArrowDown => {
            let n = searcher.rows.len();
            if n > 0 {
                let mut ix = (searcher.active_row + 1) % n;
                for _ in 0..n {
                    if searcher
                        .rows
                        .get(ix)
                        .is_some_and(NodeGraphCanvasWith::<M>::searcher_is_selectable_row)
                    {
                        searcher.active_row = ix;
                        break;
                    }
                    ix = (ix + 1) % n;
                }
                NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
            }
            return finish_searcher_event(cx);
        }
        fret_core::KeyCode::ArrowUp => {
            let n = searcher.rows.len();
            if n > 0 {
                let mut ix = if searcher.active_row == 0 {
                    n - 1
                } else {
                    searcher.active_row - 1
                };
                for _ in 0..n {
                    if searcher
                        .rows
                        .get(ix)
                        .is_some_and(NodeGraphCanvasWith::<M>::searcher_is_selectable_row)
                    {
                        searcher.active_row = ix;
                        break;
                    }
                    ix = if ix == 0 { n - 1 } else { ix - 1 };
                }
                NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
            }
            return finish_searcher_event(cx);
        }
        fret_core::KeyCode::Backspace => {
            if !searcher.query.is_empty() {
                searcher.query.pop();
                NodeGraphCanvasWith::<M>::rebuild_searcher_rows(searcher);
                return finish_searcher_event(cx);
            }
        }
        _ => {}
    }

    if !modifiers.ctrl
        && !modifiers.meta
        && let Some(ch) = fret_core::keycode_to_ascii_lowercase(key)
    {
        searcher.query.push(ch);
        NodeGraphCanvasWith::<M>::rebuild_searcher_rows(searcher);
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
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };

    let new_hover = super::hit_searcher_row(&canvas.style, searcher, position, zoom);
    if searcher.hovered_row != new_hover {
        searcher.hovered_row = new_hover;
        if let Some(ix) = new_hover
            && searcher
                .rows
                .get(ix)
                .is_some_and(NodeGraphCanvasWith::<M>::searcher_is_selectable_row)
        {
            searcher.active_row = ix;
            NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
        }
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
    if modifiers.ctrl || modifiers.meta {
        return false;
    }

    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };

    let n = searcher.rows.len();
    if n == 0 {
        return true;
    }

    let visible = super::SEARCHER_MAX_VISIBLE_ROWS.min(n);
    let max_scroll = n.saturating_sub(visible);
    if delta.y.0 > 0.0 {
        searcher.scroll = searcher.scroll.saturating_sub(1);
    } else if delta.y.0 < 0.0 {
        searcher.scroll = (searcher.scroll + 1).min(max_scroll);
    }

    NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    invalidate_searcher_paint(cx);
    true
}
