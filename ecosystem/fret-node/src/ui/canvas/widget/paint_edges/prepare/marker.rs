use std::collections::HashMap;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

pub(super) fn resolve_edge_insert_marker(
    render: &RenderData,
    custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
    bezier_steps: usize,
    zoom: f32,
    edge_id: EdgeId,
    pos: Point,
) -> Option<(Point, Color)> {
    render
        .edges
        .iter()
        .find(|edge| edge.id == edge_id)
        .map(|edge| {
            let point = custom_paths
                .get(&edge_id)
                .map(|custom| closest_point_on_path(&custom.commands, bezier_steps, pos))
                .unwrap_or_else(|| {
                    closest_point_on_edge_route(
                        edge.hint.route,
                        edge.from,
                        edge.to,
                        zoom,
                        bezier_steps,
                        pos,
                    )
                });
            (point, edge.color)
        })
}
