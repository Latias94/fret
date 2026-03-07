use std::sync::Arc;

use crate::ui::commands::{
    CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_DELETE_SELECTION,
    CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT, CMD_NODE_GRAPH_GROUP_RENAME,
    CMD_NODE_GRAPH_GROUP_SEND_TO_BACK, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_INSERT_NODE, CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_SELECT_ALL,
};
use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};

fn command_item(
    label: &'static str,
    enabled: bool,
    command: &'static str,
) -> NodeGraphContextMenuItem {
    NodeGraphContextMenuItem {
        label: Arc::<str>::from(label),
        enabled,
        action: NodeGraphContextMenuAction::Command(fret_runtime::CommandId::from(command)),
    }
}

pub(in crate::ui::canvas::widget) fn build_group_context_menu_items()
-> Vec<NodeGraphContextMenuItem> {
    vec![
        command_item("Bring to Front", true, CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT),
        command_item("Send to Back", true, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK),
        command_item("Rename...", true, CMD_NODE_GRAPH_GROUP_RENAME),
        command_item("Delete", true, CMD_NODE_GRAPH_DELETE_SELECTION),
    ]
}

pub(in crate::ui::canvas::widget) fn build_background_context_menu_items(
    paste_enabled: bool,
    has_selection: bool,
) -> Vec<NodeGraphContextMenuItem> {
    vec![
        command_item("Insert Node...", true, CMD_NODE_GRAPH_OPEN_INSERT_NODE),
        command_item("Create Group", true, CMD_NODE_GRAPH_CREATE_GROUP),
        command_item("Paste", paste_enabled, CMD_NODE_GRAPH_PASTE),
        command_item("Select All", true, CMD_NODE_GRAPH_SELECT_ALL),
        command_item(
            "Delete Selection",
            has_selection,
            CMD_NODE_GRAPH_DELETE_SELECTION,
        ),
    ]
}

pub(in crate::ui::canvas::widget) fn append_builtin_edge_context_menu_items(
    items: &mut Vec<NodeGraphContextMenuItem>,
) {
    items.push(command_item(
        "Insert Node...",
        true,
        CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    ));
    items.push(command_item(
        "Insert Reroute",
        true,
        CMD_NODE_GRAPH_INSERT_REROUTE,
    ));
    items.push(command_item(
        "Delete",
        true,
        CMD_NODE_GRAPH_DELETE_SELECTION,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut items = vec![command_item("Custom", true, CMD_NODE_GRAPH_SELECT_ALL)];

        append_builtin_edge_context_menu_items(&mut items);

        assert_eq!(items.len(), 4);
        assert_eq!(items[1].label.as_ref(), "Insert Node...");
        assert_eq!(items[2].label.as_ref(), "Insert Reroute");
        assert_eq!(items[3].label.as_ref(), "Delete");
    }
}
