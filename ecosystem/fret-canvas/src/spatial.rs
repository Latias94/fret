//! Spatial indexing helpers for large 2D canvas widgets.
//!
//! This module provides a lightweight, policy-free acceleration structure intended for:
//! - coarse culling (query items in a viewport rect),
//! - coarse hit-test candidate lookup (query items near a pointer position).
//!
//! The default implementation is a uniform grid stored in a hash map.

use std::collections::HashMap;
use std::hash::Hash;

use fret_core::{Point, Rect};

/// Runs a spatial query into `scratch`, then sorts and deduplicates the results in-place.
///
/// Many coarse spatial indices intentionally allow duplicates and preserve insertion order inside
/// buckets. This helper provides a deterministic "candidate set" suitable for hit-testing and
/// other workflows that require stable ordering.
pub fn query_sorted_dedup<T: Ord>(scratch: &mut Vec<T>, query: impl FnOnce(&mut Vec<T>)) -> &[T] {
    scratch.clear();
    query(scratch);
    scratch.sort_unstable();
    scratch.dedup();
    scratch.as_slice()
}

#[cfg(feature = "rstar")]
use crate::spatial_rstar::RStarIndexWithBackrefs;

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

/// A coarse uniform-grid index that supports incremental updates (remove/move) via back-references.
///
/// Tradeoffs:
/// - Update/removal is O(number_of_cells_for_item * cell_occupancy) due to linear removals.
/// - Memory overhead: tracks per-item cell lists.
///
/// This is a pragmatic baseline for editor canvases that need to update a small subset of items
/// (e.g. dragging a handful of nodes) without rebuilding the entire index.
#[derive(Debug, Clone)]
pub struct GridIndexWithBackrefs<T> {
    cell_size: f32,
    cells: HashMap<u64, Vec<T>>,
    item_cells: HashMap<T, Vec<(u64, usize)>>,
}

impl<T: Copy + Eq + Hash> GridIndexWithBackrefs<T> {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: if cell_size.is_finite() && cell_size > 0.0 {
                cell_size
            } else {
                1.0
            },
            cells: HashMap::new(),
            item_cells: HashMap::new(),
        }
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.item_cells.clear();
    }

    pub fn insert_aabb(&mut self, item: T, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.remove(item);

        let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);
        let mut entries = Vec::new();
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                let key = cell_key(Cell { x, y });
                let cell = self.cells.entry(key).or_default();
                let index = cell.len();
                cell.push(item);
                entries.push((key, index));
            }
        }
        self.item_cells.insert(item, entries);
    }

    pub fn update_aabb(&mut self, item: T, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        let Some(existing) = self.item_cells.get(&item) else {
            self.insert_aabb(item, min_x, min_y, max_x, max_y);
            return;
        };

        let (cx0, cx1, cy0, cy1) = cell_range_for_aabb(min_x, min_y, max_x, max_y, self.cell_size);

        // Avoid allocating a temporary `keys` buffer by comparing the expected cell sequence
        // directly against the stored backrefs. `insert_aabb` emits entries in y-major order.
        let cols = cx1.saturating_sub(cx0).saturating_add(1).max(0) as usize;
        let rows = cy1.saturating_sub(cy0).saturating_add(1).max(0) as usize;
        let expected_len = cols.saturating_mul(rows);

        let mut same = existing.len() == expected_len;
        if same {
            let mut i = 0usize;
            'cells: for y in cy0..=cy1 {
                for x in cx0..=cx1 {
                    let key = cell_key(Cell { x, y });
                    if existing[i].0 != key {
                        same = false;
                        break 'cells;
                    }
                    i += 1;
                }
            }
        }

        if same {
            return;
        }

        self.insert_aabb(item, min_x, min_y, max_x, max_y);
    }

    pub fn insert_rect(&mut self, item: T, rect: Rect) {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        self.insert_aabb(item, min_x, min_y, max_x, max_y);
    }

    pub fn update_rect(&mut self, item: T, rect: Rect) {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        self.update_aabb(item, min_x, min_y, max_x, max_y);
    }

    pub fn remove(&mut self, item: T) -> bool {
        let Some(entries) = self.item_cells.remove(&item) else {
            return false;
        };
        for (key, index) in entries {
            if let Some(items) = self.cells.get_mut(&key) {
                if index >= items.len() {
                    continue;
                }
                let removed = items.swap_remove(index);
                if index < items.len() {
                    let moved = items[index];
                    if let Some(moved_entries) = self.item_cells.get_mut(&moved) {
                        if let Some(entry) = moved_entries.iter_mut().find(|e| e.0 == key) {
                            entry.1 = index;
                        }
                    }
                }
                debug_assert!(
                    removed == item,
                    "spatial index backrefs must remove the intended item"
                );
                if items.is_empty() {
                    self.cells.remove(&key);
                }
            }
        }
        true
    }

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

