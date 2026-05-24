mod finish;
mod insert_picker;
mod reroute;
mod target;

use super::*;

pub(super) fn handle_edge_insert_picker_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl pointer_down_double_click_cx::PointerDownDoubleClickCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    insert_picker::handle_edge_insert_picker_double_click(
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
    cx: &mut impl pointer_down_double_click_cx::PointerDownDoubleClickCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    zoom: f32,
) -> bool {
    reroute::handle_edge_reroute_double_click(canvas, cx, snapshot, position, click_count, zoom)
}
