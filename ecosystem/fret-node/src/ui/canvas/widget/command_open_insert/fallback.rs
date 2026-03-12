use super::super::*;

pub(super) fn insert_picker_fallback_canvas_point<M: NodeGraphCanvasMiddleware>(
    snapshot: &ViewSnapshot,
    bounds: Option<Rect>,
) -> Option<CanvasPoint> {
    let bounds = bounds?;
    let center = Point::new(
        Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
        Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
    );
    Some(NodeGraphCanvasWith::<M>::screen_to_canvas(
        bounds,
        center,
        snapshot.pan,
        snapshot.zoom,
    ))
}

#[cfg(test)]
mod tests;
