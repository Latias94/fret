use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, Graph, Node, NodeId, NodeKindKey};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::canvas::geometry::{node_size_default_px, CanvasGeometry};
use crate::ui::{DefaultNodeGraphPresenter, NodeGraphStyle};

#[test]
fn default_node_size_has_minimum_single_row_even_without_ports() {
    let style = NodeGraphStyle::default();
    let (w, h) = node_size_default_px(0, 0, &style);
    assert_eq!(w, style.geometry.node_width);
    assert_eq!(
        h,
        style.geometry.node_header_height
            + 2.0 * style.geometry.node_padding
            + style.geometry.pin_row_height
    );
}

#[test]
fn geometry_uses_default_node_size_for_portless_nodes() {
    let mut graph = Graph::default();
    let node_id = NodeId::new();
    graph.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.portless"),
            kind_version: 0,
            pos: CanvasPoint { x: 10.0, y: 20.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let style = NodeGraphStyle::default();
    let origin = NodeGraphNodeOrigin::default();
    let mut presenter = DefaultNodeGraphPresenter::default();
    let geom = CanvasGeometry::build_with_presenter(
        &graph,
        &[node_id],
        &style,
        1.0,
        origin,
        &mut presenter,
        None,
    );

    let expected = {
        let (_w, h) = node_size_default_px(0, 0, &style);
        Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(style.geometry.node_width), Px(h)),
        )
    };

    let rect = geom.nodes.get(&node_id).expect("node geometry exists").rect;
    assert_eq!(rect, expected);
}
