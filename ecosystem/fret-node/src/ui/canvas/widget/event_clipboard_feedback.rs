use fret_ui::UiHost;

use super::event_clipboard_feedback_cx::ClipboardFeedbackCx;
use super::low_level_adapter::{CanvasPaintInvalidationCx, invalidate_canvas_paint};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::rules::DiagnosticSeverity;

pub(super) fn request_paste_feedback<H: UiHost>(cx: &mut impl CanvasPaintInvalidationCx<H>) {
    invalidate_canvas_paint(cx);
}

pub(super) fn show_clipboard_unavailable_toast<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: ClipboardFeedbackCx<H>,
{
    let window = cx.window();
    canvas.show_toast(
        cx.host(),
        window,
        DiagnosticSeverity::Info,
        "clipboard text unavailable",
    );
    request_paste_feedback(cx);
}
