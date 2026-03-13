use super::*;

pub(super) fn paint_canvas_background<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    viewport_rect: Rect,
) {
    let canvas_hint =
        crate::ui::canvas::widget::paint_grid_plan_support::resolve_canvas_chrome_hint(canvas, cx);
    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: viewport_rect,
        background: fret_core::Paint::Solid(
            canvas_hint
                .background
                .unwrap_or(canvas.style.paint.background),
        )
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}
