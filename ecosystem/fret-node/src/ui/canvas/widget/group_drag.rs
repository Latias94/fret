use fret_core::{Modifiers, Point, Px};
use fret_ui::UiHost;

use crate::core::CanvasPoint;

use super::{NodeGraphCanvas, ViewSnapshot};

pub(super) fn handle_group_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    let Some(drag) = canvas.interaction.group_drag.clone() else {
        return false;
    };

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvas::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();

    let position = Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    );

    let mut delta = CanvasPoint {
        x: position.x.0 - drag.start_pos.x.0,
        y: position.y.0 - drag.start_pos.y.0,
    };

    let allow_snap = !modifiers.alt && !modifiers.alt_gr;
    if allow_snap && snapshot.interaction.snap_to_grid {
        let origin0 = drag.start_rect.origin;
        let target = CanvasPoint {
            x: origin0.x + delta.x,
            y: origin0.y + delta.y,
        };
        let snapped = NodeGraphCanvas::snap_canvas_point(target, snapshot.interaction.snap_grid);
        delta = CanvasPoint {
            x: snapped.x - origin0.x,
            y: snapped.y - origin0.y,
        };
    }

    let _ = canvas.graph.update(cx.app, |g, _cx| {
        let Some(group) = g.groups.get_mut(&drag.group) else {
            return;
        };

        group.rect.origin = CanvasPoint {
            x: drag.start_rect.origin.x + delta.x,
            y: drag.start_rect.origin.y + delta.y,
        };

        for (node_id, start) in &drag.nodes {
            let Some(node) = g.nodes.get_mut(node_id) else {
                continue;
            };
            node.pos = CanvasPoint {
                x: start.x + delta.x,
                y: start.y + delta.y,
            };
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
