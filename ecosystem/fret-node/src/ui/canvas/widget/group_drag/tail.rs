use crate::core::{CanvasPoint, CanvasRect};
use crate::ui::canvas::state::GroupDrag;
use crate::ui::canvas::widget::*;

pub(super) fn finish_group_drag_move<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::super::low_level_adapter::CanvasPaintInvalidationCx<H>,
    drag: &mut GroupDrag,
    delta: CanvasPoint,
) {
    update_drag_preview_state(drag, delta);
    canvas.interaction.group_drag = Some(drag.clone());
    super::super::low_level_adapter::invalidate_canvas_paint(cx);
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

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px};

    use super::*;
    use crate::core::{CanvasSize, GroupId, NodeId};

    fn rect(x: f32, y: f32, width: f32, height: f32) -> CanvasRect {
        CanvasRect {
            origin: CanvasPoint { x, y },
            size: CanvasSize { width, height },
        }
    }

    #[test]
    fn update_drag_preview_state_updates_rect_nodes_and_preview_rev() {
        let node = NodeId::new();
        let mut drag = GroupDrag {
            group: GroupId::new(),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            start_rect: rect(10.0, 20.0, 100.0, 80.0),
            nodes: vec![(node, CanvasPoint { x: 15.0, y: 25.0 })],
            current_rect: rect(10.0, 20.0, 100.0, 80.0),
            current_nodes: vec![(node, CanvasPoint { x: 15.0, y: 25.0 })],
            preview_rev: 0,
        };

        update_drag_preview_state(&mut drag, CanvasPoint { x: 3.0, y: 4.0 });

        assert_eq!(drag.current_rect.origin, CanvasPoint { x: 13.0, y: 24.0 });
        assert_eq!(
            drag.current_nodes,
            vec![(node, CanvasPoint { x: 18.0, y: 29.0 })]
        );
        assert_eq!(drag.preview_rev, 1);
    }

    #[test]
    fn update_drag_preview_state_skips_noop_preview_rev() {
        let node = NodeId::new();
        let mut drag = GroupDrag {
            group: GroupId::new(),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            start_rect: rect(10.0, 20.0, 100.0, 80.0),
            nodes: vec![(node, CanvasPoint { x: 15.0, y: 25.0 })],
            current_rect: rect(10.0, 20.0, 100.0, 80.0),
            current_nodes: vec![(node, CanvasPoint { x: 15.0, y: 25.0 })],
            preview_rev: 7,
        };

        update_drag_preview_state(&mut drag, CanvasPoint { x: 0.0, y: 0.0 });

        assert_eq!(drag.preview_rev, 7);
    }
}
