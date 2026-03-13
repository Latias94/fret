use super::*;
use crate::ui::canvas::state::{
    ContextMenuState, ContextMenuTarget, SearcherRowsMode, SearcherState,
};
use fret_core::{Point, Px};

#[test]
fn edge_hover_anchor_blocks_during_overlays_and_pointer_sessions() {
    let mut interaction = InteractionState::default();
    assert!(allow_edge_hover_anchor(&interaction));

    interaction.edge_drag = Some(crate::ui::canvas::state::EdgeDrag {
        edge: crate::core::EdgeId::from_u128(1),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });
    assert!(!allow_edge_hover_anchor(&interaction));
    interaction.edge_drag = None;

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
    assert!(!allow_edge_hover_anchor(&interaction));
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
    assert!(!allow_edge_hover_anchor(&interaction));
}
