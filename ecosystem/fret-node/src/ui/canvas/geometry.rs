//! Geometry outputs for node-graph canvas interaction.
//!
//! This module computes stable, UI-facing bounds such as node rectangles and port handle bounds.
//! These outputs are the foundation for hit-testing, edge routing, minimap/fit-view, snapping,
//! and editor features like alignment guides and marquee selection.

use std::collections::{BTreeMap, BTreeSet};

use fret_core::{Point, Px, Rect, Size};

use crate::core::{Graph, NodeId, PortDirection, PortId};
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
    pub(crate) fn build_with_presenter(
        graph: &Graph,
        draw_order: &[NodeId],
        style: &NodeGraphStyle,
        zoom: f32,
        presenter: &mut dyn NodeGraphPresenter,
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
            let (w_px, h_px) = presenter
                .node_size_hint_px(graph, node_id, style)
                .unwrap_or_else(|| node_size_default_px(inputs.len(), outputs.len(), style));

            let w = w_px / zoom;
            let h = h_px / zoom;
            let rect = Rect::new(
                Point::new(Px(node.pos.x), Px(node.pos.y)),
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

fn node_size_default_px(
    input_count: usize,
    output_count: usize,
    style: &NodeGraphStyle,
) -> (f32, f32) {
    let rows = input_count.max(output_count) as f32;
    let base = style.node_header_height + 2.0 * style.node_padding;
    let pin_area = rows * style.pin_row_height;
    (style.node_width, base + pin_area)
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
