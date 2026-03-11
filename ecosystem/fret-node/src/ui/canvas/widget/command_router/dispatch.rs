use crate::ui::commands::{
    CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_COPY, CMD_NODE_GRAPH_CREATE_GROUP, CMD_NODE_GRAPH_CUT,
    CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_DUPLICATE, CMD_NODE_GRAPH_FOCUS_NEXT,
    CMD_NODE_GRAPH_FOCUS_NEXT_EDGE, CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_DOWN,
    CMD_NODE_GRAPH_FOCUS_PORT_LEFT, CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PORT_UP,
    CMD_NODE_GRAPH_FOCUS_PREV, CMD_NODE_GRAPH_FOCUS_PREV_EDGE, CMD_NODE_GRAPH_FOCUS_PREV_PORT,
    CMD_NODE_GRAPH_FRAME_ALL, CMD_NODE_GRAPH_FRAME_SELECTION, CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT,
    CMD_NODE_GRAPH_GROUP_RENAME, CMD_NODE_GRAPH_GROUP_SEND_TO_BACK, CMD_NODE_GRAPH_INSERT_REROUTE,
    CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER, CMD_NODE_GRAPH_OPEN_INSERT_NODE,
    CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE, CMD_NODE_GRAPH_PASTE, CMD_NODE_GRAPH_REDO,
    CMD_NODE_GRAPH_RESET_VIEW, CMD_NODE_GRAPH_SELECT_ALL, CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE,
    CMD_NODE_GRAPH_UNDO, CMD_NODE_GRAPH_ZOOM_IN, CMD_NODE_GRAPH_ZOOM_OUT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DirectCommandRoute {
    OpenInsertNode,
    CreateGroup,
    GroupBringToFront,
    GroupSendToBack,
    GroupRename,
    OpenSplitEdgeInsertNode,
    InsertReroute,
    OpenConversionPicker,
    FrameSelection,
    FrameAll,
    ResetView,
    ZoomIn,
    ZoomOut,
    ToggleConnectionMode,
    Undo,
    Redo,
    FocusNextNode,
    FocusPrevNode,
    FocusNextEdge,
    FocusPrevEdge,
    FocusNextPort,
    FocusPrevPort,
    FocusPortLeft,
    FocusPortRight,
    FocusPortUp,
    FocusPortDown,
    Activate,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Duplicate,
    DeleteSelection,
}

pub(super) fn direct_command_route(command: &str) -> Option<DirectCommandRoute> {
    match command {
        CMD_NODE_GRAPH_OPEN_INSERT_NODE => Some(DirectCommandRoute::OpenInsertNode),
        CMD_NODE_GRAPH_CREATE_GROUP => Some(DirectCommandRoute::CreateGroup),
        CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT => Some(DirectCommandRoute::GroupBringToFront),
        CMD_NODE_GRAPH_GROUP_SEND_TO_BACK => Some(DirectCommandRoute::GroupSendToBack),
        CMD_NODE_GRAPH_GROUP_RENAME => Some(DirectCommandRoute::GroupRename),
        CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE => {
            Some(DirectCommandRoute::OpenSplitEdgeInsertNode)
        }
        CMD_NODE_GRAPH_INSERT_REROUTE => Some(DirectCommandRoute::InsertReroute),
        CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER => Some(DirectCommandRoute::OpenConversionPicker),
        CMD_NODE_GRAPH_FRAME_SELECTION => Some(DirectCommandRoute::FrameSelection),
        CMD_NODE_GRAPH_FRAME_ALL => Some(DirectCommandRoute::FrameAll),
        CMD_NODE_GRAPH_RESET_VIEW => Some(DirectCommandRoute::ResetView),
        CMD_NODE_GRAPH_ZOOM_IN => Some(DirectCommandRoute::ZoomIn),
        CMD_NODE_GRAPH_ZOOM_OUT => Some(DirectCommandRoute::ZoomOut),
        CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE => Some(DirectCommandRoute::ToggleConnectionMode),
        CMD_NODE_GRAPH_UNDO => Some(DirectCommandRoute::Undo),
        CMD_NODE_GRAPH_REDO => Some(DirectCommandRoute::Redo),
        CMD_NODE_GRAPH_FOCUS_NEXT => Some(DirectCommandRoute::FocusNextNode),
        CMD_NODE_GRAPH_FOCUS_PREV => Some(DirectCommandRoute::FocusPrevNode),
        CMD_NODE_GRAPH_FOCUS_NEXT_EDGE => Some(DirectCommandRoute::FocusNextEdge),
        CMD_NODE_GRAPH_FOCUS_PREV_EDGE => Some(DirectCommandRoute::FocusPrevEdge),
        CMD_NODE_GRAPH_FOCUS_NEXT_PORT => Some(DirectCommandRoute::FocusNextPort),
        CMD_NODE_GRAPH_FOCUS_PREV_PORT => Some(DirectCommandRoute::FocusPrevPort),
        CMD_NODE_GRAPH_FOCUS_PORT_LEFT => Some(DirectCommandRoute::FocusPortLeft),
        CMD_NODE_GRAPH_FOCUS_PORT_RIGHT => Some(DirectCommandRoute::FocusPortRight),
        CMD_NODE_GRAPH_FOCUS_PORT_UP => Some(DirectCommandRoute::FocusPortUp),
        CMD_NODE_GRAPH_FOCUS_PORT_DOWN => Some(DirectCommandRoute::FocusPortDown),
        CMD_NODE_GRAPH_ACTIVATE => Some(DirectCommandRoute::Activate),
        CMD_NODE_GRAPH_SELECT_ALL | "edit.select_all" => Some(DirectCommandRoute::SelectAll),
        CMD_NODE_GRAPH_COPY | "edit.copy" => Some(DirectCommandRoute::Copy),
        CMD_NODE_GRAPH_CUT | "edit.cut" => Some(DirectCommandRoute::Cut),
        CMD_NODE_GRAPH_PASTE | "edit.paste" => Some(DirectCommandRoute::Paste),
        CMD_NODE_GRAPH_DUPLICATE => Some(DirectCommandRoute::Duplicate),
        CMD_NODE_GRAPH_DELETE_SELECTION => Some(DirectCommandRoute::DeleteSelection),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_command_route_maps_view_commands() {
        assert_eq!(
            direct_command_route(CMD_NODE_GRAPH_FRAME_SELECTION),
            Some(DirectCommandRoute::FrameSelection)
        );
        assert_eq!(
            direct_command_route(CMD_NODE_GRAPH_RESET_VIEW),
            Some(DirectCommandRoute::ResetView)
        );
        assert_eq!(
            direct_command_route(CMD_NODE_GRAPH_ZOOM_OUT),
            Some(DirectCommandRoute::ZoomOut)
        );
    }

    #[test]
    fn direct_command_route_maps_edit_aliases_to_canonical_routes() {
        assert_eq!(
            direct_command_route("edit.select_all"),
            Some(DirectCommandRoute::SelectAll)
        );
        assert_eq!(
            direct_command_route("edit.copy"),
            Some(DirectCommandRoute::Copy)
        );
        assert_eq!(
            direct_command_route("edit.cut"),
            Some(DirectCommandRoute::Cut)
        );
        assert_eq!(
            direct_command_route("edit.paste"),
            Some(DirectCommandRoute::Paste)
        );
    }

    #[test]
    fn direct_command_route_returns_none_for_unknown_commands() {
        assert_eq!(direct_command_route("node_graph.unknown"), None);
    }
}
