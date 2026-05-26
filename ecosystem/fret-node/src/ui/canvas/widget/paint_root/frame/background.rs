use super::*;

pub(super) fn paint_canvas_background<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    viewport_rect: Rect,
) {
    let canvas_hint =
        crate::ui::canvas::widget::paint_grid_plan_support::resolve_canvas_chrome_hint(canvas, cx);
    let background = canvas_hint
        .background
        .unwrap_or(canvas.style.paint.background);
    super::super::frame_background_adapter::paint_root_frame_background(
        cx,
        viewport_rect,
        background,
    );
}
