use super::*;
use crate::ui::style::NodeGraphBackgroundPattern;

pub(super) struct GridPaintPlan {
    pub(super) zoom: f32,
    pub(super) pattern: NodeGraphBackgroundPattern,
    pub(super) spacing: f32,
    pub(super) major_every: i64,
    pub(super) thickness: Px,
    pub(super) line_width_px: f32,
    pub(super) dot_size: f32,
    pub(super) cross_size: f32,
    pub(super) tile_size_canvas: f32,
    pub(super) major_color: Color,
    pub(super) minor_color: Color,
}

pub(super) fn prepare_grid_paint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    viewport_rect: Rect,
    render_cull_rect: Option<Rect>,
    zoom: f32,
) -> Option<GridPaintPlan> {
    let canvas_hint = super::paint_grid_plan_support::resolve_canvas_chrome_hint(canvas, cx);
    let pattern = canvas.style.paint.grid_pattern;
    let spacing = canvas.style.paint.grid_spacing;
    if !(spacing.is_finite() && spacing > 1.0e-3) {
        return None;
    }

    let line_width_px =
        super::paint_grid_plan_support::resolve_grid_line_width_px(canvas, &canvas_hint);
    let thickness = super::paint_grid_plan_support::resolve_grid_thickness(zoom, line_width_px);
    let tile_size_canvas = super::paint_grid_plan_support::resolve_grid_tile_size_canvas(zoom);
    let grid_rect = render_cull_rect.unwrap_or(viewport_rect);
    super::paint_grid_plan_support::populate_grid_tiles(
        canvas,
        viewport_rect,
        grid_rect,
        tile_size_canvas,
    );

    let major_color = canvas_hint
        .grid_major
        .unwrap_or(canvas.style.paint.grid_major_color);
    let minor_color = canvas_hint
        .grid_minor
        .unwrap_or(canvas.style.paint.grid_minor_color);
    let dot_size = canvas.style.paint.grid_dot_size;
    let cross_size = canvas.style.paint.grid_cross_size;
    if !super::paint_grid_plan_support::validate_pattern_size(pattern, dot_size, cross_size) {
        return None;
    }

    Some(GridPaintPlan {
        zoom,
        pattern,
        spacing,
        major_every: canvas.style.paint.grid_major_every.max(1) as i64,
        thickness,
        line_width_px,
        dot_size,
        cross_size,
        tile_size_canvas,
        major_color,
        minor_color,
    })
}
