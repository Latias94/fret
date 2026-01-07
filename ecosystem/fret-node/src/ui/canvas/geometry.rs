//! Geometry outputs for node-graph canvas interaction.
//!
//! This module computes stable, UI-facing bounds such as node rectangles and port handle bounds.
//! These outputs are the foundation for hit-testing, edge routing, minimap/fit-view, snapping,
//! and editor features like alignment guides and marquee selection.

use std::collections::{BTreeMap, BTreeSet};

use fret_core::{Point, Px, Rect, Size};

use crate::core::{Graph, NodeId, PortDirection, PortId};
use crate::ui::style::NodeGraphStyle;

/// Geometry for a single node.
#[derive(Debug, Clone)]
pub(crate) struct NodeGeometry {
    pub(crate) rect: Rect,
}

/// Geometry for a single port handle.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PortHandleGeometry {
    pub(crate) dir: PortDirection,
    pub(crate) center: Point,
    pub(crate) bounds: Rect,
}

/// Per-frame geometry outputs derived from graph + style + zoom.
#[derive(Debug, Default, Clone)]
pub(crate) struct CanvasGeometry {
    pub(crate) order: Vec<NodeId>,
    pub(crate) nodes: BTreeMap<NodeId, NodeGeometry>,
    pub(crate) ports: BTreeMap<PortId, PortHandleGeometry>,
}

impl CanvasGeometry {
    pub(crate) fn build(
        graph: &Graph,
        draw_order: &[NodeId],
        style: &NodeGraphStyle,
        zoom: f32,
    ) -> Self {
        let mut out = Self::default();
        if !zoom.is_finite() || zoom <= 0.0 {
            return out;
        }

        out.order = node_order(graph, draw_order);
        for node_id in out.order.iter().copied() {
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };

            let (inputs, outputs) = node_ports(graph, node_id);
            let rect = node_rect(
                node.pos.x,
                node.pos.y,
                inputs.len(),
                outputs.len(),
                style,
                zoom,
            );

            out.nodes.insert(node_id, NodeGeometry { rect });

            for (dir, ports) in [(PortDirection::In, &inputs), (PortDirection::Out, &outputs)] {
                for (row, port_id) in ports.iter().copied().enumerate() {
                    let center = port_center(rect, dir, row, style, zoom);
                    let pin_r = style.pin_radius / zoom;
                    let bounds = Rect::new(
                        Point::new(Px(center.x.0 - pin_r), Px(center.y.0 - pin_r)),
                        Size::new(Px(2.0 * pin_r), Px(2.0 * pin_r)),
                    );
                    out.ports.insert(
                        port_id,
                        PortHandleGeometry {
                            dir,
                            center,
                            bounds,
                        },
                    );
                }
            }
        }

        out
    }

    pub(crate) fn port_center(&self, port: PortId) -> Option<Point> {
        self.ports.get(&port).map(|p| p.center)
    }
}

pub(crate) fn node_order(graph: &Graph, draw_order: &[NodeId]) -> Vec<NodeId> {
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut out: Vec<NodeId> = Vec::new();

    for id in draw_order {
        if graph.nodes.contains_key(id) && seen.insert(*id) {
            out.push(*id);
        }
    }

    for id in graph.nodes.keys() {
        if seen.insert(*id) {
            out.push(*id);
        }
    }

    out
}

pub(crate) fn node_ports(graph: &Graph, node: NodeId) -> (Vec<PortId>, Vec<PortId>) {
    let Some(n) = graph.nodes.get(&node) else {
        return (Vec::new(), Vec::new());
    };

    let mut inputs: Vec<PortId> = Vec::new();
    let mut outputs: Vec<PortId> = Vec::new();
    for port_id in &n.ports {
        let Some(p) = graph.ports.get(port_id) else {
            continue;
        };
        match p.dir {
            PortDirection::In => inputs.push(*port_id),
            PortDirection::Out => outputs.push(*port_id),
        }
    }

    (inputs, outputs)
}

pub(crate) fn node_rect(
    x: f32,
    y: f32,
    input_count: usize,
    output_count: usize,
    style: &NodeGraphStyle,
    zoom: f32,
) -> Rect {
    let rows = input_count.max(output_count) as f32;
    let base = style.node_header_height + 2.0 * style.node_padding;
    let pin_area = rows * style.pin_row_height;

    let w = style.node_width / zoom;
    let h = (base + pin_area) / zoom;

    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

pub(crate) fn port_center(
    node_rect: Rect,
    dir: PortDirection,
    row: usize,
    style: &NodeGraphStyle,
    zoom: f32,
) -> Point {
    let x = match dir {
        PortDirection::In => node_rect.origin.x.0,
        PortDirection::Out => node_rect.origin.x.0 + node_rect.size.width.0,
    };
    let y = node_rect.origin.y.0
        + (style.node_header_height + style.node_padding) / zoom
        + (row as f32 + 0.5) * (style.pin_row_height / zoom);

    Point::new(Px(x), Px(y))
}
