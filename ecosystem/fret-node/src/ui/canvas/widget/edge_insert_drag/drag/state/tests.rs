use super::update_edge_insert_drag_position;
use crate::core::EdgeId;
use crate::ui::canvas::state::{EdgeInsertDrag, InteractionState};
use fret_core::{Point, Px};

#[test]
fn update_edge_insert_drag_position_returns_false_without_active_drag() {
    let mut interaction = InteractionState::default();

    assert!(!update_edge_insert_drag_position(
        &mut interaction,
        Point::new(Px(10.0), Px(20.0)),
    ));
}

#[test]
fn update_edge_insert_drag_position_updates_active_drag_position() {
    let edge = EdgeId::new();
    let mut interaction = InteractionState {
        edge_insert_drag: Some(EdgeInsertDrag {
            edge,
            pos: Point::new(Px(1.0), Px(2.0)),
        }),
        ..Default::default()
    };

    assert!(update_edge_insert_drag_position(
        &mut interaction,
        Point::new(Px(10.0), Px(20.0)),
    ));

    let drag = interaction
        .edge_insert_drag
        .as_ref()
        .expect("edge insert drag active");
    assert_eq!(drag.edge, edge);
    assert_eq!(drag.pos, Point::new(Px(10.0), Px(20.0)));
}
