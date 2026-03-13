use super::*;

pub(super) fn paint_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    marquee: &MarqueeDrag,
    zoom: f32,
) {
    let rect = rect_from_points(marquee.start_pos, marquee.pos);
    let border_w = Px(canvas.style.paint.marquee_border_width / zoom);

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(49),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.marquee_fill).into(),
        border: Edges::all(border_w),
        border_paint: fret_core::Paint::Solid(canvas.style.paint.marquee_border).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}

pub(super) fn paint_snap_guides<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    guides: &SnapGuides,
    zoom: f32,
    viewport_origin_x: f32,
    viewport_origin_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let w = Px((canvas.style.paint.snapline_width / zoom).max(0.5 / zoom));
    let half = 0.5 * w.0;

    if let Some(x) = guides.x {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(48),
            rect: Rect::new(
                Point::new(Px(x - half), Px(viewport_origin_y)),
                Size::new(w, Px(viewport_h)),
            ),
            background: fret_core::Paint::Solid(canvas.style.paint.snapline_color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    if let Some(y) = guides.y {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(48),
            rect: Rect::new(
                Point::new(Px(viewport_origin_x), Px(y - half)),
                Size::new(Px(viewport_w), w),
            ),
            background: fret_core::Paint::Solid(canvas.style.paint.snapline_color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}
