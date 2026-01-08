use std::collections::HashSet;

use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};

use super::{NodeGraphCanvas, ViewSnapshot};

pub(super) fn handle_node_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(drag) = canvas.interaction.node_drag.clone() else {
        return false;
    };

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvas::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    let snap_to_grid = snapshot.interaction.snap_to_grid;
    let snap_grid = snapshot.interaction.snap_grid;
    let snaplines = snapshot.interaction.snaplines;
    let snaplines_threshold_screen = snapshot.interaction.snaplines_threshold;

    let start_anchor = Point::new(
        Px(drag.start_pos.x.0 - drag.grab_offset.x.0),
        Px(drag.start_pos.y.0 - drag.grab_offset.y.0),
    );
    let new_anchor = Point::new(
        Px(position.x.0 - drag.grab_offset.x.0 - auto_pan_delta.x),
        Px(position.y.0 - drag.grab_offset.y.0 - auto_pan_delta.y),
    );
    let mut delta = CanvasPoint {
        x: new_anchor.x.0 - start_anchor.x.0,
        y: new_anchor.y.0 - start_anchor.y.0,
    };

    if snap_to_grid {
        let primary_start = drag
            .nodes
            .iter()
            .find(|(id, _)| *id == drag.primary)
            .map(|(_, p)| *p)
            .unwrap_or_default();
        let primary_target = CanvasPoint {
            x: primary_start.x + delta.x,
            y: primary_start.y + delta.y,
        };
        let snapped = NodeGraphCanvas::snap_canvas_point(primary_target, snap_grid);
        delta = CanvasPoint {
            x: snapped.x - primary_start.x,
            y: snapped.y - primary_start.y,
        };
    }

    if snaplines {
        let threshold_canvas =
            if snaplines_threshold_screen.is_finite() && snaplines_threshold_screen > 0.0 {
                snaplines_threshold_screen / zoom
            } else {
                0.0
            };

        let (geom, _index) = canvas.canvas_derived(&*cx.app, snapshot);
        let drag_nodes: HashSet<GraphNodeId> = drag.nodes.iter().map(|(id, _)| *id).collect();

        let mut group: Option<Rect> = None;
        for (id, start) in &drag.nodes {
            let Some(ng) = geom.nodes.get(id) else {
                continue;
            };
            let rect0 = Rect::new(Point::new(Px(start.x), Px(start.y)), ng.rect.size);
            group = Some(match group {
                Some(r) => super::rect_union(r, rect0),
                None => rect0,
            });
        }

        let mut candidates: Vec<Rect> = Vec::new();
        for (id, ng) in geom.nodes.iter() {
            if drag_nodes.contains(id) {
                continue;
            }
            candidates.push(ng.rect);
        }

        if let Some(group0) = group {
            let moving = Rect::new(
                Point::new(
                    Px(group0.origin.x.0 + delta.x),
                    Px(group0.origin.y.0 + delta.y),
                ),
                group0.size,
            );

            let snap = super::super::snaplines::snap_delta_for_rects(
                moving,
                &candidates,
                threshold_canvas,
            );
            canvas.interaction.snap_guides =
                (snap.guides.x.is_some() || snap.guides.y.is_some()).then_some(snap.guides);

            let allow_snap = !modifiers.alt && !modifiers.alt_gr;
            if allow_snap {
                delta.x += snap.delta_x;
                delta.y += snap.delta_y;
            }
        } else {
            canvas.interaction.snap_guides = None;
        }
    } else {
        canvas.interaction.snap_guides = None;
    }

    let _ = canvas.graph.update(cx.app, |g, _cx| {
        for (id, start) in &drag.nodes {
            let Some(node) = g.nodes.get(id) else {
                continue;
            };
            let from = node.pos;
            let to = CanvasPoint {
                x: start.x + delta.x,
                y: start.y + delta.y,
            };
            if from != to {
                if let Some(node) = g.nodes.get_mut(id) {
                    node.pos = to;
                }
            }
        }
    });

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
