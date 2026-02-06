use super::super::NodeGraphCanvas;
use super::{make_host_graph_view, make_test_graph_two_nodes_with_size};

#[test]
fn node_origin_center_shifts_node_rect_origin() {
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.node_origin.x = 0.5;
        s.interaction.node_origin.y = 0.5;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let rect = geom.nodes.get(&a).expect("node a rect").rect;
    assert!((rect.origin.x.0 - (-20.0)).abs() <= 1.0e-6);
    assert!((rect.origin.y.0 - (-10.0)).abs() <= 1.0e-6);
}
