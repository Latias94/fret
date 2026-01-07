//! Derived spatial index helpers for large node graphs.
//!
//! This module indexes ports and edges into a coarse grid in canvas space.
//! It is intended as a UI-only acceleration structure for hit-testing and interaction previews.

use std::collections::HashMap;

use fret_core::{Point, Px};

use crate::core::{EdgeId, Graph, PortId};

use super::geometry::CanvasGeometry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    x: i32,
    y: i32,
}

fn cell_key(cell: Cell) -> u64 {
    let x = cell.x as u32 as u64;
    let y = cell.y as u32 as u64;
    (x << 32) | y
}

fn point_to_cell(pos: Point, cell_size: f32) -> Cell {
    let s = cell_size.max(1.0e-6);
    Cell {
        x: (pos.x.0 / s).floor() as i32,
        y: (pos.y.0 / s).floor() as i32,
    }
}

fn cell_range_around(pos: Point, cell_size: f32, radius: f32) -> (i32, i32, i32, i32) {
    let s = cell_size.max(1.0e-6);
    let r = radius.max(0.0);
    let min_x = ((pos.x.0 - r) / s).floor() as i32;
    let max_x = ((pos.x.0 + r) / s).floor() as i32;
    let min_y = ((pos.y.0 - r) / s).floor() as i32;
    let max_y = ((pos.y.0 + r) / s).floor() as i32;
    (min_x, max_x, min_y, max_y)
}

fn cell_range_for_aabb(
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    cell_size: f32,
) -> (i32, i32, i32, i32) {
    let s = cell_size.max(1.0e-6);
    let min_x = (min_x / s).floor() as i32;
    let max_x = (max_x / s).floor() as i32;
    let min_y = (min_y / s).floor() as i32;
    let max_y = (max_y / s).floor() as i32;
    (min_x, max_x, min_y, max_y)
}

fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let dx = to.x.0 - from.x.0;
    let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
    let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
    let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
    let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
    (c1, c2)
}

fn wire_aabb(from: Point, to: Point, zoom: f32, pad: f32) -> (f32, f32, f32, f32) {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    let mut min_x = from.x.0.min(to.x.0).min(c1.x.0).min(c2.x.0);
    let mut max_x = from.x.0.max(to.x.0).max(c1.x.0).max(c2.x.0);
    let mut min_y = from.y.0.min(to.y.0).min(c1.y.0).min(c2.y.0);
    let mut max_y = from.y.0.max(to.y.0).max(c1.y.0).max(c2.y.0);

    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;
    (min_x, min_y, max_x, max_y)
}

/// Coarse grid index for ports and edges (canvas space).
#[derive(Debug, Clone)]
pub(crate) struct CanvasSpatialIndex {
    cell_size: f32,
    ports: HashMap<u64, Vec<PortId>>,
    edges: HashMap<u64, Vec<EdgeId>>,
}

impl CanvasSpatialIndex {
    pub(crate) fn empty() -> Self {
        Self {
            cell_size: 1.0,
            ports: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub(crate) fn build(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        max_hit_pad_canvas: f32,
    ) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cell_size = (256.0 / zoom).max(16.0 / zoom).max(1.0);
        let mut out = Self {
            cell_size,
            ports: HashMap::new(),
            edges: HashMap::new(),
        };

        for (&port_id, handle) in &geom.ports {
            let cell = point_to_cell(handle.center, cell_size);
            out.ports.entry(cell_key(cell)).or_default().push(port_id);
        }

        let pad = max_hit_pad_canvas.max(0.0);
        for (&edge_id, edge) in &graph.edges {
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let (min_x, min_y, max_x, max_y) = wire_aabb(from, to, zoom, pad);
            let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, cell_size);
            for y in cy0..=cy1 {
                for x in cx0..=cx1 {
                    out.edges
                        .entry(cell_key(Cell { x, y }))
                        .or_default()
                        .push(edge_id);
                }
            }
        }

        out
    }

    pub(crate) fn query_ports(&self, pos: Point, radius: f32, out: &mut Vec<PortId>) {
        out.clear();
        let (x0, x1, y0, y1) = cell_range_around(pos, self.cell_size, radius);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.ports.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
    }

    pub(crate) fn query_edges(&self, pos: Point, radius: f32, out: &mut Vec<EdgeId>) {
        out.clear();
        let (x0, x1, y0, y1) = cell_range_around(pos, self.cell_size, radius);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(ids) = self.edges.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(ids);
                }
            }
        }
    }
}

impl Default for CanvasSpatialIndex {
    fn default() -> Self {
        Self::empty()
    }
}
