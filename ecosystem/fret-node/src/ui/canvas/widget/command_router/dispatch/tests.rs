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
