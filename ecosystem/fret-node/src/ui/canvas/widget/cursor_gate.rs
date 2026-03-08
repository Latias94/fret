use crate::ui::canvas::state::InteractionState;

pub(super) fn allow_close_button_cursor(
    has_close_command: bool,
    interaction: &InteractionState,
) -> bool {
    has_close_command
        && interaction.node_drag.is_none()
        && interaction.node_resize.is_none()
        && interaction.wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && !interaction.panning
}

pub(super) fn allow_canvas_detail_cursor(interaction: &InteractionState) -> bool {
    interaction.node_drag.is_none()
        && interaction.node_resize.is_none()
        && interaction.wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && !interaction.panning
        && interaction.marquee.is_none()
        && interaction.context_menu.is_none()
        && interaction.searcher.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px};

    use crate::ui::canvas::state::{
        ContextMenuState, ContextMenuTarget, InteractionState, SearcherRowsMode, SearcherState,
    };

    #[test]
    fn close_button_cursor_requires_idle_interaction_and_command() {
        let interaction = InteractionState::default();
        assert!(!allow_close_button_cursor(false, &interaction));
        assert!(allow_close_button_cursor(true, &interaction));
    }

    #[test]
    fn canvas_detail_cursor_blocks_when_searcher_or_context_menu_is_open() {
        let mut interaction = InteractionState::default();
        assert!(allow_canvas_detail_cursor(&interaction));

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
        assert!(!allow_canvas_detail_cursor(&interaction));
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
        assert!(!allow_canvas_detail_cursor(&interaction));
    }
}
