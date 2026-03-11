use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn allow_close_button_cursor(
    has_close_command: bool,
    interaction: &InteractionState,
) -> bool {
    has_close_command && allows_pointer_detail_gestures(interaction)
}

pub(in super::super) fn allow_canvas_detail_cursor(interaction: &InteractionState) -> bool {
    allows_pointer_detail_gestures(interaction)
        && interaction.marquee.is_none()
        && interaction.context_menu.is_none()
        && interaction.searcher.is_none()
}

fn allows_pointer_detail_gestures(interaction: &InteractionState) -> bool {
    interaction.node_drag.is_none()
        && interaction.node_resize.is_none()
        && interaction.wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && !interaction.panning
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::canvas::state::{
        ContextMenuState, ContextMenuTarget, SearcherRowsMode, SearcherState,
    };
    use fret_core::{Point, Px};

    #[test]
    fn close_button_cursor_requires_command_and_detail_gestures() {
        let mut interaction = InteractionState::default();
        assert!(!allow_close_button_cursor(false, &interaction));
        assert!(allow_close_button_cursor(true, &interaction));

        interaction.panning = true;
        assert!(!allow_close_button_cursor(true, &interaction));
    }

    #[test]
    fn canvas_detail_cursor_blocks_on_overlays_and_marquee() {
        let mut interaction = InteractionState::default();
        assert!(allow_canvas_detail_cursor(&interaction));

        interaction.marquee = Some(crate::ui::canvas::state::MarqueeDrag {
            start_pos: Point::new(Px(0.0), Px(0.0)),
            pos: Point::new(Px(1.0), Px(1.0)),
        });
        assert!(!allow_canvas_detail_cursor(&interaction));
        interaction.marquee = None;

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
