use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;

use super::prelude::NodeGraphCanvas;
use super::{
    TestUiHostImpl, insert_editor_config_with, insert_view, make_test_graph_two_nodes_with_size,
};

#[test]
fn frame_view_padding_reduces_zoom_for_same_nodes() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 600.0, y: 0.0 };

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);

    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.interaction.frame_view_duration_ms = 0;
        state.interaction.frame_view_padding = 0.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_editor_config_model(editor_config);
    assert!(canvas.frame_nodes_in_view(&mut host, None, bounds, &[a, b]));
    let no_padding = canvas.sync_view_state(&mut host).zoom;

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 600.0, y: 0.0 };

    let mut host2 = TestUiHostImpl::default();
    let graph2 = host2.models.insert(graph_value);

    let view2 = insert_view(&mut host2);
    let editor_config2 = insert_editor_config_with(&mut host2, |state| {
        state.interaction.frame_view_duration_ms = 0;
        state.interaction.frame_view_padding = 0.2;
    });

    let mut canvas2 = NodeGraphCanvas::new(graph2, view2).with_editor_config_model(editor_config2);
    assert!(canvas2.frame_nodes_in_view(&mut host2, None, bounds, &[a, b]));
    let with_padding = canvas2.sync_view_state(&mut host2).zoom;

    assert!(
        with_padding < no_padding,
        "expected padding to reduce zoom (no_padding={no_padding}, with_padding={with_padding})"
    );
}
