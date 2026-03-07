use crate::ui::canvas::widget::*;

pub(super) fn advance_context_menu_active_item(menu: &mut ContextMenuState, backwards: bool) {
    let item_count = menu.items.len();
    if item_count == 0 {
        return;
    }

    let mut index = if backwards {
        if menu.active_item == 0 {
            item_count - 1
        } else {
            menu.active_item - 1
        }
    } else {
        (menu.active_item + 1) % item_count
    };

    for _ in 0..item_count {
        if menu.items.get(index).is_some_and(|item| item.enabled) {
            break;
        }
        index = if backwards {
            if index == 0 {
                item_count - 1
            } else {
                index - 1
            }
        } else {
            (index + 1) % item_count
        };
    }

    menu.active_item = index;
}

pub(super) fn pop_context_menu_typeahead(menu: &mut ContextMenuState) -> bool {
    if menu.typeahead.is_empty() {
        return false;
    }
    menu.typeahead.pop();
    true
}

pub(super) fn apply_context_menu_typeahead(menu: &mut ContextMenuState, ch: char) {
    let try_find = |needle: &str| -> Option<usize> {
        if needle.is_empty() {
            return None;
        }
        menu.items.iter().position(|item| {
            item.enabled && item.label.as_ref().to_ascii_lowercase().starts_with(needle)
        })
    };

    menu.typeahead.push(ch);
    let mut needle = menu.typeahead.to_ascii_lowercase();
    let mut hit = try_find(&needle);
    if hit.is_none() {
        needle.clear();
        needle.push(ch);
        hit = try_find(&needle);
        if hit.is_some() {
            menu.typeahead.clear();
            menu.typeahead.push(ch);
        }
    }
    if let Some(index) = hit {
        menu.active_item = index.min(menu.items.len().saturating_sub(1));
    }
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

        advance_context_menu_active_item(&mut menu, false);

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

        advance_context_menu_active_item(&mut menu, true);

        assert_eq!(menu.active_item, 2);
    }

    #[test]
    fn typeahead_falls_back_to_single_character_match() {
        let mut menu = menu(vec![item("Alpha", true), item("Beta", true)], 0);
        menu.typeahead.push('a');

        apply_context_menu_typeahead(&mut menu, 'b');

        assert_eq!(menu.typeahead, "b");
        assert_eq!(menu.active_item, 1);
    }

    #[test]
    fn pop_typeahead_reports_whether_anything_changed() {
        let mut menu = menu(vec![item("Alpha", true)], 0);
        assert!(!pop_context_menu_typeahead(&mut menu));

        menu.typeahead.push('a');
        assert!(pop_context_menu_typeahead(&mut menu));
        assert!(menu.typeahead.is_empty());
    }
}
