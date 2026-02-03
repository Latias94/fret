//! Derived spatial index helpers for large node graphs.
//!
//! This module indexes ports and edges into a coarse grid in canvas space.
//! It is intended as a UI-only acceleration structure for hit-testing and interaction previews.

use fret_canvas::spatial::DefaultIndexWithBackrefs;
use fret_canvas::wires as canvas_wires;
use fret_core::{Point, Rect};
use std::collections::HashMap;

use crate::core::{EdgeId, Graph, NodeId, PortId};

use super::geometry::CanvasGeometry;

/// Coarse grid index for ports and edges (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    nodes: DefaultIndexWithBackrefs<NodeId>,
    ports: DefaultIndexWithBackrefs<PortId>,
    edges: DefaultIndexWithBackrefs<EdgeId>,
    edge_aabb_pad_canvas: f32,
    edges_by_port: HashMap<PortId, Vec<EdgeId>>,
}

impl CanvasSpatialIndex {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: DefaultIndexWithBackrefs::new(1.0),
            ports: DefaultIndexWithBackrefs::new(1.0),
            edges: DefaultIndexWithBackrefs::new(1.0),
            edge_aabb_pad_canvas: 0.0,
            edges_by_port: HashMap::new(),
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

        let mut nodes = DefaultIndexWithBackrefs::new(cell_size);
        let mut ports = DefaultIndexWithBackrefs::new(cell_size);
        let mut edges = DefaultIndexWithBackrefs::new(cell_size);
        let mut edges_by_port: HashMap<PortId, Vec<EdgeId>> = HashMap::new();

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
            if edge.from == edge.to {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
            } else {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
                edges_by_port.entry(edge.to).or_default().push(edge_id);
            }

            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let rect = canvas_wires::wire_aabb(from, to, zoom, pad);
            edges.insert_rect(edge_id, rect);
        }

        Self {
            nodes,
            ports,
            edges,
            edge_aabb_pad_canvas: pad,
            edges_by_port,
        }
    }

    pub(crate) fn query_ports_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<PortId>,
    ) -> &'a [PortId] {
        self.ports.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<EdgeId>,
    ) -> &'a [EdgeId] {
        self.edges.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        let _ = self.edges.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        let _ = self.nodes.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn edge_aabb(&self, from: Point, to: Point, zoom: f32) -> Rect {
        canvas_wires::wire_aabb(from, to, zoom, self.edge_aabb_pad_canvas)
    }

    pub(crate) fn edges_for_port(&self, port: PortId) -> Option<&[EdgeId]> {
        self.edges_by_port.get(&port).map(|v| v.as_slice())
    }

    pub(crate) fn update_node_rect(&mut self, node: NodeId, rect: Rect) {
        self.nodes.update_rect(node, rect);
    }

    pub(crate) fn update_port_rect(&mut self, port: PortId, rect: Rect) {
        self.ports.update_rect(port, rect);
    }

    pub(crate) fn update_edge_rect(&mut self, edge: EdgeId, rect: Rect) {
        self.edges.update_rect(edge, rect);
    }
}

impl Default for CanvasSpatialIndex {
    fn default() -> Self {
        Self::empty()
    }
}
