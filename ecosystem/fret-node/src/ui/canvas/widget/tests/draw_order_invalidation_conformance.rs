use std::sync::Arc;

use super::{
    make_host_graph_view_editor_config, make_host_graph_view_editor_config_with,
    make_test_graph_two_nodes_with_ports,
};

#[test]
fn selection_updates_do_not_rebuild_geometry_when_elevate_nodes_on_select_is_enabled() {
    let (graph_value, a, _a_in, _a_out, b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (mut host, graph, view, editor_config) =
        make_host_graph_view_editor_config_with(graph_value, |state| {
            state.interaction.elevate_nodes_on_select = true;
        });

    let _ = view.update(&mut host, |s, _cx| {
        s.draw_order = vec![a, b];
    });

    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    for step in 0..40 {
        let _ = view.update(&mut host, |s, _cx| {
            s.selected_nodes = if step % 2 == 0 { vec![a] } else { vec![b] };
            s.draw_order = vec![a, b];
        });

        let snapshot = canvas.sync_view_state(&mut host);
        let (geom, index) = canvas.canvas_derived(&host, &snapshot);
        assert!(
            Arc::ptr_eq(&geom0, &geom),
            "expected selection updates to not rebuild derived geometry when draw_order is unchanged"
        );
        assert!(
            Arc::ptr_eq(&index0, &index),
            "expected selection updates to not rebuild the spatial index when draw_order is unchanged"
        );
    }
}

#[test]
fn draw_order_updates_rebuild_derived_geometry_and_spatial_index() {
    let (graph_value, a, _a_in, _a_out, b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (mut host, graph, view, editor_config) = make_host_graph_view_editor_config(graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.draw_order = vec![a, b];
    });

    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    let _ = view.update(&mut host, |s, _cx| {
        s.draw_order = vec![b, a];
    });

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    assert!(
        !Arc::ptr_eq(&geom0, &geom1),
        "expected draw_order changes to invalidate cached CanvasGeometry"
    );
    assert!(
        !Arc::ptr_eq(&index0, &index1),
        "expected draw_order changes to invalidate cached CanvasSpatialIndex"
    );

    assert_eq!(geom0.order, vec![a, b]);
    assert_eq!(geom1.order, vec![b, a]);
}
