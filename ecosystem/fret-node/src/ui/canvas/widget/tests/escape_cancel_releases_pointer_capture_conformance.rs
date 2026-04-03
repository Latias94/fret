use fret_core::{Point, Px, Rect, Size};

use super::{
    NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config,
    make_test_graph_two_nodes,
};

#[test]
fn escape_cancel_releases_pointer_capture_during_panning() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);

    let mut canvas = new_canvas!(host, graph, view, editor_config);
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

    cx.pointer_id = Some(fret_core::PointerId::default());

    let snapshot = canvas.sync_view_state(cx.app);
    let start_pos = Point::new(Px(100.0), Px(100.0));
    assert!(crate::ui::canvas::widget::pan_zoom::begin_panning(
        &mut canvas,
        &mut cx,
        &snapshot,
        start_pos,
        fret_core::MouseButton::Middle,
    ));
    assert!(
        canvas.interaction.panning,
        "expected panning to be active after begin_panning"
    );
    assert_eq!(
        cx.requested_capture,
        Some(Some(cx.node)),
        "expected begin_panning to request pointer capture"
    );

    crate::ui::canvas::widget::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    assert!(
        !canvas.interaction.panning,
        "expected escape cancel to clear panning state"
    );
    assert_eq!(
        cx.requested_capture,
        Some(None),
        "expected escape cancel to request pointer capture release"
    );
}
