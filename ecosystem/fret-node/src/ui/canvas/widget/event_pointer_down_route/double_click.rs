use super::super::*;

pub(in super::super) fn handle_left_button_double_click_routes<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
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
