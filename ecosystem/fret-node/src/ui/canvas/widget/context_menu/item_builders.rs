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
mod tests {
    use super::*;
    use crate::ui::commands::CMD_NODE_GRAPH_SELECT_ALL;

    #[test]
    fn background_items_reflect_paste_and_selection_enablement() {
        let items = build_background_context_menu_items(false, true);

        assert_eq!(items.len(), 5);
        assert_eq!(items[2].label.as_ref(), "Paste");
        assert!(!items[2].enabled);
        assert_eq!(items[4].label.as_ref(), "Delete Selection");
        assert!(items[4].enabled);
    }

    #[test]
    fn group_items_have_expected_commands() {
        let items = build_group_context_menu_items();

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].label.as_ref(), "Bring to Front");
        assert_eq!(items[1].label.as_ref(), "Send to Back");
        assert_eq!(items[2].label.as_ref(), "Rename...");
        assert_eq!(items[3].label.as_ref(), "Delete");
        assert!(items.iter().all(|item| item.enabled));
    }

    #[test]
    fn edge_builtins_append_expected_suffix_items() {
        let mut items = vec![command_item::command_item(
            "Custom",
            true,
            CMD_NODE_GRAPH_SELECT_ALL,
        )];

        append_builtin_edge_context_menu_items(&mut items);

        assert_eq!(items.len(), 4);
        assert_eq!(items[1].label.as_ref(), "Insert Node...");
        assert_eq!(items[2].label.as_ref(), "Insert Reroute");
        assert_eq!(items[3].label.as_ref(), "Delete");
    }
}
