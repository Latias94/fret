use crate::core::{CanvasPoint, CanvasSize, NodeId as GraphNodeId};

pub(super) fn dragged_group_bounds(
    geometry: &crate::ui::canvas::geometry::CanvasGeometry,
    nodes: &[GraphNodeId],
) -> Option<(CanvasPoint, CanvasSize)> {
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    let mut any = false;

    for id in nodes {
        let Some(node_geom) = geometry.nodes.get(id) else {
            continue;
        };
        let width = node_geom.rect.size.width.0.max(0.0);
        let height = node_geom.rect.size.height.0.max(0.0);
        let x0 = node_geom.rect.origin.x.0;
        let y0 = node_geom.rect.origin.y.0;
        if !x0.is_finite() || !y0.is_finite() || !width.is_finite() || !height.is_finite() {
            continue;
        }

        any = true;
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x0 + width);
        max_y = max_y.max(y0 + height);
    }

    if !any || !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite()
    {
        return None;
    }

    Some((
        CanvasPoint { x: min_x, y: min_y },
        CanvasSize {
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        },
    ))
}
