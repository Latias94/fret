mod active_item;
mod hover;
mod key_down;
mod pointer_move;
mod typeahead;

use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(super) fn handle_context_menu_key_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    key_down::handle_context_menu_key_down_event(canvas, cx, key)
}

pub(super) fn handle_context_menu_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    pointer_move::handle_context_menu_pointer_move_event(canvas, cx, position, zoom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px};
    use fret_runtime::CommandId;

    fn item(label: &str, enabled: bool) -> NodeGraphContextMenuItem {
        NodeGraphContextMenuItem {
            label: Arc::<str>::from(label),
            enabled,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        }
    }

    fn menu(items: Vec<NodeGraphContextMenuItem>, active_item: usize) -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::Background,
            items,
            candidates: Vec::new(),
            hovered_item: None,
            active_item,
            typeahead: String::new(),
        }
    }

    #[test]
    fn advance_active_item_skips_disabled_entries() {
        let mut menu = menu(
            vec![
                item("first", false),
                item("second", true),
                item("third", false),
            ],
            0,
        );

        active_item::advance_context_menu_active_item(&mut menu, false);

        assert_eq!(menu.active_item, 1);
    }

    #[test]
    fn advance_active_item_wraps_backwards() {
        let mut menu = menu(
            vec![
                item("first", true),
                item("second", false),
                item("third", true),
            ],
            0,
        );

        active_item::advance_context_menu_active_item(&mut menu, true);

        assert_eq!(menu.active_item, 2);
    }

    #[test]
    fn typeahead_falls_back_to_single_character_match() {
        let mut menu = menu(vec![item("Alpha", true), item("Beta", true)], 0);
        menu.typeahead.push('a');

        typeahead::apply_context_menu_typeahead(&mut menu, 'b');

        assert_eq!(menu.typeahead, "b");
        assert_eq!(menu.active_item, 1);
    }

    #[test]
    fn pop_typeahead_reports_whether_anything_changed() {
        let mut menu = menu(vec![item("Alpha", true)], 0);
        assert!(!typeahead::pop_context_menu_typeahead(&mut menu));

        menu.typeahead.push('a');
        assert!(typeahead::pop_context_menu_typeahead(&mut menu));
        assert!(menu.typeahead.is_empty());
    }

    #[test]
    fn sync_hovered_item_promotes_enabled_item_and_clears_typeahead() {
        let mut menu = menu(vec![item("Alpha", true), item("Beta", true)], 0);
        menu.typeahead.push('a');

        assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
        assert_eq!(menu.hovered_item, Some(1));
        assert_eq!(menu.active_item, 1);
        assert!(menu.typeahead.is_empty());
    }

    #[test]
    fn sync_hovered_item_keeps_active_for_disabled_item() {
        let mut menu = menu(vec![item("Alpha", true), item("Beta", false)], 0);

        assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
        assert_eq!(menu.hovered_item, Some(1));
        assert_eq!(menu.active_item, 0);
    }
}
