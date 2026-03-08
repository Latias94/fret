use std::collections::HashSet;

use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::ui::canvas::state::NodeDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn apply_snaplines_delta<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    mut delta: CanvasPoint,
    snaplines: bool,
    snaplines_threshold_screen: f32,
    modifiers: Modifiers,
    zoom: f32,
) -> CanvasPoint {
    if !snaplines {
        canvas.interaction.snap_guides = None;
        return delta;
    }

    let threshold_canvas = canvas_units_from_screen_px(snaplines_threshold_screen, zoom);
    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let drag_nodes: HashSet<GraphNodeId> = drag.nodes.iter().map(|(id, _)| *id).collect();

    let mut group: Option<Rect> = None;
    for (id, start) in &drag.nodes {
        let Some(node_geom) = geom.nodes.get(id) else {
            continue;
        };
        let rect = Rect::new(Point::new(Px(start.x), Px(start.y)), node_geom.rect.size);
        group = Some(match group {
            Some(group_rect) => super::rect_union(group_rect, rect),
            None => rect,
        });
    }

    let mut candidates: Vec<Rect> = Vec::new();
    for (id, node_geom) in geom.nodes.iter() {
        if drag_nodes.contains(id) {
            continue;
        }
        candidates.push(node_geom.rect);
    }

    let Some(group_rect) = group else {
        canvas.interaction.snap_guides = None;
        return delta;
    };

    let moving = Rect::new(
        Point::new(
            Px(group_rect.origin.x.0 + delta.x),
            Px(group_rect.origin.y.0 + delta.y),
        ),
        group_rect.size,
    );
    let snap =
        crate::ui::canvas::snaplines::snap_delta_for_rects(moving, &candidates, threshold_canvas);
    canvas.interaction.snap_guides =
        (snap.guides.x.is_some() || snap.guides.y.is_some()).then_some(snap.guides);

    let allow_snap = !modifiers.alt && !modifiers.alt_gr;
    if allow_snap {
        delta.x += snap.delta_x;
        delta.y += snap.delta_y;
    }
    delta
}
