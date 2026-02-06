use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::io::NodeGraphViewState;

use super::super::{NodeGraphCanvas, pending_drag, pending_wire_drag};
use super::{NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports};
use crate::ui::canvas::state::{
    PendingNodeDrag, PendingNodeSelectAction, PendingWireDrag, WireDragKind,
};
use crate::ui::canvas::widget::hit_test::hit_test_canvas_units_from_screen_px;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn node_drag_threshold_is_zoom_invariant_in_screen_space() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let threshold_screen = 8.0;
    let eps_screen = 0.1;

    for zoom in [0.5, 2.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
            s.interaction.node_drag_threshold = threshold_screen;
        });

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let snapshot = canvas.sync_view_state(&mut host);
        assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

        canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
            primary: a,
            nodes: vec![a],
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_pos: Point::new(Px(0.0), Px(0.0)),
            select_action: PendingNodeSelectAction::None,
            drag_enabled: true,
        });

        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds(),
            &mut prevented_default_actions,
        );

        let pos_small = Point::new(
            Px(hit_test_canvas_units_from_screen_px(
                threshold_screen - eps_screen,
                zoom,
            )),
            Px(0.0),
        );
        assert!(pending_drag::handle_pending_node_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_small,
            zoom,
        ));
        assert!(canvas.interaction.node_drag.is_none());
        assert!(canvas.interaction.pending_node_drag.is_some());

        let pos_big = Point::new(
            Px(hit_test_canvas_units_from_screen_px(
                threshold_screen + 1.0,
                zoom,
            )),
            Px(0.0),
        );
        assert!(!pending_drag::handle_pending_node_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_big,
            zoom,
        ));
        assert!(canvas.interaction.node_drag.is_some());
        assert!(canvas.interaction.pending_node_drag.is_none());

        canvas.interaction.node_drag = None;
        canvas.interaction.pending_node_drag = None;
    }
}

#[test]
fn connection_drag_threshold_is_zoom_invariant_in_screen_space() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let threshold_screen = 8.0;
    let eps_screen = 0.1;

    for zoom in [0.5, 2.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
            s.interaction.connection_drag_threshold = threshold_screen;
            s.interaction.connection_radius = 0.0;
            s.interaction.edge_interaction_width = 0.0;
        });

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let snapshot = canvas.sync_view_state(&mut host);
        assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

        canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: vec![a_out],
            },
            start_pos: Point::new(Px(0.0), Px(0.0)),
        });

        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds(),
            &mut prevented_default_actions,
        );

        let pos_small = Point::new(
            Px(hit_test_canvas_units_from_screen_px(
                threshold_screen - eps_screen,
                zoom,
            )),
            Px(0.0),
        );
        assert!(pending_wire_drag::handle_pending_wire_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_small,
            Modifiers::default(),
            zoom,
        ));
        assert!(canvas.interaction.wire_drag.is_none());
        assert!(canvas.interaction.pending_wire_drag.is_some());

        let pos_big = Point::new(
            Px(hit_test_canvas_units_from_screen_px(
                threshold_screen + 1.0,
                zoom,
            )),
            Px(0.0),
        );
        let _ = pending_wire_drag::handle_pending_wire_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_big,
            Modifiers::default(),
            zoom,
        );
        assert!(canvas.interaction.wire_drag.is_some());
        assert!(canvas.interaction.pending_wire_drag.is_none());

        canvas.interaction.wire_drag = None;
        canvas.interaction.pending_wire_drag = None;
    }
}
