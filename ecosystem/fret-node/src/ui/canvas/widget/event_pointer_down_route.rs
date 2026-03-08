use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerDownTailRoute {
    RightClick,
    LeftClick,
    Ignore,
}

pub(super) fn route_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) {
    if searcher::handle_searcher_pointer_down(canvas, cx, position, button, zoom) {
        return;
    }

    if pointer_down_gesture_start::handle_close_button_pointer_down(
        canvas, cx, snapshot, position, button, zoom,
    ) {
        return;
    }

    if handle_left_button_double_click_routes(
        canvas,
        cx,
        snapshot,
        position,
        button,
        modifiers,
        click_count,
        zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_context_menu_pointer_down(
        canvas, cx, position, button, zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_pending_right_click_start(
        canvas, cx, snapshot, position, button,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_sticky_wire_pointer_down(
        canvas, cx, snapshot, position, button, zoom,
    ) {
        return;
    }

    if pointer_down_gesture_start::handle_pan_start_pointer_down(
        canvas, cx, snapshot, position, button, modifiers,
    ) {
        return;
    }

    match tail_pointer_down_route(button) {
        PointerDownTailRoute::RightClick => {
            let _ =
                right_click::handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom);
        }
        PointerDownTailRoute::LeftClick => {
            let _ = left_click::handle_left_click_pointer_down(
                canvas, cx, snapshot, position, modifiers, zoom,
            );
        }
        PointerDownTailRoute::Ignore => {}
    }
}

fn handle_left_button_double_click_routes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }

    if pointer_down_double_click::handle_background_zoom_double_click(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        click_count,
        zoom,
    ) {
        return true;
    }

    if pointer_down_double_click::handle_edge_insert_picker_double_click(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        click_count,
        zoom,
    ) {
        return true;
    }

    pointer_down_double_click::handle_edge_reroute_double_click(
        canvas,
        cx,
        snapshot,
        position,
        click_count,
        zoom,
    )
}

fn tail_pointer_down_route(button: MouseButton) -> PointerDownTailRoute {
    match button {
        MouseButton::Right => PointerDownTailRoute::RightClick,
        MouseButton::Left => PointerDownTailRoute::LeftClick,
        _ => PointerDownTailRoute::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_pointer_down_route_maps_buttons_to_expected_lane() {
        assert_eq!(
            tail_pointer_down_route(MouseButton::Left),
            PointerDownTailRoute::LeftClick
        );
        assert_eq!(
            tail_pointer_down_route(MouseButton::Right),
            PointerDownTailRoute::RightClick
        );
        assert_eq!(
            tail_pointer_down_route(MouseButton::Middle),
            PointerDownTailRoute::Ignore
        );
    }
}
