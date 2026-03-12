use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect};
use crate::ui::canvas::state::GroupDrag;
use crate::ui::canvas::widget::*;

pub(super) fn finish_group_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    drag: &mut GroupDrag,
    delta: CanvasPoint,
    auto_pan_delta: CanvasPoint,
) {
    update_drag_preview_state(drag, delta);
    canvas.interaction.group_drag = Some(drag.clone());

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    super::super::paint_invalidation::invalidate_paint(cx);
}

fn update_drag_preview_state(drag: &mut GroupDrag, delta: CanvasPoint) {
    let next_rect = CanvasRect {
        origin: CanvasPoint {
            x: drag.start_rect.origin.x + delta.x,
            y: drag.start_rect.origin.y + delta.y,
        },
        size: drag.start_rect.size,
    };
    let next_nodes = drag
        .nodes
        .iter()
        .map(|(node_id, start)| {
            (
                *node_id,
                CanvasPoint {
                    x: start.x + delta.x,
                    y: start.y + delta.y,
                },
            )
        })
        .collect::<Vec<_>>();

    if drag.current_rect != next_rect || drag.current_nodes != next_nodes {
        drag.current_rect = next_rect;
        drag.current_nodes = next_nodes;
        drag.preview_rev = drag.preview_rev.wrapping_add(1);
    }
}
