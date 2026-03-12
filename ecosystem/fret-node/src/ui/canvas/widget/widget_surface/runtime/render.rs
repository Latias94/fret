use super::*;

pub(super) fn compute_render_cull_rect<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) -> Option<Rect> {
    if !snapshot.interaction.only_render_visible_elements {
        return None;
    }

    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 1.0e-6 {
        return None;
    }

    let viewport = NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(bounds, snapshot.pan, zoom);
    let viewport_rect = viewport.visible_canvas_rect();
    let viewport_w = viewport_rect.size.width.0;
    let viewport_h = viewport_rect.size.height.0;
    let margin_screen = canvas.style.paint.render_cull_margin_px;

    if !margin_screen.is_finite()
        || margin_screen <= 0.0
        || !viewport_w.is_finite()
        || !viewport_h.is_finite()
        || viewport_w <= 0.0
        || viewport_h <= 0.0
    {
        return None;
    }

    let margin = margin_screen / zoom;
    Some(inflate_rect(viewport_rect, margin))
}

#[cfg(test)]
pub(super) fn debug_derived_build_counters<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
) -> crate::ui::canvas::state::DerivedBuildCounters {
    canvas.geometry.counters
}

#[cfg(test)]
pub(super) fn debug_render_metrics_for_bounds<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    bounds: Rect,
) -> paint_render_data::RenderMetrics {
    let snapshot = canvas.sync_view_state(host);
    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 1.0e-6 {
        return paint_render_data::RenderMetrics::default();
    }

    let render_cull_rect = compute_render_cull_rect(canvas, &snapshot, bounds);
    let (geom, index) = canvas.canvas_derived(host, &snapshot);
    canvas
        .collect_render_data(
            host,
            &snapshot,
            geom,
            index,
            render_cull_rect,
            zoom,
            None,
            true,
            true,
            true,
        )
        .metrics
}
