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
