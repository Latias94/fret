use super::*;

pub(super) fn handle_pointer_up_guards<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    right_click::handle_pending_right_click_pointer_up(canvas, cx, snapshot, position, button, zoom)
        || (button == MouseButton::Left
            && searcher::handle_searcher_pointer_up(canvas, cx, position, button, zoom))
}

pub(super) fn dispatch_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    click_count: u8,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    pointer_up::handle_pointer_up(
        canvas,
        cx,
        snapshot,
        position,
        button,
        click_count,
        modifiers,
        zoom,
    )
}