/// Default incremental spatial index backend used by canvas widgets.
///
/// This is a small wrapper that keeps call sites stable while allowing the backend to be swapped
/// via a feature flag:
/// - default: uniform grid (`GridIndexWithBackrefs`)
/// - `fret-canvas/rstar`: R-tree (`RStarIndexWithBackrefs`)
#[derive(Debug, Clone)]
pub struct DefaultIndexWithBackrefs<T: Copy> {
    #[cfg(feature = "rstar")]
    inner: RStarIndexWithBackrefs<T>,
    #[cfg(not(feature = "rstar"))]
    inner: GridIndexWithBackrefs<T>,
}

impl<T: Copy + Eq + Hash> DefaultIndexWithBackrefs<T> {
    pub fn new(cell_size: f32) -> Self {
        Self {
            #[cfg(feature = "rstar")]
            inner: {
                let _ = cell_size;
                RStarIndexWithBackrefs::new()
            },
            #[cfg(not(feature = "rstar"))]
            inner: GridIndexWithBackrefs::new(cell_size),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        #[cfg(feature = "rstar")]
        {
            "rstar"
        }
        #[cfg(not(feature = "rstar"))]
        {
            "grid"
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn insert_rect(&mut self, item: T, rect: Rect) {
        self.inner.insert_rect(item, rect);
    }

    pub fn update_rect(&mut self, item: T, rect: Rect) {
        self.inner.update_rect(item, rect);
    }

    pub fn remove(&mut self, item: T) -> bool {
        self.inner.remove(item)
    }

    pub fn query_radius(&self, pos: Point, radius: f32, out: &mut Vec<T>) {
        self.inner.query_radius(pos, radius, out);
    }

    pub fn query_rect(&self, rect: Rect, out: &mut Vec<T>) {
        self.inner.query_rect(rect, out);
    }
}

impl<T: Copy + Eq + Hash + Ord> DefaultIndexWithBackrefs<T> {
    pub fn query_radius_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<T>,
    ) -> &'a [T] {
        query_sorted_dedup(out, |scratch| self.query_radius(pos, radius, scratch))
    }

    pub fn query_rect_sorted_dedup<'a>(&self, rect: Rect, out: &'a mut Vec<T>) -> &'a [T] {
        query_sorted_dedup(out, |scratch| self.query_rect(rect, scratch))
    }
}

