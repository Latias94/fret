use crate::ui::commands::{
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
    CMD_NODE_GRAPH_GROUP_RENAME, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK,
};
use crate::ui::presenter::NodeGraphContextMenuItem;

pub(super) fn build_group_context_menu_items() -> Vec<NodeGraphContextMenuItem> {
    vec![
        super::command_item::command_item(
            "Bring to Front",
            true,
            CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
        ),
        super::command_item::command_item("Send to Back", true, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK),
        super::command_item::command_item("Rename...", true, CMD_NODE_GRAPH_GROUP_RENAME),
        super::command_item::command_item("Delete", true, CMD_NODE_GRAPH_DELETE_SELECTION),
    ]
}
