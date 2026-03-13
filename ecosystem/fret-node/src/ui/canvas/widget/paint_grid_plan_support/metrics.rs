use super::*;

pub(super) fn resolve_grid_line_width_px<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    canvas_hint: &crate::ui::CanvasChromeHint,
) -> f32 {
    if let Some(value) = canvas_hint.grid_line_width_px
        && value.is_finite()
        && value > 0.0
    {
        value
    } else if canvas.style.paint.grid_line_width.is_finite()
        && canvas.style.paint.grid_line_width > 0.0
    {
        canvas.style.paint.grid_line_width
    } else {
        1.0
    }
}

pub(super) fn resolve_grid_thickness(zoom: f32, line_width_px: f32) -> Px {
    let z = zoom.max(1.0e-6);
    let thickness_px = line_width_px.max(0.25);
    Px(thickness_px / z)
}

pub(super) fn resolve_grid_tile_size_canvas(zoom: f32) -> f32 {
    (NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::GRID_TILE_SIZE_SCREEN_PX
        / zoom.max(1.0e-6))
    .max(1.0)
}
