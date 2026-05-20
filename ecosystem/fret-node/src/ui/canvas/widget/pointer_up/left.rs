use super::*;

pub(super) fn handle_left_pointer_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: super::super::pointer_up_cx::PointerUpCx<H, M>,
{
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
