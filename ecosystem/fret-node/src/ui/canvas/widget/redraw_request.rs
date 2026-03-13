use fret_ui::UiHost;
use fret_ui::retained_bridge::{LayoutCx, PaintCx};

use super::wire_drag::WireCommitCx;

pub(super) fn request_paint_redraw<H: UiHost>(cx: &mut PaintCx<'_, H>) {
    cx.request_redraw();
}

pub(super) fn request_layout_redraw<H: UiHost>(cx: &mut LayoutCx<'_, H>) {
    cx.request_redraw();
}

pub(super) fn request_commit_redraw<H: UiHost>(cx: &mut impl WireCommitCx<H>) {
    cx.request_redraw();
}

pub(super) fn request_paint_redraw_if<H: UiHost>(cx: &mut PaintCx<'_, H>, redraw: bool) {
    if redraw {
        request_paint_redraw(cx);
    }
}

pub(super) fn request_layout_redraw_if<H: UiHost>(cx: &mut LayoutCx<'_, H>, redraw: bool) {
    if redraw {
        request_layout_redraw(cx);
    }
}
