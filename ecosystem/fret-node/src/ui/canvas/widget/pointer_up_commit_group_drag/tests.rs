use fret_core::{Point, Px};

use super::*;
use crate::core::{CanvasRect, CanvasSize, GroupId};

fn point(x: f32, y: f32) -> CanvasPoint {
    CanvasPoint { x, y }
}

fn size(width: f32, height: f32) -> CanvasSize {
    CanvasSize { width, height }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> CanvasRect {
    CanvasRect {
        origin: point(x, y),
        size: size(width, height),
    }
}

#[test]
fn build_group_drag_ops_includes_group_and_moved_nodes_only() {
    let group = GroupId::new();
    let moved = NodeId::new();
    let stable = NodeId::new();
    let drag = GroupDrag {
        group,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: rect(0.0, 0.0, 200.0, 120.0),
        nodes: vec![(moved, point(10.0, 20.0)), (stable, point(30.0, 40.0))],
        current_rect: rect(20.0, 10.0, 200.0, 120.0),
        current_nodes: vec![(moved, point(30.0, 30.0)), (stable, point(30.0, 40.0))],
        preview_rev: 0,
    };

    let ops = build_group_drag_ops(&drag);

    assert_eq!(ops.len(), 2);
    assert!(ops.iter().any(|op| matches!(
        op,
        GraphOp::SetGroupRect { id, from, to }
            if *id == group
                && *from == rect(0.0, 0.0, 200.0, 120.0)
                && *to == rect(20.0, 10.0, 200.0, 120.0)
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodePos { id, from, to }
            if *id == moved && *from == point(10.0, 20.0) && *to == point(30.0, 30.0)
    )));
}
