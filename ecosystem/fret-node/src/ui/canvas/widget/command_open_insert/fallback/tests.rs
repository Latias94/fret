use super::*;

#[test]
fn insert_picker_fallback_canvas_point_uses_bounds_center() {
    let snapshot = ViewSnapshot {
        pan: CanvasPoint { x: 10.0, y: 20.0 },
        zoom: 2.0,
        selected_nodes: Vec::new(),
        selected_edges: Vec::new(),
        selected_groups: Vec::new(),
        draw_order: Vec::new(),
        group_draw_order: Vec::new(),
        interaction: crate::io::NodeGraphInteractionState::default(),
    };
    let bounds = Rect::new(
        Point::new(Px(100.0), Px(50.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let center = Point::new(Px(300.0), Px(150.0));

    let at = insert_picker_fallback_canvas_point::<NoopNodeGraphCanvasMiddleware>(
        &snapshot,
        Some(bounds),
    )
    .expect("fallback point");
    let expected = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::screen_to_canvas(
        bounds,
        center,
        snapshot.pan,
        snapshot.zoom,
    );

    assert_eq!(at, expected);
}

#[test]
fn insert_picker_fallback_canvas_point_returns_none_without_bounds() {
    let snapshot = ViewSnapshot {
        pan: CanvasPoint { x: 0.0, y: 0.0 },
        zoom: 1.0,
        selected_nodes: Vec::new(),
        selected_edges: Vec::new(),
        selected_groups: Vec::new(),
        draw_order: Vec::new(),
        group_draw_order: Vec::new(),
        interaction: crate::io::NodeGraphInteractionState::default(),
    };

    assert!(
        insert_picker_fallback_canvas_point::<NoopNodeGraphCanvasMiddleware>(&snapshot, None)
            .is_none()
    );
}
