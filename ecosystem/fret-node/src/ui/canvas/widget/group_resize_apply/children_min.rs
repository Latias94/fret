use fret_ui::UiHost;

use crate::core::CanvasRect;
use crate::ui::canvas::state::GroupResize;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn min_group_resize_children_size<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    resize: &GroupResize,
    new_rect: &CanvasRect,
) -> (f32, f32) {
    let geom = canvas.canvas_geometry(&*host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut max_x = new_rect.origin.x;
            let mut max_y = new_rect.origin.y;
            for (node_id, node) in &graph.nodes {
                if node.parent != Some(resize.group) {
                    continue;
                }
                let Some(node_geom) = geom.nodes.get(node_id) else {
                    continue;
                };
                let x1 = node_geom.rect.origin.x.0 + node_geom.rect.size.width.0;
                let y1 = node_geom.rect.origin.y.0 + node_geom.rect.size.height.0;
                max_x = max_x.max(x1);
                max_y = max_y.max(y1);
            }
            (
                (max_x - new_rect.origin.x).max(0.0),
                (max_y - new_rect.origin.y).max(0.0),
            )
        })
        .ok()
        .unwrap_or_default()
}
