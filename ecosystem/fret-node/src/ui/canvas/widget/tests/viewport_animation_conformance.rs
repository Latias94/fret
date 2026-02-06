use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Effect;

use crate::core::CanvasPoint;

use super::prelude::NodeGraphCanvas;
use super::{NullServices, event_cx, make_host_graph_view, make_test_graph_two_nodes};

#[test]
fn frame_view_animates_over_timer_ticks_and_reaches_target() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 5000.0, y: 0.0 };

    let (mut host, graph, view) = make_host_graph_view(graph_value);
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let did = canvas.frame_nodes_in_view(&mut host, None, bounds, &[a, b]);
    assert!(did);

    let token = canvas
        .interaction
        .viewport_animation
        .as_ref()
        .expect("viewport animation started")
        .timer;
    assert!(
        host.effects
            .iter()
            .any(|e| matches!(e, Effect::SetTimer { token: t, .. } if *t == token)),
        "expected a timer to be scheduled for the viewport animation"
    );

    let mut services = NullServices::default();
    for _ in 0..64 {
        if canvas.interaction.viewport_animation.is_none() {
            break;
        }
        let snap = canvas.sync_view_state(&mut host);
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        canvas.handle_timer(&mut cx, &snap, token);
    }
    assert!(
        canvas.interaction.viewport_animation.is_none(),
        "viewport animation should finish within a bounded number of ticks"
    );

    let final_snap = canvas.sync_view_state(&mut host);

    // Compute the exact target by running the same framing logic with duration=0 (immediate).
    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 5000.0, y: 0.0 };

    let (mut host2, graph2, view2) = make_host_graph_view(graph_value);
    let _ = view2.update(&mut host2, |s, _cx| {
        s.interaction.frame_view_duration_ms = 0;
    });
    let mut canvas2 = NodeGraphCanvas::new(graph2, view2);
    let did = canvas2.frame_nodes_in_view(&mut host2, None, bounds, &[a, b]);
    assert!(did);
    let expected = canvas2.sync_view_state(&mut host2);

    assert!((final_snap.zoom - expected.zoom).abs() <= 1.0e-3);
    assert!((final_snap.pan.x - expected.pan.x).abs() <= 1.0e-2);
    assert!((final_snap.pan.y - expected.pan.y).abs() <= 1.0e-2);
}
