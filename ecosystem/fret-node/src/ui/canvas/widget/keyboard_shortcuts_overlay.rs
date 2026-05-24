use super::*;

pub(in crate::ui::canvas::widget) trait KeyboardOverlayCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    searcher::SearcherCx<H, M> + context_menu::ContextMenuCx<H, M> + cancel_cx::CancelGestureCx<H>
{
}

impl<H: UiHost, M, T> KeyboardOverlayCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: searcher::SearcherCx<H, M>
        + context_menu::ContextMenuCx<H, M>
        + cancel_cx::CancelGestureCx<H>,
{
}

pub(super) fn handle_escape_key<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl KeyboardOverlayCx<H, M>,
    key: fret_core::KeyCode,
) -> bool {
    if key != fret_core::KeyCode::Escape {
        return false;
    }

    if searcher::handle_searcher_escape(canvas, cx)
        || context_menu::handle_context_menu_escape(canvas, cx)
    {
        return true;
    }

    cancel::handle_escape_cancel(canvas, cx);
    true
}

pub(super) fn handle_overlay_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl KeyboardOverlayCx<H, M>,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    searcher::handle_searcher_key_down(canvas, cx, key, modifiers)
        || context_menu::handle_context_menu_key_down(canvas, cx, key)
}
