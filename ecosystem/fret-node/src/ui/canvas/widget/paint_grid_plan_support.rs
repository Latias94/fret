use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

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

pub(super) fn populate_grid_tiles<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    viewport_rect: Rect,
    grid_rect: Rect,
    tile_size_canvas: f32,
) {
    let grid_tiles = TileGrid2D::new(tile_size_canvas);
    grid_tiles.tiles_in_rect(grid_rect, &mut canvas.grid_tiles_scratch);
    grid_tiles.sort_tiles_center_first(viewport_rect, &mut canvas.grid_tiles_scratch);
}

pub(super) fn validate_pattern_size(
    pattern: NodeGraphBackgroundPattern,
    dot_size: f32,
    cross_size: f32,
) -> bool {
    if matches!(pattern, NodeGraphBackgroundPattern::Dots)
        && !(dot_size.is_finite() && dot_size > 0.0)
    {
        return false;
    }
    if matches!(pattern, NodeGraphBackgroundPattern::Cross)
        && !(cross_size.is_finite() && cross_size > 0.0)
    {
        return false;
    }
    true
}
