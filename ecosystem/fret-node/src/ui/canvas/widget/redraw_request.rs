use fret_ui::UiHost;

use super::low_level_adapter::CanvasRedrawCx;

pub(super) fn request_redraw<H: UiHost>(cx: &mut impl CanvasRedrawCx<H>) {
    cx.request_redraw();
}

pub(super) fn request_paint_redraw<H: UiHost>(cx: &mut impl CanvasRedrawCx<H>) {
    request_redraw(cx);
}

pub(super) fn request_layout_redraw<H: UiHost>(cx: &mut impl CanvasRedrawCx<H>) {
    request_redraw(cx);
}

pub(super) fn request_paint_redraw_if<H: UiHost>(cx: &mut impl CanvasRedrawCx<H>, redraw: bool) {
    if redraw {
        request_paint_redraw(cx);
    }
}

pub(super) fn request_layout_redraw_if<H: UiHost>(cx: &mut impl CanvasRedrawCx<H>, redraw: bool) {
    if redraw {
        request_layout_redraw(cx);
    }
}
