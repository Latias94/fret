use super::*;

pub(super) fn route_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::super::event_pointer_down_route_cx::PointerDownRouteCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) {
    super::super::event_pointer_down_route::route_pointer_down(
        canvas,
        cx,
        snapshot,
        position,
        button,
        modifiers,
        click_count,
        zoom,
    );
}
