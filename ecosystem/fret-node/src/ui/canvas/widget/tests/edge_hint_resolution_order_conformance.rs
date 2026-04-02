use std::sync::Arc;

use fret_core::scene::DashPatternV1;
use fret_core::{Color, Point, Px, Rect, Size};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphStyle;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::tests::prelude::NodeGraphCanvas;
use crate::ui::edge_types::{EdgeTypeKey, NodeGraphEdgeTypes};
use crate::ui::presenter::{EdgeMarker, EdgeRenderHint, EdgeRouteKind, NodeGraphPresenter};
use crate::ui::skin::{NodeGraphSkin, NodeGraphSkinRef};

use super::{
    TestUiHostImpl, insert_editor_config_with, insert_view, make_test_graph_two_nodes_with_ports,
};

fn assert_close(a: f32, b: f32) {
    assert!(
        (a - b).abs() <= 1.0e-6,
        "expected {a:?} to be close to {b:?}"
    );
}

fn assert_color_eq(a: Color, b: Color) {
    assert_close(a.r, b.r);
    assert_close(a.g, b.g);
    assert_close(a.b, b.b);
    assert_close(a.a, b.a);
}

fn assert_dash_eq(a: Option<DashPatternV1>, b: Option<DashPatternV1>) {
    match (a, b) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            assert_close(a.dash.0, b.dash.0);
            assert_close(a.gap.0, b.gap.0);
            assert_close(a.phase.0, b.phase.0);
        }
        _ => panic!("expected dash patterns to match"),
    }
}

fn assert_marker_eq(a: Option<EdgeMarker>, b: Option<EdgeMarker>) {
    match (a, b) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            assert_eq!(a.kind, b.kind);
            assert_close(a.size, b.size);
        }
        _ => panic!("expected markers to match"),
    }
}

fn assert_hint_eq(a: &EdgeRenderHint, b: &EdgeRenderHint) {
    assert_eq!(a.label.as_deref(), b.label.as_deref());
    match (a.color, b.color) {
        (None, None) => {}
        (Some(a), Some(b)) => assert_color_eq(a, b),
        _ => panic!("expected colors to match"),
    }
    assert_close(a.width_mul, b.width_mul);
    assert_eq!(a.route, b.route);
    assert_marker_eq(a.start_marker.clone(), b.start_marker.clone());
    assert_marker_eq(a.end_marker.clone(), b.end_marker.clone());
    assert_dash_eq(a.dash, b.dash);
}

struct StagePresenter {
    base_hint: EdgeRenderHint,
}

impl NodeGraphPresenter for StagePresenter {
    fn node_title(&self, _graph: &crate::core::Graph, node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from(format!("node {node:?}"))
    }

    fn port_label(&self, _graph: &crate::core::Graph, port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from(format!("port {port:?}"))
    }

    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        self.base_hint.clone()
    }
}

struct StageSkin {
    expected_base: EdgeRenderHint,
    out: EdgeRenderHint,
}

impl NodeGraphSkin for StageSkin {
    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
        base: &EdgeRenderHint,
        selected: bool,
        hovered: bool,
    ) -> EdgeRenderHint {
        assert!(
            selected,
            "expected selected edge to be passed to skin stage"
        );
        assert!(hovered, "expected hovered edge to be passed to skin stage");
        assert_hint_eq(base, &self.expected_base);
        self.out.clone()
    }
}

fn collect_edges(
    canvas: &mut NodeGraphCanvas,
    host: &TestUiHostImpl,
    snapshot: &crate::ui::canvas::widget::tests::prelude::ViewSnapshot,
    hovered_edge: Option<EdgeId>,
) -> RenderData {
    let (geom, index) = canvas.canvas_derived(host, snapshot);
    canvas.collect_render_data(
        host,
        snapshot,
        geom,
        index,
        None,
        snapshot.zoom,
        hovered_edge,
        false,
        false,
        true,
    )
}

#[test]
fn edge_render_hint_is_resolved_in_stage_order_presenter_edge_types_skin() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = crate::core::CanvasPoint::default();
        s.zoom = 1.0;
        s.selected_edges = vec![edge_id];
    });

    let presenter_hint = EdgeRenderHint {
        label: Some(Arc::<str>::from("presenter")),
        color: Some(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        width_mul: 1.25,
        route: EdgeRouteKind::Straight,
        start_marker: Some(EdgeMarker::arrow(10.0)),
        end_marker: Some(EdgeMarker::arrow(11.0)),
        dash: Some(DashPatternV1::new(Px(3.0), Px(2.0), Px(1.0))),
    };

    let expected_edge_types_base = presenter_hint.clone().normalized();

    let edge_types_hint = EdgeRenderHint {
        label: Some(Arc::<str>::from("edge_types")),
        color: Some(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        width_mul: 2.5,
        route: EdgeRouteKind::Bezier,
        start_marker: Some(EdgeMarker::arrow(20.0)),
        end_marker: Some(EdgeMarker::arrow(21.0)),
        dash: Some(DashPatternV1::new(Px(6.0), Px(5.0), Px(4.0))),
    };

    let expected_skin_base = edge_types_hint.clone().normalized();

    let skin_hint = EdgeRenderHint {
        label: Some(Arc::<str>::from("skin")),
        color: Some(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        width_mul: 3.75,
        route: EdgeRouteKind::Step,
        start_marker: Some(EdgeMarker::arrow(30.0)),
        end_marker: Some(EdgeMarker::arrow(31.0)),
        dash: Some(DashPatternV1::new(Px(9.0), Px(8.0), Px(7.0))),
    };

    let edge_types = NodeGraphEdgeTypes::new()
        .with_resolver(|_g, _edge| EdgeTypeKey::new("test"))
        .register(EdgeTypeKey::new("test"), move |_g, _edge, _style, base| {
            assert_hint_eq(&base, &expected_edge_types_base);
            edge_types_hint.clone()
        });

    let skin: NodeGraphSkinRef = Arc::new(StageSkin {
        expected_base: expected_skin_base.clone(),
        out: skin_hint.clone(),
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style)
        .with_presenter(StagePresenter {
            base_hint: presenter_hint.clone(),
        })
        .with_edge_types(edge_types)
        .with_skin(skin)
        .with_editor_config_model(editor_config);

    let snapshot = canvas.sync_view_state(&mut host);
    let _ = bounds;
    let render = collect_edges(&mut canvas, &host, &snapshot, Some(edge_id));
    assert_eq!(render.edges.len(), 1, "expected one edge in RenderData");

    let edge = &render.edges[0];
    assert_eq!(edge.id, edge_id);
    assert_hint_eq(&edge.hint, &skin_hint.clone().normalized());
    assert_color_eq(edge.color, skin_hint.color.unwrap());
}
