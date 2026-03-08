use super::*;

pub(super) fn handle_background_zoom_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    super::pointer_down_double_click_background::handle_background_zoom_double_click(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        click_count,
        zoom,
    )
}

pub(super) fn handle_edge_insert_picker_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    super::pointer_down_double_click_edge::handle_edge_insert_picker_double_click(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        click_count,
        zoom,
    )
}

pub(super) fn handle_edge_reroute_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    zoom: f32,
) -> bool {
    super::pointer_down_double_click_edge::handle_edge_reroute_double_click(
        canvas,
        cx,
        snapshot,
        position,
        click_count,
        zoom,
    )
}
