use super::super::*;

fn apply_searcher_overlay_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    searcher: SearcherState,
) {
    super::super::context_menu::clear_context_menu(interaction);
    interaction.searcher = Some(searcher);
}

pub(super) fn install_searcher_overlay<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    searcher: SearcherState,
) {
    apply_searcher_overlay_state(&mut canvas.interaction, searcher);
}

pub(super) fn open_searcher_overlay<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    invoked_at: Point,
    bounds: Rect,
    snapshot: &ViewSnapshot,
    target: ContextMenuTarget,
    candidates: Vec<InsertNodeCandidate>,
    rows_mode: SearcherRowsMode,
) {
    let searcher = build_searcher_state(
        canvas,
        invoked_at,
        bounds,
        snapshot,
        target,
        candidates,
        canvas.interaction.recent_kinds.clone(),
        rows_mode,
    );
    install_searcher_overlay(canvas, searcher);
}

#[cfg(test)]
mod tests {
    use super::apply_searcher_overlay_state;
    use crate::core::{CanvasPoint, NodeKindKey};
    use crate::ui::canvas::state::{
        ContextMenuState, ContextMenuTarget, InteractionState, SearcherRowsMode, SearcherState,
    };
    use crate::ui::presenter::{
        InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
    };
    use fret_core::{Point, Px};
    use std::sync::Arc;

    fn test_context_menu() -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(4.0), Px(8.0)),
            invoked_at: Point::new(Px(4.0), Px(8.0)),
            target: ContextMenuTarget::Background,
            items: vec![NodeGraphContextMenuItem {
                label: Arc::<str>::from("Menu Item"),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(1),
            }],
            candidates: Vec::new(),
            hovered_item: None,
            active_item: 0,
            typeahead: String::new(),
        }
    }

    fn test_searcher() -> SearcherState {
        SearcherState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target: ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint { x: 10.0, y: 20.0 },
            },
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates: vec![InsertNodeCandidate {
                kind: NodeKindKey::new("math.add"),
                label: Arc::<str>::from("Math/Add"),
                enabled: true,
                template: None,
                payload: serde_json::Value::Null,
            }],
            recent_kinds: Vec::new(),
            rows: Vec::new(),
            hovered_row: None,
            active_row: 0,
            scroll: 0,
        }
    }

    #[test]
    fn install_state_replaces_context_menu_with_searcher_overlay() {
        let mut interaction = InteractionState {
            context_menu: Some(test_context_menu()),
            ..Default::default()
        };

        apply_searcher_overlay_state(&mut interaction, test_searcher());

        assert!(interaction.context_menu.is_none());
        assert!(matches!(
            interaction.searcher,
            Some(SearcherState {
                target: ContextMenuTarget::BackgroundInsertNodePicker { .. },
                ..
            })
        ));
    }

    #[test]
    fn install_state_replaces_existing_searcher_contents() {
        let mut interaction = InteractionState {
            searcher: Some(SearcherState {
                query: String::from("old"),
                ..test_searcher()
            }),
            ..Default::default()
        };

        let mut next = test_searcher();
        next.query = String::from("next");

        apply_searcher_overlay_state(&mut interaction, next);

        assert!(matches!(
            interaction.searcher,
            Some(SearcherState { query, .. }) if query == "next"
        ));
    }
}
