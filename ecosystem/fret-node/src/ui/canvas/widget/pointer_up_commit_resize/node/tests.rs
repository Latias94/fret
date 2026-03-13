use fret_core::{Point, Px};

use super::*;
use crate::core::{CanvasPoint, CanvasRect, CanvasSize, Group, GroupId, NodeId};
use crate::ui::canvas::state::NodeResizeHandle;

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

fn node_resize_with_group(group: GroupId, start_group_rect: CanvasRect) -> (NodeResize, Graph) {
    let node = NodeId::new();
    let mut graph = Graph::default();
    graph.groups.insert(
        group,
        Group {
            title: "Group".into(),
            rect: start_group_rect,
            color: None,
        },
    );
    (
        NodeResize {
            node,
            handle: NodeResizeHandle::Right,
            start_pos: Point::new(Px(0.0), Px(0.0)),
            start_node_pos: point(10.0, 20.0),
            start_size: size(100.0, 60.0),
            start_size_opt: Some(size(100.0, 60.0)),
            current_node_pos: point(20.0, 25.0),
            current_size_opt: Some(size(120.0, 65.0)),
            current_groups: vec![(group, rect(0.0, 0.0, 240.0, 160.0))],
            preview_rev: 0,
        },
        graph,
    )
}

#[test]
fn build_node_resize_ops_collects_node_and_group_changes() {
    let group = GroupId::new();
    let start_group_rect = rect(0.0, 0.0, 200.0, 120.0);
    let (resize, graph) = node_resize_with_group(group, start_group_rect);

    let ops = build_node_resize_ops(&resize, &graph);

    assert_eq!(ops.len(), 3);
    assert!(ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodePos { id, from, to }
            if *id == resize.node && *from == point(10.0, 20.0) && *to == point(20.0, 25.0)
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        GraphOp::SetNodeSize { id, from, to }
            if *id == resize.node
                && *from == Some(size(100.0, 60.0))
                && *to == Some(size(120.0, 65.0))
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        GraphOp::SetGroupRect { id, from, to }
            if *id == group && *from == start_group_rect && *to == rect(0.0, 0.0, 240.0, 160.0)
    )));
}
