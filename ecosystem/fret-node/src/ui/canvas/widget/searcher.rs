use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::NodeGraphCanvas;

pub(super) fn handle_searcher_escape<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.searcher.take().is_some() {
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    false
}

pub(super) fn handle_searcher_key_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
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
                        .is_some_and(NodeGraphCanvas::searcher_is_selectable_row)
                    {
                        searcher.active_row = ix;
                        break;
                    }
                    ix = (ix + 1) % n;
                }
                NodeGraphCanvas::ensure_searcher_active_visible(searcher);
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
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
                        .is_some_and(NodeGraphCanvas::searcher_is_selectable_row)
                    {
                        searcher.active_row = ix;
                        break;
                    }
                    ix = if ix == 0 { n - 1 } else { ix - 1 };
                }
                NodeGraphCanvas::ensure_searcher_active_visible(searcher);
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Backspace => {
            if !searcher.query.is_empty() {
                searcher.query.pop();
                NodeGraphCanvas::rebuild_searcher_rows(searcher);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                return true;
            }
        }
        _ => {}
    }

    if !modifiers.ctrl
        && !modifiers.meta
        && let Some(ch) = fret_core::keycode_to_ascii_lowercase(key)
    {
        searcher.query.push(ch);
        NodeGraphCanvas::rebuild_searcher_rows(searcher);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}

pub(super) fn handle_searcher_pointer_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    let (inside, hit_row) = if let Some(searcher) = canvas.interaction.searcher.as_ref() {
        let visible = super::searcher_visible_rows(searcher);
        let rect = super::searcher_rect_at(&canvas.style, searcher.origin, visible, zoom);
        let inside = rect.contains(position);
        let hit_row = super::hit_searcher_row(&canvas.style, searcher, position, zoom);
        (inside, hit_row)
    } else {
        (false, None)
    };

    match button {
        MouseButton::Left => {
            if let Some(row_ix) = hit_row {
                let _ = canvas.try_activate_searcher_row(cx, row_ix);
            } else if !inside {
                canvas.interaction.searcher = None;
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        MouseButton::Right => {
            canvas.interaction.searcher = None;
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        _ => {
            canvas.interaction.searcher = None;
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
    }
}

pub(super) fn handle_searcher_pointer_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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
                .is_some_and(NodeGraphCanvas::searcher_is_selectable_row)
        {
            searcher.active_row = ix;
            NodeGraphCanvas::ensure_searcher_active_visible(searcher);
        }
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
    true
}

pub(super) fn handle_searcher_wheel<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
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

    NodeGraphCanvas::ensure_searcher_active_visible(searcher);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
