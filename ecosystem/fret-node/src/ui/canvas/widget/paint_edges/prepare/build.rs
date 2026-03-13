use crate::ui::canvas::widget::*;

use super::EdgePaint;

pub(super) fn build_edge_paint<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    edge: &crate::ui::canvas::widget::paint_render_data::EdgeRender,
) -> EdgePaint {
    let mut width = canvas.style.geometry.wire_width * edge.hint.width_mul.max(0.0);
    if edge.selected {
        width *= canvas.style.paint.wire_width_selected_mul;
    }
    if edge.hovered {
        width *= canvas.style.paint.wire_width_hover_mul;
    }

    EdgePaint {
        id: edge.id,
        from: edge.from,
        to: edge.to,
        color: edge.color,
        paint: edge.paint,
        width,
        route: edge.hint.route,
        dash: edge.hint.dash,
        start_marker: edge.hint.start_marker.clone(),
        end_marker: edge.hint.end_marker.clone(),
        selected: edge.selected,
        hovered: edge.hovered,
    }
}
