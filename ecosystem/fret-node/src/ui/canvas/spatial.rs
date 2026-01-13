//! Derived spatial index helpers for large node graphs.
//!
//! This module indexes ports and edges into a coarse grid in canvas space.
//! It is intended as a UI-only acceleration structure for hit-testing and interaction previews.

use fret_canvas::spatial::GridIndexWithBackrefs;
use fret_canvas::wires as canvas_wires;
use fret_core::{Point, Px, Rect, Size};

use crate::core::{EdgeId, Graph, NodeId, PortId};

use super::geometry::CanvasGeometry;

fn wire_aabb(from: Point, to: Point, zoom: f32, pad: f32) -> Rect {
    let (c1, c2) = canvas_wires::wire_ctrl_points(from, to, zoom);
    let mut min_x = from.x.0.min(to.x.0).min(c1.x.0).min(c2.x.0);
    let mut max_x = from.x.0.max(to.x.0).max(c1.x.0).max(c2.x.0);
    let mut min_y = from.y.0.min(to.y.0).min(c1.y.0).min(c2.y.0);
    let mut max_y = from.y.0.max(to.y.0).max(c1.y.0).max(c2.y.0);

    let pad = if pad.is_finite() { pad.max(0.0) } else { 0.0 };
    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

/// Coarse grid index for ports and edges (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    nodes: GridIndexWithBackrefs<NodeId>,
    ports: GridIndexWithBackrefs<PortId>,
    edges: GridIndexWithBackrefs<EdgeId>,
}

impl CanvasSpatialIndex {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: GridIndexWithBackrefs::new(1.0),
            ports: GridIndexWithBackrefs::new(1.0),
            edges: GridIndexWithBackrefs::new(1.0),
        }
    }

    pub(crate) fn build(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        max_hit_pad_canvas: f32,
        cell_size_canvas: f32,
    ) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cell_size = if cell_size_canvas.is_finite() && cell_size_canvas > 0.0 {
            cell_size_canvas
        } else {
            (256.0 / zoom).max(16.0 / zoom).max(1.0)
        };

        let mut nodes = GridIndexWithBackrefs::new(cell_size);
        let mut ports = GridIndexWithBackrefs::new(cell_size);
        let mut edges = GridIndexWithBackrefs::new(cell_size);

        // Index nodes in draw order so deterministic tie-breaking can be layered on top.
        for node_id in geom.order.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node_id) else {
                continue;
            };
            nodes.insert_rect(node_id, node_geom.rect);
        }

        // Insert ports in node draw order so that tie-breaking (when distances match) can prefer
        // the top-most node without relying on map iteration order.
        for node_id in geom.order.iter().copied() {
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };
            for port_id in node.ports.iter().copied() {
                let Some(handle) = geom.ports.get(&port_id) else {
                    continue;
                };
                ports.insert_rect(port_id, handle.bounds);
            }
        }

        let pad = max_hit_pad_canvas.max(0.0);
        for (&edge_id, edge) in &graph.edges {
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let rect = wire_aabb(from, to, zoom, pad);
            edges.insert_rect(edge_id, rect);
        }

        Self {
            nodes,
            ports,
            edges,
        }
    }

    pub(crate) fn query_ports(&self, pos: Point, radius: f32, out: &mut Vec<PortId>) {
        self.ports.query_radius(pos, radius, out);
    }

    pub(crate) fn query_edges(&self, pos: Point, radius: f32, out: &mut Vec<EdgeId>) {
        self.edges.query_radius(pos, radius, out);
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        self.edges.query_rect(rect, out);
        out.sort_unstable();
        out.dedup();
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        self.nodes.query_rect(rect, out);
        out.sort_unstable();
        out.dedup();
    }
}

impl Default for CanvasSpatialIndex {
    fn default() -> Self {
        Self::empty()
    }
}
