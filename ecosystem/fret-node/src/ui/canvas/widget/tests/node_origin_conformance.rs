use super::{make_host_graph_view_editor_config_with, make_test_graph_two_nodes_with_size};

#[test]
fn node_origin_center_shifts_node_rect_origin() {
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view, editor_config) =
        make_host_graph_view_editor_config_with(graph_value, |state| {
            state.interaction.node_origin.x = 0.5;
            state.interaction.node_origin.y = 0.5;
        });
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let rect = geom.nodes.get(&a).expect("node a rect").rect;
    assert!((rect.origin.x.0 - (-20.0)).abs() <= 1.0e-6);
    assert!((rect.origin.y.0 - (-10.0)).abs() <= 1.0e-6);
}
