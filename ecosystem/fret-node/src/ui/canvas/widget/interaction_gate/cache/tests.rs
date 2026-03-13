use super::*;
use crate::ui::canvas::state::PendingInsertNodeDrag;
use crate::ui::presenter::InsertNodeCandidate;
use fret_core::{Point, Px};
use serde_json::Value;

#[test]
fn allow_edges_cache_blocks_while_wire_insert_preview_is_active() {
    let mut interaction = InteractionState::default();
    assert!(allow_edges_cache(&interaction));

    interaction.pending_insert_node_drag = Some(PendingInsertNodeDrag {
        candidate: InsertNodeCandidate {
            kind: crate::core::NodeKindKey::new("test.node"),
            label: "Test".into(),
            enabled: true,
            template: None,
            payload: Value::Null,
        },
        start_pos: Point::new(Px(1.0), Px(2.0)),
        pointer_id: fret_core::PointerId(1),
        start_tick: fret_runtime::TickId(1),
    });
    assert!(!allow_edges_cache(&interaction));
}
