use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::io::NodeGraphSelectionMode;

use super::super::NodeGraphCanvas;
use super::{NullServices, event_cx, make_host_graph_view, make_test_graph_two_nodes_with_size};
use crate::ui::canvas::state::ViewSnapshot;

#[test]
fn marquee_partial_selects_intersecting_nodes() {
    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 1000.0, y: 0.0 };
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.selection_mode = NodeGraphSelectionMode::Partial;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Intersects node A (0..40, 0..20) but does not fully contain it.
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(super::super::left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let end = Point::new(Px(30.0), Px(10.0));
    assert!(super::super::marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(super::super::marquee::handle_left_up(&mut canvas, &mut cx));

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}

#[test]
fn marquee_full_requires_nodes_to_be_fully_contained() {
    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 1000.0, y: 0.0 };
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.selection_mode = NodeGraphSelectionMode::Full;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Intersects node A (0..40, 0..20) but does not fully contain it.
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(super::super::left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let end = Point::new(Px(30.0), Px(10.0));
    assert!(super::super::marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(super::super::marquee::handle_left_up(&mut canvas, &mut cx));

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert!(selected.is_empty());

    // Now fully contain the node.
    let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(super::super::left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));
    let end = Point::new(Px(80.0), Px(40.0));
    assert!(super::super::marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(super::super::marquee::handle_left_up(&mut canvas, &mut cx));

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}
