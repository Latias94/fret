use super::prelude::path_midpoint_and_normal;
use super::*;

#[test]
fn hit_testing_uses_custom_edge_path() {
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

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let edge_types = crate::ui::NodeGraphEdgeTypes::new().register_path(
        crate::ui::EdgeTypeKey::new("data"),
        |_graph, _edge, _style, _hint, input| {
            let z = input.zoom.max(1.0e-6);
            let bend = 400.0 / z;
            let c1 = Point::new(Px(input.from.x.0), Px(input.from.y.0 - bend));
            let c2 = Point::new(Px(input.to.x.0), Px(input.to.y.0 - bend));

            Some(crate::ui::EdgeCustomPath {
                cache_key: 1,
                commands: vec![
                    fret_core::PathCommand::MoveTo(input.from),
                    fret_core::PathCommand::CubicTo {
                        ctrl1: c1,
                        ctrl2: c2,
                        to: input.to,
                    },
                ],
            })
        },
    );

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1200.0), Px(800.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let snapshot = canvas.sync_view_state(cx.app);
    let zoom = snapshot.zoom;
    assert!((zoom - 1.0).abs() <= 1.0e-6);

    let geom = canvas.canvas_geometry(&*cx.app, &snapshot);
    let index = canvas.geometry.index.clone();

    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let custom = canvas
        .graph
        .read_ref(cx.app, |g| {
            canvas.edge_custom_path(
                g,
                edge_id,
                &canvas.edge_render_hint(g, edge_id),
                from,
                to,
                zoom,
            )
        })
        .ok()
        .flatten()
        .expect("custom path must exist");

    let (mid, _normal) = path_midpoint_and_normal(
        &custom.commands,
        usize::from(snapshot.interaction.bezier_hit_test_steps.max(1)),
    )
    .expect("midpoint must exist");

    // Sanity: this point is *far* from the default wire AABB, so without our Stage 2 path-based
    // spatial index patch it would likely not be considered in candidate queries.
    let default_aabb = fret_canvas::wires::wire_aabb(from, to, zoom, 0.0);
    assert!(!default_aabb.contains(mid));

    let graph = canvas
        .graph
        .read_ref(cx.app, |g| g.clone())
        .unwrap_or_default();
    let mut scratch = super::super::HitTestScratch::default();
    let mut ctx = super::super::HitTestCtx::new(&geom, &index, zoom, &mut scratch);
    let hit = canvas.hit_edge(&graph, &snapshot, &mut ctx, mid);
    assert_eq!(hit, Some(edge_id));
}

#[test]
fn spatial_index_includes_custom_edge_path_bounds() {
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

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let edge_types = crate::ui::NodeGraphEdgeTypes::new().register_path(
        crate::ui::EdgeTypeKey::new("data"),
        |_graph, _edge, _style, _hint, input| {
            let z = input.zoom.max(1.0e-6);
            let bend = 600.0 / z;
            let c1 = Point::new(Px(input.from.x.0), Px(input.from.y.0 - bend));
            let c2 = Point::new(Px(input.to.x.0), Px(input.to.y.0 - bend));

            Some(crate::ui::EdgeCustomPath {
                cache_key: 123,
                commands: vec![
                    fret_core::PathCommand::MoveTo(input.from),
                    fret_core::PathCommand::CubicTo {
                        ctrl1: c1,
                        ctrl2: c2,
                        to: input.to,
                    },
                ],
            })
        },
    );

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1200.0), Px(800.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let snapshot = canvas.sync_view_state(cx.app);
    let zoom = snapshot.zoom;
    assert!((zoom - 1.0).abs() <= 1.0e-6);

    let geom = canvas.canvas_geometry(&*cx.app, &snapshot);
    let index = canvas.geometry.index.clone();

    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let custom = canvas
        .graph
        .read_ref(cx.app, |g| {
            canvas.edge_custom_path(
                g,
                edge_id,
                &canvas.edge_render_hint(g, edge_id),
                from,
                to,
                zoom,
            )
        })
        .ok()
        .flatten()
        .expect("custom path must exist");

    let (mid, _normal) = path_midpoint_and_normal(
        &custom.commands,
        usize::from(snapshot.interaction.bezier_hit_test_steps.max(1)),
    )
    .expect("midpoint must exist");

    let mut candidates = Vec::new();
    let candidates = index.query_edges_sorted_dedup(mid, 1.0, &mut candidates);
    assert!(candidates.contains(&edge_id));
}

#[test]
fn custom_edge_path_generation_is_deterministic_for_identical_inputs() {
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

    let (graph, view_state) = insert_graph_view(&mut host, graph_value);
    let view = view_state.clone();

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.25;
        s.pan = CanvasPoint::default();
    });

    let edge_types = crate::ui::NodeGraphEdgeTypes::new().register_path(
        crate::ui::EdgeTypeKey::new("data"),
        |_graph, edge, _style, _hint, input| {
            let z = input.zoom.max(1.0e-6);
            let bend = 420.0 / z;
            let c1 = Point::new(Px(input.from.x.0), Px(input.from.y.0 - bend));
            let c2 = Point::new(Px(input.to.x.0), Px(input.to.y.0 - bend));

            let u = edge.0.as_u128();
            let cache_key = (((u >> 64) ^ u) as u64) ^ (z.to_bits() as u64);

            Some(crate::ui::EdgeCustomPath {
                cache_key,
                commands: vec![
                    fret_core::PathCommand::MoveTo(input.from),
                    fret_core::PathCommand::CubicTo {
                        ctrl1: c1,
                        ctrl2: c2,
                        to: input.to,
                    },
                ],
            })
        },
    );

    let mut canvas = NodeGraphCanvas::new(graph, view_state).with_edge_types(edge_types);
    let snapshot = canvas.sync_view_state(&mut host);
    let zoom = snapshot.zoom;
    assert!((zoom - 1.25).abs() <= 1.0e-6);

    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let p0 = canvas
        .graph
        .read_ref(&host, |g| {
            let hint = canvas.edge_render_hint(g, edge_id);
            canvas.edge_custom_path(g, edge_id, &hint, from, to, zoom)
        })
        .ok()
        .flatten()
        .expect("custom path must exist");
    let p1 = canvas
        .graph
        .read_ref(&host, |g| {
            let hint = canvas.edge_render_hint(g, edge_id);
            canvas.edge_custom_path(g, edge_id, &hint, from, to, zoom)
        })
        .ok()
        .flatten()
        .expect("custom path must exist");
    assert_eq!(p0.cache_key, p1.cache_key);
    assert_eq!(p0.commands, p1.commands);

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: 123.0, y: 45.0 };
    });
    let snapshot2 = canvas.sync_view_state(&mut host);
    let zoom2 = snapshot2.zoom;
    assert!((zoom2 - zoom).abs() <= 1.0e-6);

    let p2 = canvas
        .graph
        .read_ref(&host, |g| {
            let hint = canvas.edge_render_hint(g, edge_id);
            canvas.edge_custom_path(g, edge_id, &hint, from, to, zoom2)
        })
        .ok()
        .flatten()
        .expect("custom path must exist");
    assert_eq!(p0.cache_key, p2.cache_key);
    assert_eq!(p0.commands, p2.commands);
}