impl<T: Copy + Eq + Hash> Default for DefaultIndexWithBackrefs<T> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl<T: Copy + Eq + Hash> Default for GridIndexWithBackrefs<T> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Px, Rect, Size};

    use super::*;

    #[test]
    fn query_sorted_dedup_sorts_and_dedups_in_place() {
        let mut scratch = vec![999u32];
        let out = query_sorted_dedup(&mut scratch, |v| v.extend([3, 1, 2, 2, 1]));
        assert_eq!(out, [1, 2, 3]);
        assert_eq!(scratch, [1, 2, 3]);

        let out = query_sorted_dedup(&mut scratch, |v| v.extend([5, 4, 4]));
        assert_eq!(out, [4, 5]);
        assert_eq!(scratch, [4, 5]);
    }

    #[test]
    fn default_index_with_backrefs_sorted_dedup_is_deterministic() {
        let mut idx = DefaultIndexWithBackrefs::new(10.0);
        idx.insert_rect(
            2u32,
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(5.0), Px(5.0))),
        );
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(5.0), Px(5.0))),
        );
        idx.insert_rect(
            3u32,
            Rect::new(
                Point::new(Px(100.0), Px(100.0)),
                Size::new(Px(5.0), Px(5.0)),
            ),
        );

        let mut out = Vec::new();
        let got = idx.query_radius_sorted_dedup(Point::new(Px(2.0), Px(2.0)), 0.25, &mut out);
        assert_eq!(got, [1, 2]);

        let got = idx.query_rect_sorted_dedup(
            Rect::new(
                Point::new(Px(-1.0), Px(-1.0)),
                Size::new(Px(20.0), Px(20.0)),
            ),
            &mut out,
        );
        assert_eq!(got, [1, 2]);
    }

    #[cfg(not(feature = "rstar"))]
    #[test]
    fn default_index_with_backrefs_sorted_dedup_removes_duplicates_from_grid_backend() {
        let mut idx = DefaultIndexWithBackrefs::new(10.0);
        // This item spans multiple cells; a multi-cell query returns duplicates in the grid backend.
        idx.insert_rect(
            7u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(25.0), Px(25.0))),
        );

        let mut raw = Vec::new();
        idx.query_rect(
            Rect::new(
                Point::new(Px(-10.0), Px(-10.0)),
                Size::new(Px(60.0), Px(60.0)),
            ),
            &mut raw,
        );
        assert!(
            raw.len() > 1,
            "expected duplicates from multi-cell grid query"
        );
        assert!(raw.iter().all(|v| *v == 7));

        let mut out = Vec::new();
        let got = idx.query_rect_sorted_dedup(
            Rect::new(
                Point::new(Px(-10.0), Px(-10.0)),
                Size::new(Px(60.0), Px(60.0)),
            ),
            &mut out,
        );
        assert_eq!(got, [7]);
    }

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

    #[test]
    fn grid_index_with_backrefs_supports_move_update() {
        let mut idx = GridIndexWithBackrefs::new(10.0);
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(5.0), Px(5.0))),
        );

        let mut out = Vec::new();
        idx.query_radius(Point::new(Px(2.0), Px(2.0)), 1.0, &mut out);
        assert_eq!(out, vec![1]);

        idx.update_rect(
            1u32,
            Rect::new(
                Point::new(Px(100.0), Px(100.0)),
                Size::new(Px(5.0), Px(5.0)),
            ),
        );
        idx.query_radius(Point::new(Px(2.0), Px(2.0)), 1.0, &mut out);
        assert!(!out.contains(&1));

        idx.query_radius(Point::new(Px(102.0), Px(102.0)), 1.0, &mut out);
        assert!(out.contains(&1));
    }

    #[test]
    fn grid_index_with_backrefs_remove_clears_queries() {
        let mut idx = GridIndexWithBackrefs::new(10.0);
        idx.insert_aabb(1u32, 0.0, 0.0, 5.0, 5.0);
        assert!(idx.remove(1));
        assert!(!idx.remove(1));
        let mut out = Vec::new();
        idx.query_rect(
            Rect::new(
                Point::new(Px(-10.0), Px(-10.0)),
                Size::new(Px(100.0), Px(100.0)),
            ),
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn grid_index_with_backrefs_remove_does_not_corrupt_other_items() {
        let mut idx = GridIndexWithBackrefs::new(10.0);
        // Put both items into the same cell so removal triggers swap_remove behavior.
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(5.0), Px(5.0))),
        );
        idx.insert_rect(
            2u32,
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(5.0), Px(5.0))),
        );

        assert!(idx.remove(1));

        let mut out = Vec::new();
        idx.query_radius(Point::new(Px(2.0), Px(2.0)), 1.0, &mut out);
        assert!(out.contains(&2));
        assert!(!out.contains(&1));

        // Move the remaining item to ensure its backrefs were updated correctly.
        idx.update_rect(
            2u32,
            Rect::new(
                Point::new(Px(100.0), Px(100.0)),
                Size::new(Px(5.0), Px(5.0)),
            ),
        );
        idx.query_radius(Point::new(Px(2.0), Px(2.0)), 1.0, &mut out);
        assert!(!out.contains(&2));
        idx.query_radius(Point::new(Px(102.0), Px(102.0)), 1.0, &mut out);
        assert!(out.contains(&2));
    }

    #[test]
    fn grid_index_with_backrefs_update_is_noop_when_cells_do_not_change() {
        let mut idx = GridIndexWithBackrefs::new(10.0);
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(5.0), Px(5.0))),
        );

        // Still in the same cell (0,0) => update should not disturb removal bookkeeping.
        idx.update_rect(
            1u32,
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(5.0), Px(5.0))),
        );

        assert!(idx.remove(1));
        assert!(!idx.remove(1));
    }
}
