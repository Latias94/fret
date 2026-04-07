use super::*;
use crate::ui::canvas::state::{ContextMenuTarget, InteractionState, SearcherRowsMode};
use fret_core::{Point, Px};

fn sample_context_menu_state() -> ContextMenuState {
    ContextMenuState {
        origin: Point::new(Px(0.0), Px(0.0)),
        invoked_at: Point::new(Px(0.0), Px(0.0)),
        target: ContextMenuTarget::Background,
        items: Vec::new(),
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    }
}

fn sample_searcher_state() -> SearcherState {
    SearcherState {
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
    }
}

#[test]
fn active_menu_session_requires_context_menu_or_searcher() {
    let mut interaction = InteractionState::default();
    assert!(!has_active_menu_session(&interaction));

    interaction.context_menu = Some(sample_context_menu_state());
    assert!(has_active_menu_session(&interaction));

    interaction.context_menu = None;
    interaction.searcher = Some(sample_searcher_state());
    assert!(has_active_menu_session(&interaction));
}
