use crate::ui::commands::{
    CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_OPEN_INSERT_NODE,
    CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_SELECT_ALL,
};
use crate::ui::presenter::NodeGraphContextMenuItem;

pub(super) fn build_background_context_menu_items(
    paste_enabled: bool,
    has_selection: bool,
) -> Vec<NodeGraphContextMenuItem> {
    vec![
        super::command_item::command_item("Insert Node...", true, CMD_NODE_GRAPH_OPEN_INSERT_NODE),
        super::command_item::command_item("Create Group", true, CMD_NODE_GRAPH_CREATE_GROUP),
        super::command_item::command_item("Paste", paste_enabled, CMD_NODE_GRAPH_PASTE),
        super::command_item::command_item("Select All", true, CMD_NODE_GRAPH_SELECT_ALL),
        super::command_item::command_item(
            "Delete Selection",
            has_selection,
            CMD_NODE_GRAPH_DELETE_SELECTION,
        ),
    ]
}
