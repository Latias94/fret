use super::super::{
    MouseButton, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, Point, UiHost, ViewSnapshot,
    right_click, searcher,
};

pub(in super::super) trait PointerUpGuardCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    right_click::RightClickCx<H, M> + searcher::SearcherCx<H, M>
{
}

impl<H, M, T> PointerUpGuardCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: right_click::RightClickCx<H, M> + searcher::SearcherCx<H, M>,
{
}

pub(super) fn handle_pointer_up_guards<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerUpGuardCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    right_click::handle_pending_right_click_pointer_up(canvas, cx, snapshot, position, button, zoom)
        || (button == MouseButton::Left
            && searcher::handle_searcher_pointer_up(canvas, cx, position, button, zoom))
}
