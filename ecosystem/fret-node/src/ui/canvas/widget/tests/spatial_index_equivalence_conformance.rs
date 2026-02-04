use fret_core::{Point, Px};
use uuid::Uuid;

use crate::core::{Edge, EdgeId, EdgeKind, Graph, PortId};
use crate::io::NodeGraphViewState;
use crate::ui::canvas::geometry::CanvasGeometry;

use super::super::EdgeEndpoint;
use super::super::NodeGraphCanvas;
use super::super::{HitTestCtx, HitTestScratch};
use super::super::{
    dist2_point_to_segment, path_start_end_tangents, step_wire_distance2, wire_distance2,
    wire_distance2_path,
};
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn hit_port_slow(geom: &CanvasGeometry, pos: Point) -> Option<PortId> {
    let mut best: Option<(PortId, u32)> = None;
    for (&port_id, handle) in geom.ports.iter() {
        if !NodeGraphCanvas::rect_contains_point(handle.bounds, pos) {
            continue;
        }
        let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
        match best {
            Some((best_id, best_rank)) => {
                if rank > best_rank || (rank == best_rank && port_id < best_id) {
                    best = Some((port_id, rank));
                }
            }
            None => best = Some((port_id, rank)),
        }
    }
    best.map(|(id, _)| id)
}

fn hit_edge_slow(
    canvas: &NodeGraphCanvas,
    graph: &Graph,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    geom: &CanvasGeometry,
    pos: Point,
    zoom: f32,
) -> Option<EdgeId> {
    let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
    let hit_w = super::super::hit_test::hit_test_canvas_units_from_screen_px(
        snapshot.interaction.edge_interaction_width,
        zoom,
    )
    .max(canvas.style.wire_width / super::super::hit_test::zoom_z(zoom));
    let threshold2 = hit_w * hit_w;
    let eps = super::super::hit_test::zoom_eps(zoom);

    let mut edge_ids: Vec<EdgeId> = graph.edges.keys().copied().collect();
    edge_ids.sort_unstable();

    let mut best: Option<(EdgeId, f32)> = None;
    for edge_id in edge_ids {
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };
        let Some(from) = geom.port_center(edge.from) else {
            continue;
        };
        let Some(to) = geom.port_center(edge.to) else {
            continue;
        };

        let hint = canvas.edge_render_hint(graph, edge_id);
        let d2 =
            if let Some(custom) = canvas.edge_custom_path(graph, edge_id, &hint, from, to, zoom) {
                wire_distance2_path(pos, &custom.commands, bezier_steps)
            } else {
                match hint.route {
                    crate::ui::presenter::EdgeRouteKind::Bezier => {
                        wire_distance2(pos, from, to, zoom, bezier_steps)
                    }
                    crate::ui::presenter::EdgeRouteKind::Straight => {
                        dist2_point_to_segment(pos, from, to)
                    }
                    crate::ui::presenter::EdgeRouteKind::Step => step_wire_distance2(pos, from, to),
                }
            };

        if d2 > threshold2 {
            continue;
        }

        match best {
            Some((best_id, best_d2)) => {
                if d2 + eps < best_d2 || ((d2 - best_d2).abs() <= eps && edge_id < best_id) {
                    best = Some((edge_id, d2));
                }
            }
            None => best = Some((edge_id, d2)),
        }
    }

    best.map(|(id, _)| id)
}

