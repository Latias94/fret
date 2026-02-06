//! Geometry outputs for node-graph canvas interaction.
//!
//! This module computes stable, UI-facing bounds such as node rectangles and port handle bounds.
//! These outputs are the foundation for hit-testing, edge routing, minimap/fit-view, snapping,
//! and editor features like alignment guides and marquee selection.

mod layout;
mod order;
mod origin;

pub(crate) use layout::{node_size_default_px, port_center};
pub(crate) use order::{group_order, node_order, node_ports};
pub(crate) use origin::{
    node_anchor_from_rect_origin, node_origin_offset_canvas, node_rect_origin_from_anchor,
};

use std::collections::BTreeMap;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{Graph, NodeId, PortDirection, PortId};
use crate::io::NodeGraphNodeOrigin;
use crate::ui::presenter::{NodeGraphPresenter, PortAnchorHint};
use crate::ui::style::NodeGraphStyle;

/// Geometry for a single node.
#[derive(Debug, Clone)]
pub(crate) struct NodeGeometry {
    pub(crate) rect: Rect,
}

/// Geometry for a single port handle.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PortHandleGeometry {
    pub(crate) node: NodeId,
    pub(crate) dir: PortDirection,
    pub(crate) center: Point,
    pub(crate) bounds: Rect,
}

/// Per-frame geometry outputs derived from graph + style + zoom.
#[derive(Debug, Default, Clone)]
pub(crate) struct CanvasGeometry {
    pub(crate) order: Vec<NodeId>,
    pub(crate) node_rank: BTreeMap<NodeId, u32>,
    pub(crate) nodes: BTreeMap<NodeId, NodeGeometry>,
    pub(crate) ports: BTreeMap<PortId, PortHandleGeometry>,
}

impl CanvasGeometry {
    pub(crate) fn build_with_presenter(
        graph: &Graph,
        draw_order: &[NodeId],
        style: &NodeGraphStyle,
        zoom: f32,
        node_origin: NodeGraphNodeOrigin,
        presenter: &mut dyn NodeGraphPresenter,
    ) -> Self {
        let mut out = Self::default();
        if !zoom.is_finite() || zoom <= 0.0 {
            return out;
        }
        let node_origin = node_origin.normalized();

        out.order = node_order(graph, draw_order);
        out.node_rank = out
            .order
            .iter()
            .copied()
            .enumerate()
            .map(|(ix, id)| (id, ix as u32))
            .collect();

        for node_id in out.order.iter().copied() {
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };

            let (inputs, outputs) = node_ports(graph, node_id);
            let (w_px, h_px) = node
                .size
                .map(|s| (s.width, s.height))
                .or_else(|| presenter.node_size_hint_px(graph, node_id, style))
                .unwrap_or_else(|| node_size_default_px(inputs.len(), outputs.len(), style));

            let w = w_px / zoom;
            let h = h_px / zoom;
            let off = node_origin_offset_canvas(
                crate::core::CanvasSize {
                    width: w,
                    height: h,
                },
                node_origin,
            );
            let rect = Rect::new(
                Point::new(Px(node.pos.x - off.x), Px(node.pos.y - off.y)),
                Size::new(Px(w), Px(h)),
            );

            out.nodes.insert(node_id, NodeGeometry { rect });

            for port_id in node.ports.iter().copied() {
                let Some(p) = graph.ports.get(&port_id) else {
                    continue;
                };
                let hint: Option<PortAnchorHint> =
                    presenter.port_anchor_hint(graph, node_id, port_id, style);
                let (dir, center, bounds) = if let Some(hint) = hint {
                    let center = Point::new(
                        Px(rect.origin.x.0 + hint.center.x.0 / zoom),
                        Px(rect.origin.y.0 + hint.center.y.0 / zoom),
                    );
                    let bounds = Rect::new(
                        Point::new(
                            Px(rect.origin.x.0 + hint.bounds.origin.x.0 / zoom),
                            Px(rect.origin.y.0 + hint.bounds.origin.y.0 / zoom),
                        ),
                        Size::new(
                            Px(hint.bounds.size.width.0 / zoom),
                            Px(hint.bounds.size.height.0 / zoom),
                        ),
                    );
                    (p.dir, center, bounds)
                } else {
                    let (row, dir) = match p.dir {
                        PortDirection::In => (
                            inputs.iter().position(|id| *id == port_id).unwrap_or(0),
                            PortDirection::In,
                        ),
                        PortDirection::Out => (
                            outputs.iter().position(|id| *id == port_id).unwrap_or(0),
                            PortDirection::Out,
                        ),
                    };
                    let center = port_center(rect, dir, row, style, zoom);
                    let pin_r = style.pin_radius / zoom;
                    let bounds = Rect::new(
                        Point::new(Px(center.x.0 - pin_r), Px(center.y.0 - pin_r)),
                        Size::new(Px(2.0 * pin_r), Px(2.0 * pin_r)),
                    );
                    (dir, center, bounds)
                };

                out.ports.insert(
                    port_id,
                    PortHandleGeometry {
                        node: node_id,
                        dir,
                        center,
                        bounds,
                    },
                );
            }
        }

