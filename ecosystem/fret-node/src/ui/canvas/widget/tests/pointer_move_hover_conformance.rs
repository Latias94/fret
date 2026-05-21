use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::Invalidation;

use crate::core::{Edge, EdgeId, EdgeKind};

use super::{
    NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config,
    make_test_graph_two_nodes_with_ports,
};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn hover_fallback_updates_hover_edge_and_invalidates_paint_once() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let mut canvas = new_canvas!(host, graph, view, editor_config);

    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");
    let mid = Point::new(Px((from.x.0 + to.x.0) * 0.5), Px((from.y.0 + to.y.0) * 0.5));

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        test_bounds(),
        &mut prevented_default_actions,
    );
    let window = AppWindowId::default();
    cx.window = Some(window);

    super::super::hover::update_hover_edge(&mut canvas, &mut cx, &snapshot, mid, snapshot.zoom);

    assert_eq!(canvas.interaction.hover_edge, Some(edge_id));
    assert!(
        cx.invalidations
            .iter()
            .any(|(_, kind)| *kind == Invalidation::Paint)
    );
    assert!(cx.app.redraw.contains(&window));

    let invalidation_count = cx.invalidations.len();
    super::super::hover::update_hover_edge(&mut canvas, &mut cx, &snapshot, mid, snapshot.zoom);

    assert_eq!(canvas.interaction.hover_edge, Some(edge_id));
    assert_eq!(cx.invalidations.len(), invalidation_count);
}
