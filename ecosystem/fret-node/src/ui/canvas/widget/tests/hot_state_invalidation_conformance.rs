use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::io::NodeGraphViewState;
use crate::rules::{DiagnosticSeverity, EdgeEndpoint};

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports};

#[test]
fn hover_state_updates_do_not_rebuild_canvas_derived_geometry_or_spatial_index() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    let edge = crate::core::EdgeId::new();
    for step in 0..80 {
        canvas.interaction.hover_port = if step % 2 == 0 {
            Some(a_out)
        } else {
            Some(b_in)
        };
        canvas.interaction.hover_port_valid = step % 3 == 0;
        canvas.interaction.hover_port_convertible = step % 5 == 0;
        canvas.interaction.hover_port_diagnostic = if step % 7 == 0 {
            Some((DiagnosticSeverity::Error, Arc::<str>::from("test")))
        } else {
            None
        };
        canvas.interaction.hover_edge = if step % 2 == 0 { Some(edge) } else { None };
        canvas.interaction.hover_edge_anchor = if step % 3 == 0 {
            Some((edge, EdgeEndpoint::From))
        } else {
            None
        };

        let snapshot = canvas.sync_view_state(&mut host);
        let (geom, index) = canvas.canvas_derived(&host, &snapshot);
        assert!(
            Arc::ptr_eq(&geom0, &geom),
            "expected hover-only state updates to not rebuild derived geometry"
        );
        assert!(
            Arc::ptr_eq(&index0, &index),
            "expected hover-only state updates to not rebuild the spatial index"
        );
    }
}

#[test]
fn selection_state_updates_do_not_rebuild_canvas_derived_geometry_or_spatial_index_when_draw_order_is_constant()
 {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.draw_order = vec![a, b];
        s.interaction.elevate_nodes_on_select = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    for step in 0..40 {
        let _ = view.update(&mut host, |s, _cx| {
            s.selected_nodes = if step % 2 == 0 { vec![a] } else { vec![b] };
            s.selected_edges = Vec::new();
            s.selected_groups = Vec::new();
            s.draw_order = vec![a, b];
        });

        let snapshot = canvas.sync_view_state(&mut host);
        let (geom, index) = canvas.canvas_derived(&host, &snapshot);
        assert!(
            Arc::ptr_eq(&geom0, &geom),
            "expected selection-only state updates to not rebuild derived geometry (when draw_order is unchanged)"
        );
        assert!(
            Arc::ptr_eq(&index0, &index),
            "expected selection-only state updates to not rebuild the spatial index (when draw_order is unchanged)"
        );
    }
}

#[test]
fn presenter_geometry_revision_rebuilds_canvas_derived_geometry_and_spatial_index() {
    #[derive(Clone)]
    struct DynamicSizePresenter {
        rev: Arc<AtomicU64>,
        w_bits: Arc<AtomicU32>,
        h_bits: Arc<AtomicU32>,
    }

    impl crate::ui::presenter::NodeGraphPresenter for DynamicSizePresenter {
        fn geometry_revision(&self) -> u64 {
            self.rev.load(Ordering::Relaxed)
        }

        fn node_title(&self, _graph: &crate::core::Graph, node: crate::core::NodeId) -> Arc<str> {
            Arc::<str>::from(format!("node {node:?}"))
        }

        fn port_label(&self, _graph: &crate::core::Graph, port: crate::core::PortId) -> Arc<str> {
            Arc::<str>::from(format!("port {port:?}"))
        }

        fn node_size_hint_px(
            &mut self,
            _graph: &crate::core::Graph,
            _node: crate::core::NodeId,
            _style: &crate::ui::NodeGraphStyle,
        ) -> Option<(f32, f32)> {
            let w = f32::from_bits(self.w_bits.load(Ordering::Relaxed));
            let h = f32::from_bits(self.h_bits.load(Ordering::Relaxed));
            Some((w, h))
        }
    }

    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let rev = Arc::new(AtomicU64::new(0));
    let w_bits = Arc::new(AtomicU32::new(420.0f32.to_bits()));
    let h_bits = Arc::new(AtomicU32::new(180.0f32.to_bits()));
    let presenter = DynamicSizePresenter {
        rev: rev.clone(),
        w_bits: w_bits.clone(),
        h_bits: h_bits.clone(),
    };

    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(presenter);
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);
    let rect0 = geom0.nodes.get(&a).expect("node must exist").rect;

    w_bits.store(520.0f32.to_bits(), Ordering::Relaxed);
    h_bits.store(240.0f32.to_bits(), Ordering::Relaxed);
    // `NodeGraphCanvas::with_presenter` wraps presenters with an auto-measured fallback store.
    // That wrapper reports `max(inner.geometry_revision(), measured.revision())`, so bump above any
    // incidental measured revision to ensure the cache key changes.
    rev.store(u64::MAX, Ordering::Relaxed);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let rect1 = geom1.nodes.get(&a).expect("node must exist").rect;

    assert!(
        !Arc::ptr_eq(&geom0, &geom1),
        "expected presenter geometry_revision change to invalidate cached CanvasGeometry"
    );
    assert!(
        !Arc::ptr_eq(&index0, &index1),
        "expected presenter geometry_revision change to invalidate cached CanvasSpatialIndex"
    );
    assert_ne!(
        rect0.size, rect1.size,
        "expected node size hint change to be reflected in derived geometry"
    );
}
