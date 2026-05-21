use fret_core::{CursorIcon, Point, Px, Rect, Size};
use fret_runtime::CommandId;

use crate::core::Graph;

use super::{
    NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config,
    prelude::NodeGraphCanvas,
};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn pointer_move_cursor_update_sets_close_button_cursor() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, Graph::default());
    let mut canvas = new_canvas!(host, graph, view, editor_config)
        .with_close_command(CommandId::from("node_graph.close"));

    let snapshot = canvas.sync_view_state(&mut host);
    let rect = NodeGraphCanvas::close_button_rect(snapshot.pan, snapshot.zoom);
    let position = Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    );

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        test_bounds(),
        &mut prevented_default_actions,
    );

    super::super::cursor::update_cursors(&mut canvas, &mut cx, &snapshot, position, snapshot.zoom);

    assert_eq!(cx.requested_cursor, Some(CursorIcon::Pointer));
}
