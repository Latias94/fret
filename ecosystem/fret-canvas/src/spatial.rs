//! Spatial indexing helpers for large 2D canvas widgets.
//!
//! This module provides a lightweight, policy-free acceleration structure intended for:
//! - coarse culling (query items in a viewport rect),
//! - coarse hit-test candidate lookup (query items near a pointer position).
//!
//! The default implementation is a uniform grid stored in a hash map.

use std::collections::HashMap;

use fret_core::{Point, Rect};

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

/// A coarse uniform-grid spatial index (canvas space).
///
/// This structure is intentionally "dumb":
/// - insertion order is preserved inside each cell (caller decides tie-breaking by insertion order),
/// - queries may return duplicates (callers can sort/dedup when needed).
#[derive(Debug, Clone)]
pub struct GridIndex<T> {
    cell_size: f32,
    cells: HashMap<u64, Vec<T>>,
}

impl<T: Copy> GridIndex<T> {
    /// Creates a new empty grid index with the given `cell_size` (canvas units).
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: if cell_size.is_finite() && cell_size > 0.0 {
                cell_size
            } else {
                1.0
            },
            cells: HashMap::new(),
        }
    }

    /// Returns the configured cell size (canvas units).
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Removes all indexed items.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Inserts an item by its axis-aligned bounding box in canvas space.
    pub fn insert_aabb(&mut self, item: T, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                self.cells
                    .entry(cell_key(Cell { x, y }))
                    .or_default()
                    .push(item);
            }
        }
    }

    /// Inserts an item by a `Rect` in canvas space.
    ///
    /// Note: this handles negative widths/heights by computing a normalized AABB.
    pub fn insert_rect(&mut self, item: T, rect: Rect) {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        self.insert_aabb(item, min_x, min_y, max_x, max_y);
    }

    /// Appends candidate items within `radius` of `pos` (canvas space).
    pub fn query_radius(&self, pos: Point, radius: f32, out: &mut Vec<T>) {
        out.clear();
        let (x0, x1, y0, y1) = cell_range_around(pos, self.cell_size, radius);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(items) = self.cells.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(items);
                }
            }
        }
    }

    /// Appends candidate items that intersect `rect`'s AABB (canvas space).
    pub fn query_rect(&self, rect: Rect, out: &mut Vec<T>) {
        out.clear();
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        let (x0, x1, y0, y1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(items) = self.cells.get(&cell_key(Cell { x, y })) {
                    out.extend_from_slice(items);
                }
            }
        }
    }
}

impl<T: Copy> Default for GridIndex<T> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Px, Rect, Size};

    use super::*;

    #[test]
    fn grid_index_query_radius_returns_inserted_items() {
        let mut idx = GridIndex::new(10.0);
        idx.insert_aabb(1u32, 0.0, 0.0, 5.0, 5.0);

        let mut out = Vec::new();
        idx.query_radius(Point::new(Px(2.0), Px(2.0)), 1.0, &mut out);
        assert_eq!(out, vec![1]);
    }

    #[test]
    fn grid_index_query_rect_covers_aabb() {
        let mut idx = GridIndex::new(10.0);
        idx.insert_aabb(1u32, 0.0, 0.0, 5.0, 5.0);
        idx.insert_aabb(2u32, 100.0, 100.0, 105.0, 105.0);

        let mut out = Vec::new();
        idx.query_rect(
            Rect::new(
                Point::new(Px(-1.0), Px(-1.0)),
                Size::new(Px(20.0), Px(20.0)),
            ),
            &mut out,
        );
        assert!(out.contains(&1));
        assert!(!out.contains(&2));
    }
}
