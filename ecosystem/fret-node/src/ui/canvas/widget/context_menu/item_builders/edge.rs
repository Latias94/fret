use crate::ui::commands::{
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
};
use crate::ui::presenter::NodeGraphContextMenuItem;

pub(super) fn append_builtin_edge_context_menu_items(items: &mut Vec<NodeGraphContextMenuItem>) {
    items.push(super::command_item::command_item(
        "Insert Node...",
        true,
        CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE,
    ));
    items.push(super::command_item::command_item(
        "Insert Reroute",
        true,
        CMD_NODE_GRAPH_INSERT_REROUTE,
    ));
    items.push(super::command_item::command_item(
        "Delete",
        true,
        CMD_NODE_GRAPH_DELETE_SELECTION,
    ));
}
