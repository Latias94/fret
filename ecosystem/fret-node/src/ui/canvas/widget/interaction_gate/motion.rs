use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn pan_inertia_should_tick(interaction: &InteractionState) -> bool {
    if interaction.searcher.is_some() || interaction.context_menu.is_some() {
        return false;
    }
    if interaction.panning {
        return false;
    }
    interaction.pending_marquee.is_none()
        && interaction.marquee.is_none()
        && interaction.pending_node_drag.is_none()
        && interaction.node_drag.is_none()
        && interaction.pending_group_drag.is_none()
        && interaction.group_drag.is_none()
        && interaction.pending_group_resize.is_none()
        && interaction.group_resize.is_none()
        && interaction.pending_node_resize.is_none()
        && interaction.node_resize.is_none()
        && interaction.pending_wire_drag.is_none()
        && interaction.wire_drag.is_none()
        && interaction.edge_drag.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::canvas::state::{
        ContextMenuState, ContextMenuTarget, PendingNodeDrag, SearcherRowsMode, SearcherState,
    };
    use fret_core::{Point, Px};

    #[test]
    fn pan_inertia_gate_blocks_on_active_drag_or_overlay() {
        let mut interaction = InteractionState::default();
        assert!(pan_inertia_should_tick(&interaction));

        let node = crate::core::NodeId::from_u128(1);
        interaction.pending_node_drag = Some(PendingNodeDrag {
            primary: node,
            nodes: vec![node],
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            select_action: Default::default(),
            drag_enabled: true,
        });
        assert!(!pan_inertia_should_tick(&interaction));
        interaction.pending_node_drag = None;

        interaction.context_menu = Some(ContextMenuState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::Background,
            items: Vec::new(),
            candidates: Vec::new(),
            hovered_item: None,
            active_item: 0,
            typeahead: String::new(),
        });
        assert!(!pan_inertia_should_tick(&interaction));
        interaction.context_menu = None;

        interaction.searcher = Some(SearcherState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::Background,
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates: Vec::new(),
            recent_kinds: Vec::new(),
            rows: Vec::new(),
            hovered_row: None,
            active_row: 0,
            scroll: 0,
        });
        assert!(!pan_inertia_should_tick(&interaction));
    }
}
