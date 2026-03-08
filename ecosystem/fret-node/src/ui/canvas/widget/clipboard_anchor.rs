use super::*;

pub(super) fn next_paste_canvas_point<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    bounds: Rect,
    snapshot: &ViewSnapshot,
) -> CanvasPoint {
    let zoom = if snapshot.zoom.is_finite() && snapshot.zoom > 0.0 {
        snapshot.zoom
    } else {
        1.0
    };

    let anchor = canvas.interaction.last_canvas_pos.unwrap_or_else(|| {
        let cx0 = bounds.origin.x.0 + 0.5 * bounds.size.width.0;
        let cy0 = bounds.origin.y.0 + 0.5 * bounds.size.height.0;
        let center = Point::new(Px(cx0), Px(cy0));
        NodeGraphCanvasWith::<M>::screen_to_canvas(bounds, center, snapshot.pan, zoom)
    });

    let (series, at) = PasteSeries::next(canvas.interaction.paste_series, anchor, zoom);
    canvas.interaction.paste_series = Some(series);
    at
}
