use crate::ui::canvas::widget::*;

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
