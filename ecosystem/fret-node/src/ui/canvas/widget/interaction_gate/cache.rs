use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn allow_edges_cache(interaction: &InteractionState) -> bool {
    interaction.pending_wire_drag.is_none()
        && interaction.wire_drag.is_none()
        && interaction.suspended_wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && interaction.pending_insert_node_drag.is_none()
        && interaction.insert_node_drag_preview.is_none()
}

#[cfg(test)]
mod tests {
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
}
