use std::sync::Arc;

use crate::core::CanvasPoint;
use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_graph_view, make_test_graph_two_nodes_with_ports_spaced_x};
use crate::ui::canvas::state::{ContextMenuState, ContextMenuTarget, SearcherState};

#[test]
fn overlay_state_changes_do_not_rebuild_derived_geometry_or_spatial_index() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    // Simulate overlay state changes (context menu + searcher) without touching view-state or graph.
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin: fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(20.0)),
        invoked_at: fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(20.0)),
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Test"),
            enabled: true,
            action: NodeGraphContextMenuAction::Custom(1),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });

    canvas.interaction.searcher = Some(SearcherState {
        origin: fret_core::Point::new(fret_core::Px(30.0), fret_core::Px(40.0)),
        invoked_at: fret_core::Point::new(fret_core::Px(30.0), fret_core::Px(40.0)),
        target: ContextMenuTarget::ConnectionInsertNodePicker {
            from: a_out,
            at: CanvasPoint::default(),
        },
        query: String::new(),
        candidates: Vec::new(),
        recent_kinds: Vec::new(),
        rows: Vec::new(),
        hovered_row: None,
        active_row: 0,
        scroll: 0,
    });

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    assert!(
        Arc::ptr_eq(&geom0, &geom1),
        "expected overlay-only state changes to not rebuild derived geometry"
    );
    assert!(
        Arc::ptr_eq(&index0, &index1),
        "expected overlay-only state changes to not rebuild the spatial index"
    );

    // Sanity: geometry remains correct (ports exist).
    assert!(geom1.port_center(a_out).is_some());
    assert!(geom1.port_center(b_in).is_some());
}

#[test]
fn overlay_hover_and_scroll_updates_do_not_rebuild_derived_geometry_or_spatial_index() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    canvas.interaction.context_menu = Some(ContextMenuState {
        origin: fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(20.0)),
        invoked_at: fret_core::Point::new(fret_core::Px(10.0), fret_core::Px(20.0)),
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Test"),
            enabled: true,
            action: NodeGraphContextMenuAction::Custom(1),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });

    canvas.interaction.searcher = Some(SearcherState {
        origin: fret_core::Point::new(fret_core::Px(30.0), fret_core::Px(40.0)),
        invoked_at: fret_core::Point::new(fret_core::Px(30.0), fret_core::Px(40.0)),
        target: ContextMenuTarget::ConnectionInsertNodePicker {
            from: a_out,
            at: CanvasPoint::default(),
        },
        query: String::new(),
        candidates: Vec::new(),
        recent_kinds: Vec::new(),
        rows: Vec::new(),
        hovered_row: None,
        active_row: 0,
        scroll: 0,
    });

    // Mutate only overlay fields that are expected to be "hot" during interaction.
    for step in 0..50 {
        if let Some(menu) = canvas.interaction.context_menu.as_mut() {
            menu.hovered_item = Some(step % menu.items.len());
            menu.active_item = step % menu.items.len();
            menu.typeahead = format!("t{step}");
        }
        if let Some(searcher) = canvas.interaction.searcher.as_mut() {
            searcher.hovered_row = Some(step);
            searcher.active_row = step;
            searcher.scroll = step / 3;
            searcher.query = format!("q{step}");
        }

        let snapshot = canvas.sync_view_state(&mut host);
        let (geom, index) = canvas.canvas_derived(&host, &snapshot);
        assert!(
            Arc::ptr_eq(&geom0, &geom),
            "expected overlay-only hover/scroll updates to not rebuild derived geometry"
        );
        assert!(
            Arc::ptr_eq(&index0, &index),
            "expected overlay-only hover/scroll updates to not rebuild the spatial index"
        );
    }

    // Sanity: geometry remains correct (ports exist).
    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, _) = canvas.canvas_derived(&host, &snapshot1);
    assert!(geom1.port_center(a_out).is_some());
    assert!(geom1.port_center(b_in).is_some());
}
