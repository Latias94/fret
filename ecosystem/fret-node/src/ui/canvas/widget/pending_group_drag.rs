use fret_core::Point;
use fret_ui::UiHost;

use super::threshold::exceeds_drag_threshold;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{GroupDrag, ViewSnapshot};

pub(super) fn handle_pending_group_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.group_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_group_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold;
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, zoom) {
        return true;
    }

    let nodes = canvas
        .graph
        .read_ref(cx.app, |g| {
            g.nodes
                .iter()
                .filter_map(|(id, node)| {
                    (node.parent == Some(pending.group)).then_some((*id, node.pos))
                })
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default();

    canvas.interaction.pending_group_drag = None;
    canvas.interaction.group_drag = Some(GroupDrag {
        group: pending.group,
        start_pos: pending.start_pos,
        start_rect: pending.start_rect,
        nodes: nodes.clone(),
        current_rect: pending.start_rect,
        current_nodes: nodes,
        preview_rev: 0,
    });

    false
}

pub(super) fn group_header_hit(
    rect: crate::core::CanvasRect,
    header_height_screen: f32,
    zoom: f32,
    position: Point,
) -> bool {
    let header_h =
        fret_canvas::scale::canvas_units_from_screen_px(header_height_screen, zoom).max(0.0);
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = rect.origin.x + rect.size.width;
    let y1 = rect.origin.y + header_h.min(rect.size.height.max(0.0));

    position.x.0 >= x0
        && position.y.0 >= y0
        && position.x.0 <= x1
        && position.y.0 <= y1
        && header_h > 0.0
}
