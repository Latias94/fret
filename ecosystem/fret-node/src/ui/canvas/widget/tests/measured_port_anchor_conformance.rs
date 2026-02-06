use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};

use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;
use crate::ui::measured::{MeasuredGeometryApplyOptions, MeasuredGeometryExclusiveBatch};
use crate::ui::presenter::PortAnchorHint;
use crate::ui::{DefaultNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter};

use super::prelude::*;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn pick_target_port_at(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    from: crate::core::PortId,
    pos: Point,
) -> Option<crate::core::PortId> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let this = canvas;
    this.graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            this.pick_target_port(graph, snapshot, &mut ctx, from, true, pos)
        })
        .ok()
        .flatten()
}

fn make_hint(center_x: f32, center_y: f32) -> PortAnchorHint {
    let center = Point::new(Px(center_x), Px(center_y));
    let bounds = Rect::new(
        Point::new(Px(center_x - 10.0), Px(center_y - 10.0)),
        Size::new(Px(20.0), Px(20.0)),
    );
    PortAnchorHint { center, bounds }
}

#[test]
fn measured_port_anchor_hint_updates_hit_testing_in_strict_mode() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connection_mode = NodeGraphConnectionMode::Strict;
    });

    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, _index0) = canvas.canvas_derived(&host, &snapshot0);
    let old = geom0
        .ports
        .get(&b_in)
        .expect("target port handle should exist")
        .center;

    let hint = make_hint(120.0, 90.0);
    let changed = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: Vec::new(),
            port_anchors_px: vec![(b_in, hint)],
        },
        MeasuredGeometryApplyOptions::default(),
    );
    assert!(
        changed.is_some(),
        "expected measured update to bump revision"
    );

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, _index1) = canvas.canvas_derived(&host, &snapshot1);
    let new = geom1
        .ports
        .get(&b_in)
        .expect("target port handle should exist")
        .center;

    assert!(
        (new.x.0 - old.x.0).abs() > 1.0e-3 || (new.y.0 - old.y.0).abs() > 1.0e-3,
        "expected measured hint to move the port handle center"
    );

    assert_eq!(
        pick_target_port_at(&mut canvas, &mut host, &snapshot1, a_out, old),
        None,
        "expected strict pick at old center to miss after measured hint moves the handle"
    );
    assert_eq!(
        pick_target_port_at(&mut canvas, &mut host, &snapshot1, a_out, new),
        Some(b_in),
        "expected strict pick at measured center to hit the handle"
    );
}

#[test]
fn measured_port_anchor_hint_is_scaled_in_canvas_space_by_zoom() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(0.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let measured = Arc::new(MeasuredGeometryStore::new());
    let presenter =
        MeasuredNodeGraphPresenter::new(DefaultNodeGraphPresenter::default(), measured.clone());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let hint = make_hint(140.0, 60.0);
    let _ = measured.apply_exclusive_batch_if_changed(
        MeasuredGeometryExclusiveBatch {
            node_sizes_px: Vec::new(),
            port_anchors_px: vec![(b_in, hint)],
        },
        MeasuredGeometryApplyOptions::default(),
    );

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 2.0;
    });

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);

    let node_rect = geom.nodes.get(&b).expect("node geometry must exist").rect;
    let handle = geom
        .ports
        .get(&b_in)
        .expect("port handle geometry must exist");

    let expected_x = node_rect.origin.x.0 + hint.center.x.0 / snapshot.zoom;
    let expected_y = node_rect.origin.y.0 + hint.center.y.0 / snapshot.zoom;

    assert!((handle.center.x.0 - expected_x).abs() <= 1.0e-3);
    assert!((handle.center.y.0 - expected_y).abs() <= 1.0e-3);
}
