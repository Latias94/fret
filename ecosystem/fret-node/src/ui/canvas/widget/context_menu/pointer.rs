use crate::ui::canvas::widget::*;
pub(super) fn handle_context_menu_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    let Some(menu) = canvas.interaction.context_menu.take() else {
        return false;
    };
    match button {
        MouseButton::Left => {
            if let Some(ix) = hit_context_menu_item(&canvas.style, &menu, position, zoom) {
                let item = menu.items.get(ix).cloned();
                let target = menu.target.clone();
                let invoked_at = menu.invoked_at;
                let candidates = menu.candidates.clone();
                if let Some(item) = item
                    && item.enabled
                {
                    canvas.activate_context_menu_item(cx, &target, invoked_at, item, &candidates);
                }
            } else {
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        MouseButton::Right => false,
        _ => {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
    }
}
pub(super) fn handle_context_menu_pointer_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(menu) = canvas.interaction.context_menu.as_mut() else {
        return false;
    };
    let new_hover = hit_context_menu_item(&canvas.style, menu, position, zoom);
    if menu.hovered_item != new_hover {
        menu.hovered_item = new_hover;
        if let Some(ix) = new_hover {
            if menu.items.get(ix).is_some_and(|it| it.enabled) {
                menu.active_item = ix.min(menu.items.len().saturating_sub(1));
                menu.typeahead.clear();
            }
        }
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
    true
}
