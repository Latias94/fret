use std::sync::Arc;

use fret_core::Color;

use crate::io::NodeGraphViewState;
use crate::ui::style::{NodeGraphBackgroundPattern, NodeGraphBackgroundStyle};
use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports};

#[test]
fn background_style_updates_do_not_rebuild_canvas_derived_geometry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view).with_style(style);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    let background = NodeGraphBackgroundStyle {
        background: Color {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        },
        grid_pattern: NodeGraphBackgroundPattern::Dots,
        grid_spacing: 48.0,
        grid_minor_color: Color {
            r: 0.20,
            g: 0.20,
            b: 0.20,
            a: 1.0,
        },
        grid_major_every: 5,
        grid_major_color: Color {
            r: 0.35,
            g: 0.35,
            b: 0.35,
            a: 1.0,
        },
        grid_line_width: 2.0,
        grid_dot_size: 2.0,
        grid_cross_size: 6.0,
    };

    assert_ne!(canvas.background_style(), background);
    canvas = canvas.with_background_style(background);
    assert_eq!(canvas.background_style(), background);

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(
        Arc::ptr_eq(&geom1, &geom2),
        "expected background style updates to preserve cached CanvasGeometry"
    );
    assert!(
        Arc::ptr_eq(&index1, &index2),
        "expected background style updates to preserve cached CanvasSpatialIndex"
    );
}
