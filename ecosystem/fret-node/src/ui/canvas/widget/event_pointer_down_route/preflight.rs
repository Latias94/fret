use super::*;

pub(in super::super) trait PointerDownPreflightCx<H: UiHost>:
    searcher_activation::SearcherPointerDownCx<H>
    + pointer_down_close_button_cx::PointerDownCloseButtonCx<H>
    + pointer_down_double_click_cx::PointerDownDoubleClickCx<H>
{
}

impl<H, T> PointerDownPreflightCx<H> for T
where
    H: UiHost,
    T: searcher_activation::SearcherPointerDownCx<H>
        + pointer_down_close_button_cx::PointerDownCloseButtonCx<H>
        + pointer_down_double_click_cx::PointerDownDoubleClickCx<H>,
{
}

pub(super) fn handle_pointer_down_preflight<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerDownPreflightCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) -> bool {
    searcher::handle_searcher_pointer_down(canvas, cx, position, button, zoom)
        || pointer_down_gesture_start::handle_close_button_pointer_down(
            canvas, cx, snapshot, position, button, zoom,
        )
        || double_click::handle_left_button_double_click_routes(
            canvas,
            cx,
            snapshot,
            position,
            button,
            modifiers,
            click_count,
            zoom,
        )
}