fn hit_edge_focus_anchor_slow(
    canvas: &NodeGraphCanvas,
    graph: &Graph,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    geom: &CanvasGeometry,
    pos: Point,
    zoom: f32,
) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
    let eps = super::super::hit_test::zoom_eps(zoom);

    let mut edge_ids: Vec<EdgeId> = graph.edges.keys().copied().collect();
    edge_ids.sort_unstable();

    let mut best: Option<(EdgeId, EdgeEndpoint, PortId, f32, u8)> = None;

    for edge_id in edge_ids {
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };
        let (allow_source, allow_target) =
            NodeGraphCanvas::edge_reconnectable_flags(edge, &snapshot.interaction);
        if !allow_source && !allow_target {
            continue;
        }

        let Some(from) = geom.port_center(edge.from) else {
            continue;
        };
        let Some(to) = geom.port_center(edge.to) else {
            continue;
        };

        let hint = canvas.edge_render_hint(graph, edge_id);
        let (a0, a1) =
            if let Some(custom) = canvas.edge_custom_path(graph, edge_id, &hint, from, to, zoom) {
                if let Some((t0, t1)) = path_start_end_tangents(&custom.commands) {
                    NodeGraphCanvas::edge_focus_anchor_centers_from_tangents(from, to, zoom, t0, t1)
                } else {
                    NodeGraphCanvas::edge_focus_anchor_centers(hint.route, from, to, zoom)
                }
            } else {
                NodeGraphCanvas::edge_focus_anchor_centers(hint.route, from, to, zoom)
            };

        let r0 = NodeGraphCanvas::edge_focus_anchor_rect(a0, zoom);
        let r1 = NodeGraphCanvas::edge_focus_anchor_rect(a1, zoom);

        let mut consider =
            |center: Point, rect: fret_core::Rect, endpoint: EdgeEndpoint, fixed: PortId| {
                if !rect.contains(pos) {
                    return;
                }
                let dx = pos.x.0 - center.x.0;
                let dy = pos.y.0 - center.y.0;
                let d2 = dx * dx + dy * dy;
                let endpoint_order = match endpoint {
                    EdgeEndpoint::From => 0u8,
                    EdgeEndpoint::To => 1u8,
                };
                match best {
                    Some((best_id, best_ep, best_fixed, best_d2, best_ep_order)) => {
                        let better = if d2 + eps < best_d2 {
                            true
                        } else if (d2 - best_d2).abs() <= eps {
                            edge_id < best_id
                                || (edge_id == best_id
                                    && (endpoint_order < best_ep_order
                                        || (endpoint_order == best_ep_order
                                            && (endpoint != best_ep || fixed < best_fixed))))
                        } else {
                            false
                        };
                        if better {
                            best = Some((edge_id, endpoint, fixed, d2, endpoint_order));
                        }
                    }
                    None => best = Some((edge_id, endpoint, fixed, d2, endpoint_order)),
                }
            };

        if allow_source {
            consider(a0, r0, EdgeEndpoint::From, edge.to);
        }
        if allow_target {
            consider(a1, r1, EdgeEndpoint::To, edge.from);
        }
    }

    best.map(|(id, endpoint, fixed, _d2, _ord)| (id, endpoint, fixed))
}

#[test]
fn spatial_index_hit_port_matches_slow_scan() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, index) = canvas.canvas_derived(&host, &snapshot);

    let handle = geom.ports.get(&b_in).expect("port handle should exist");
    let pos = handle.center;

    let mut scratch = HitTestScratch::default();
    let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
    let fast = canvas.hit_port(&mut ctx, pos);
    let slow = hit_port_slow(geom.as_ref(), pos);

    assert_eq!(fast, slow);
}

#[test]
fn spatial_index_hit_edge_matches_slow_scan() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let e1 = EdgeId(Uuid::from_u128(1));
    let e2 = EdgeId(Uuid::from_u128(2));
    graph_value.edges.insert(
        e1,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: Some(crate::core::EdgeReconnectable::Bool(true)),
        },
    );
    graph_value.edges.insert(
        e2,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: Some(crate::core::EdgeReconnectable::Bool(true)),
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edge_interaction_width = 24.0;
        s.interaction.edges_reconnectable = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, index) = canvas.canvas_derived(&host, &snapshot);

    let from = geom.port_center(a_out).expect("from center should exist");
    let to = geom.port_center(b_in).expect("to center should exist");
    let pos = Point::new(Px(0.5 * (from.x.0 + to.x.0)), Px(0.5 * (from.y.0 + to.y.0)));

    let fast = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            canvas.hit_edge(g, &snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten();

    let slow = canvas
        .graph
        .read_ref(&host, |g| {
            hit_edge_slow(&canvas, g, &snapshot, geom.as_ref(), pos, snapshot.zoom)
        })
        .ok()
        .flatten();

    assert_eq!(fast, slow);
}

#[test]
fn spatial_index_edge_focus_anchor_hit_testing_matches_slow_scan() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let e1 = EdgeId(Uuid::from_u128(1));
    let e2 = EdgeId(Uuid::from_u128(2));
    graph_value.edges.insert(
        e1,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: Some(crate::core::EdgeReconnectable::Bool(true)),
        },
    );
    graph_value.edges.insert(
        e2,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: Some(crate::core::EdgeReconnectable::Bool(true)),
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, index) = canvas.canvas_derived(&host, &snapshot);

    let from = geom.port_center(a_out).expect("from center should exist");
    let to = geom.port_center(b_in).expect("to center should exist");
    let route = canvas
        .graph
        .read_ref(&host, |g| canvas.edge_render_hint(g, e1).route)
        .unwrap();
    let (a0, _a1) = NodeGraphCanvas::edge_focus_anchor_centers(route, from, to, snapshot.zoom);
    let pos = a0;

    let fast = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten();

    let slow = canvas
        .graph
        .read_ref(&host, |g| {
            hit_edge_focus_anchor_slow(&canvas, g, &snapshot, geom.as_ref(), pos, snapshot.zoom)
        })
        .ok()
        .flatten();

    assert_eq!(fast, slow);
}
