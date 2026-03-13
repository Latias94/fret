use super::*;

pub(super) fn resolve_canvas_chrome_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) -> crate::ui::CanvasChromeHint {
    if let Some(skin) = canvas.skin.as_ref() {
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                skin.canvas_chrome_hint(graph, &canvas.style)
            })
            .ok()
            .unwrap_or_default()
    } else {
        crate::ui::CanvasChromeHint::default()
    }
}
