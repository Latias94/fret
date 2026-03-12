use super::*;

pub(super) fn handle_left_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    super::super::pointer_up_left_route::handle_left_pointer_up(
        canvas,
        cx,
        snapshot,
        position,
        click_count,
        modifiers,
        zoom,
    )
}
