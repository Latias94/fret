use super::*;
use crate::io::NodeGraphInteractionState;
use fret_core::{Point, Rect, Size};

#[test]
fn viewport_from_snapshot_matches_pan_zoom_constructor() {
    let bounds = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(300.0), Px(200.0)),
    );
    let snapshot = ViewSnapshot {
        pan: CanvasPoint { x: 12.0, y: -8.0 },
        zoom: 1.5,
        selected_nodes: Vec::new(),
        selected_edges: Vec::new(),
        selected_groups: Vec::new(),
        draw_order: Vec::new(),
        group_draw_order: Vec::new(),
        interaction: NodeGraphInteractionState::default(),
    };

    assert_eq!(
        viewport_from_snapshot(bounds, &snapshot),
        viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom)
    );
}