        out
    }

    pub(crate) fn port_center(&self, port: PortId) -> Option<Point> {
        self.ports.get(&port).map(|p| p.center)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Graph, Group, GroupId, Node, NodeId, NodeKindKey,
        Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    };
    use crate::io::NodeGraphNodeOrigin;
    use crate::ui::presenter::{NodeGraphPresenter, PortAnchorHint};
    use crate::ui::style::NodeGraphStyle;
    use fret_core::{Point, Px, Rect, Size};
    use serde_json::Value;
    use std::sync::Arc;

    #[test]
    fn group_order_prefers_draw_order_then_appends_rest() {
        let mut graph = Graph::default();

        let g1 = GroupId::new();
        let g2 = GroupId::new();
        let g3 = GroupId::new();

        let group = |title: &str| Group {
            title: title.to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 10.0,
                    height: 10.0,
                },
            },
            color: None,
        };

        graph.groups.insert(g1, group("g1"));
        graph.groups.insert(g2, group("g2"));
        graph.groups.insert(g3, group("g3"));

        let order = super::group_order(&graph, &[g2, g1]);
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], g2);
        assert_eq!(order[1], g1);
        assert!(order.contains(&g3));
    }

    #[test]
    fn node_order_filters_hidden_nodes_and_dedupes_draw_order() {
        let mut graph = Graph::default();

        let n1 = NodeId::new();
        let n2 = NodeId::new();
        let n3 = NodeId::new();

        let node = |hidden: bool| Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        };

        graph.nodes.insert(n1, node(false));
        graph.nodes.insert(n2, node(true));
        graph.nodes.insert(n3, node(false));

        let order = super::node_order(&graph, &[n3, n3, n2, n1]);
        assert_eq!(order, vec![n3, n1]);
    }

    #[test]
    fn node_origin_anchor_roundtrips() {
        let size = CanvasSize {
            width: 420.0,
            height: 180.0,
        };
        let origin = NodeGraphNodeOrigin { x: 0.25, y: 0.75 };
        let anchor = CanvasPoint { x: 10.0, y: -20.0 };
        let rect_origin = super::node_rect_origin_from_anchor(anchor, size, origin);
        let round = super::node_anchor_from_rect_origin(rect_origin, size, origin);
        assert!((round.x - anchor.x).abs() <= 1.0e-6);
        assert!((round.y - anchor.y).abs() <= 1.0e-6);
    }

    #[derive(Default)]
    struct HintPresenter {
        node_w_px: f32,
        node_h_px: f32,
        hint: Option<PortAnchorHint>,
    }

    impl NodeGraphPresenter for HintPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("port")
        }

        fn node_size_hint_px(
            &mut self,
            _graph: &Graph,
            _node: NodeId,
            _style: &NodeGraphStyle,
        ) -> Option<(f32, f32)> {
            Some((self.node_w_px, self.node_h_px))
        }

        fn port_anchor_hint(
            &mut self,
            _graph: &Graph,
            _node: NodeId,
            _port: PortId,
            _style: &NodeGraphStyle,
        ) -> Option<PortAnchorHint> {
            self.hint
        }
    }

    fn assert_near(a: f32, b: f32) {
        assert!((a - b).abs() <= 1.0e-6, "{a} != {b}");
    }

    #[test]
    fn presenter_anchor_hint_is_zoom_invariant_in_window_space() {
        let mut graph = Graph::default();
        let node_id = NodeId::new();
        let port_id = PortId::new();

        graph.nodes.insert(
            node_id,
            Node {
                kind: NodeKindKey::new("test.node"),
                kind_version: 1,
                pos: CanvasPoint { x: 50.0, y: 60.0 },
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
                ports: vec![port_id],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            port_id,
            Port {
                node: node_id,
                key: PortKey::new("p"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );

        let hint = PortAnchorHint {
            center: Point::new(Px(12.0), Px(34.0)),
            bounds: Rect::new(Point::new(Px(10.0), Px(30.0)), Size::new(Px(8.0), Px(8.0))),
        };

        let style = NodeGraphStyle::default();
        let origin = NodeGraphNodeOrigin { x: 0.0, y: 0.0 };

        let mut presenter = HintPresenter {
            node_w_px: 200.0,
            node_h_px: 100.0,
            hint: Some(hint),
        };

        let zoom_a = 1.0;
        let geom_a = super::CanvasGeometry::build_with_presenter(
            &graph,
            &[node_id],
            &style,
            zoom_a,
            origin,
            &mut presenter,
        );
        let node_rect_a = geom_a.nodes.get(&node_id).unwrap().rect;
        let port_a = geom_a.ports.get(&port_id).unwrap();

        let dx_a = (port_a.center.x.0 - node_rect_a.origin.x.0) * zoom_a;
        let dy_a = (port_a.center.y.0 - node_rect_a.origin.y.0) * zoom_a;
        assert_near(dx_a, 12.0);
        assert_near(dy_a, 34.0);

        let zoom_b = 2.0;
        let geom_b = super::CanvasGeometry::build_with_presenter(
            &graph,
            &[node_id],
            &style,
            zoom_b,
            origin,
            &mut presenter,
        );
        let node_rect_b = geom_b.nodes.get(&node_id).unwrap().rect;
        let port_b = geom_b.ports.get(&port_id).unwrap();

        let dx_b = (port_b.center.x.0 - node_rect_b.origin.x.0) * zoom_b;
        let dy_b = (port_b.center.y.0 - node_rect_b.origin.y.0) * zoom_b;
        assert_near(dx_b, 12.0);
        assert_near(dy_b, 34.0);
    }
}
