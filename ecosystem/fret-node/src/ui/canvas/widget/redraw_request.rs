use fret_ui::UiHost;

use super::widget_tail::WidgetRedrawCx;

pub(super) fn request_redraw<H: UiHost>(cx: &mut impl WidgetRedrawCx<H>) {
    cx.request_redraw();
}

pub(super) fn request_paint_redraw<H: UiHost>(cx: &mut impl WidgetRedrawCx<H>) {
    request_redraw(cx);
}

pub(super) fn request_layout_redraw<H: UiHost>(cx: &mut impl WidgetRedrawCx<H>) {
    request_redraw(cx);
}

pub(super) fn request_paint_redraw_if<H: UiHost>(cx: &mut impl WidgetRedrawCx<H>, redraw: bool) {
    if redraw {
        request_paint_redraw(cx);
    }
}

pub(super) fn request_layout_redraw_if<H: UiHost>(cx: &mut impl WidgetRedrawCx<H>, redraw: bool) {
    if redraw {
        request_layout_redraw(cx);
    }
}
