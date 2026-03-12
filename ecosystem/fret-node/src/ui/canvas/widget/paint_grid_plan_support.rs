#[path = "paint_grid_plan_support/hint.rs"]
mod hint;
#[path = "paint_grid_plan_support/metrics.rs"]
mod metrics;
#[path = "paint_grid_plan_support/tiles.rs"]
mod tiles;
#[path = "paint_grid_plan_support/validate.rs"]
mod validate;

use super::*;

pub(super) fn resolve_canvas_chrome_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) -> crate::ui::CanvasChromeHint {
    hint::resolve_canvas_chrome_hint(canvas, cx)
}

pub(super) fn resolve_grid_line_width_px<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    canvas_hint: &crate::ui::CanvasChromeHint,
) -> f32 {
    metrics::resolve_grid_line_width_px(canvas, canvas_hint)
}

pub(super) fn resolve_grid_thickness(zoom: f32, line_width_px: f32) -> Px {
    metrics::resolve_grid_thickness(zoom, line_width_px)
}

pub(super) fn resolve_grid_tile_size_canvas(zoom: f32) -> f32 {
    metrics::resolve_grid_tile_size_canvas(zoom)
}

pub(super) fn populate_grid_tiles<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    viewport_rect: Rect,
    grid_rect: Rect,
    tile_size_canvas: f32,
) {
    tiles::populate_grid_tiles(canvas, viewport_rect, grid_rect, tile_size_canvas)
}

pub(super) fn validate_pattern_size(
    pattern: crate::ui::style::NodeGraphBackgroundPattern,
    dot_size: f32,
    cross_size: f32,
) -> bool {
    validate::validate_pattern_size(pattern, dot_size, cross_size)
}
