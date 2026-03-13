mod background;
mod command_item;
mod edge;
mod group;

use crate::ui::presenter::NodeGraphContextMenuItem;

pub(in crate::ui::canvas::widget) fn build_group_context_menu_items()
-> Vec<NodeGraphContextMenuItem> {
    group::build_group_context_menu_items()
}

pub(in crate::ui::canvas::widget) fn build_background_context_menu_items(
    paste_enabled: bool,
    has_selection: bool,
) -> Vec<NodeGraphContextMenuItem> {
    background::build_background_context_menu_items(paste_enabled, has_selection)
}

pub(in crate::ui::canvas::widget) fn append_builtin_edge_context_menu_items(
    items: &mut Vec<NodeGraphContextMenuItem>,
) {
    edge::append_builtin_edge_context_menu_items(items)
}

#[cfg(test)]
mod tests;
