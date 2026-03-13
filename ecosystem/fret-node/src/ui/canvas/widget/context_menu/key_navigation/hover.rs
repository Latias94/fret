use crate::ui::canvas::widget::*;

pub(super) fn sync_context_menu_hovered_item(
    menu: &mut ContextMenuState,
    hovered_item: Option<usize>,
) -> bool {
    if menu.hovered_item == hovered_item {
        return false;
    }

    menu.hovered_item = hovered_item;
    if let Some(ix) = hovered_item
        && menu.items.get(ix).is_some_and(|item| item.enabled)
    {
        menu.active_item = ix.min(menu.items.len().saturating_sub(1));
        menu.typeahead.clear();
    }
    true
}
