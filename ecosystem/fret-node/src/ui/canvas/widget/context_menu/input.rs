use super::super::*;
pub(super) fn handle_context_menu_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.context_menu.take().is_some() {
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    false
}
pub(super) fn handle_context_menu_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    let Some(mut menu) = canvas.interaction.context_menu.take() else {
        return false;
    };
    match key {
        fret_core::KeyCode::ArrowDown => {
            let n = menu.items.len();
            if n > 0 {
                let mut ix = (menu.active_item + 1) % n;
                for _ in 0..n {
                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                        break;
                    }
                    ix = (ix + 1) % n;
                }
                menu.active_item = ix;
            }
            menu.typeahead.clear();
            canvas.interaction.context_menu = Some(menu);
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::ArrowUp => {
            let n = menu.items.len();
            if n > 0 {
                let mut ix = if menu.active_item == 0 {
                    n - 1
                } else {
                    menu.active_item - 1
                };
                for _ in 0..n {
                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                        break;
                    }
                    ix = if ix == 0 { n - 1 } else { ix - 1 };
                }
                menu.active_item = ix;
            }
            menu.typeahead.clear();
            canvas.interaction.context_menu = Some(menu);
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
            let ix = menu.active_item.min(menu.items.len().saturating_sub(1));
            let item = menu.items.get(ix).cloned();
            let target = menu.target.clone();
            let invoked_at = menu.invoked_at;
            let candidates = menu.candidates.clone();
            if let Some(item) = item
                && item.enabled
            {
                canvas.activate_context_menu_item(cx, &target, invoked_at, item, &candidates);
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Backspace => {
            if !menu.typeahead.is_empty() {
                menu.typeahead.pop();
                canvas.interaction.context_menu = Some(menu);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                return true;
            }
        }
        _ => {}
    }
    if let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) {
        let try_find = |needle: &str| -> Option<usize> {
            if needle.is_empty() {
                return None;
            }
            menu.items.iter().position(|it| {
                it.enabled && it.label.as_ref().to_ascii_lowercase().starts_with(needle)
            })
        };
        menu.typeahead.push(ch);
        let mut needle = menu.typeahead.to_ascii_lowercase();
        let mut hit = try_find(&needle);
        if hit.is_none() {
            needle.clear();
            needle.push(ch);
            hit = try_find(&needle);
            if hit.is_some() {
                menu.typeahead.clear();
                menu.typeahead.push(ch);
            }
        }
        if let Some(ix) = hit {
            menu.active_item = ix.min(menu.items.len().saturating_sub(1));
        }
        canvas.interaction.context_menu = Some(menu);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    canvas.interaction.context_menu = Some(menu);
    false
}
