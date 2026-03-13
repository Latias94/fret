use std::collections::BTreeMap;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, Graph, Group, GroupId};

use super::best_parent_group;

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

fn canvas_rect(x: f32, y: f32, w: f32, h: f32) -> CanvasRect {
    CanvasRect {
        origin: CanvasPoint { x, y },
        size: CanvasSize {
            width: w,
            height: h,
        },
    }
}

fn group(rect: CanvasRect) -> Group {
    Group {
        title: "Group".to_string(),
        rect,
        color: None,
    }
}

#[test]
fn best_parent_group_prefers_smallest_containing_group() {
    let outer = GroupId::new();
    let inner = GroupId::new();
    let mut graph = Graph::default();
    graph
        .groups
        .insert(outer, group(canvas_rect(0.0, 0.0, 200.0, 200.0)));
    graph
        .groups
        .insert(inner, group(canvas_rect(20.0, 20.0, 80.0, 80.0)));

    let parent = best_parent_group(rect(30.0, 30.0, 20.0, 20.0), &graph, &BTreeMap::new());

    assert_eq!(parent, Some(inner));
}

#[test]
fn best_parent_group_uses_override_rects() {
    let group_id = GroupId::new();
    let mut graph = Graph::default();
    graph
        .groups
        .insert(group_id, group(canvas_rect(200.0, 200.0, 50.0, 50.0)));
    let overrides = BTreeMap::from([(group_id, canvas_rect(0.0, 0.0, 100.0, 100.0))]);

    let parent = best_parent_group(rect(10.0, 10.0, 20.0, 20.0), &graph, &overrides);

    assert_eq!(parent, Some(group_id));
}

#[test]
fn best_parent_group_rejects_partial_overlap() {
    let group_id = GroupId::new();
    let mut graph = Graph::default();
    graph
        .groups
        .insert(group_id, group(canvas_rect(0.0, 0.0, 40.0, 40.0)));

    let parent = best_parent_group(rect(30.0, 30.0, 20.0, 20.0), &graph, &BTreeMap::new());

    assert_eq!(parent, None);
}
