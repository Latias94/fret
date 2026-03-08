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
    let canvas_hint = resolve_canvas_chrome_hint(canvas, cx);
    let pattern = canvas.style.paint.grid_pattern;
    let spacing = canvas.style.paint.grid_spacing;
    if !(spacing.is_finite() && spacing > 1.0e-3) {
        return None;
    }

    let line_width_px = resolve_grid_line_width_px(canvas, &canvas_hint);
    let thickness = resolve_grid_thickness(zoom, line_width_px);
    let tile_size_canvas = resolve_grid_tile_size_canvas(zoom);
    let grid_rect = render_cull_rect.unwrap_or(viewport_rect);
    populate_grid_tiles(canvas, viewport_rect, grid_rect, tile_size_canvas);

    let major_color = canvas_hint
        .grid_major
        .unwrap_or(canvas.style.paint.grid_major_color);
    let minor_color = canvas_hint
        .grid_minor
        .unwrap_or(canvas.style.paint.grid_minor_color);
    let dot_size = canvas.style.paint.grid_dot_size;
    let cross_size = canvas.style.paint.grid_cross_size;
    if !validate_pattern_size(pattern, dot_size, cross_size) {
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

fn resolve_canvas_chrome_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

fn resolve_grid_line_width_px<M: NodeGraphCanvasMiddleware>(
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

fn resolve_grid_thickness(zoom: f32, line_width_px: f32) -> Px {
    let z = zoom.max(1.0e-6);
    let thickness_px = line_width_px.max(0.25);
    Px(thickness_px / z)
}

fn resolve_grid_tile_size_canvas(zoom: f32) -> f32 {
    (NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::GRID_TILE_SIZE_SCREEN_PX
        / zoom.max(1.0e-6))
    .max(1.0)
}

fn populate_grid_tiles<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    viewport_rect: Rect,
    grid_rect: Rect,
    tile_size_canvas: f32,
) {
    let grid_tiles = TileGrid2D::new(tile_size_canvas);
    grid_tiles.tiles_in_rect(grid_rect, &mut canvas.grid_tiles_scratch);
    grid_tiles.sort_tiles_center_first(viewport_rect, &mut canvas.grid_tiles_scratch);
}

fn validate_pattern_size(
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
