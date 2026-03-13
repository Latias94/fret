use std::collections::HashMap;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::PreparedEdgePaintBatches;

pub(super) fn prepare_edge_paint_batches<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    render: &RenderData,
    custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
    zoom: f32,
) -> PreparedEdgePaintBatches {
    let mut batches = PreparedEdgePaintBatches::default();
    let edge_insert_marker_request = canvas
        .interaction
        .edge_insert_drag
        .as_ref()
        .map(|drag| (drag.edge, drag.pos));
    let insert_node_drag_preview = canvas.interaction.insert_node_drag_preview.as_ref();
    let elevate_edges_on_select = snapshot.interaction.elevate_edges_on_select;
    let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

    batches.edge_insert_marker = edge_insert_marker_request.and_then(|(edge_id, pos)| {
        super::marker::resolve_edge_insert_marker(
            render,
            custom_paths,
            bezier_steps,
            zoom,
            edge_id,
            pos,
        )
    });

    batches.insert_node_drag_marker = insert_node_drag_preview.as_ref().map(|preview| {
        if let Some(edge_id) = preview.edge
            && let Some((pos, color)) = super::marker::resolve_edge_insert_marker(
                render,
                custom_paths,
                bezier_steps,
                zoom,
                edge_id,
                preview.pos,
            )
        {
            (pos, color)
        } else {
            (preview.pos, canvas.style.paint.wire_color_preview)
        }
    });

    for edge in &render.edges {
        let paint = super::build::build_edge_paint(canvas, edge);
        if edge.hovered {
            batches.edges_hovered.push(paint);
        } else if edge.selected && elevate_edges_on_select {
            batches.edges_selected.push(paint);
        } else {
            batches.edges_normal.push(paint);
        }
    }

    batches
}
