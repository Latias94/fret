//! R-tree spatial indexing helpers for large 2D canvas widgets.
//!
//! This backend is feature-gated (`fret-canvas/rstar`) so ecosystem crates can opt in after
//! evaluating performance characteristics on real workloads.

#![cfg(feature = "rstar")]

use std::collections::HashMap;
use std::hash::Hash;

use fret_core::{Point, Rect};
use rstar::{AABB, RTree, RTreeObject};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Entry<T> {
    item: T,
    envelope: AABB<[f32; 2]>,
}

impl<T: Copy> RTreeObject for Entry<T> {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

fn rect_to_aabb(rect: Rect) -> Option<AABB<[f32; 2]>> {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    if !x0.is_finite() || !y0.is_finite() || !x1.is_finite() || !y1.is_finite() {
        return None;
    }

    let min_x = x0.min(x1);
    let min_y = y0.min(y1);
    let max_x = x0.max(x1);
    let max_y = y0.max(y1);
    if min_x > max_x || min_y > max_y {
        return None;
    }

    Some(AABB::from_corners([min_x, min_y], [max_x, max_y]))
}

fn aabb_for_radius(pos: Point, radius: f32) -> Option<AABB<[f32; 2]>> {
    let r = if radius.is_finite() {
        radius.max(0.0)
    } else {
        0.0
    };
    let x = pos.x.0;
    let y = pos.y.0;
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    Some(AABB::from_corners([x - r, y - r], [x + r, y + r]))
}

/// R-tree index that supports incremental updates (remove/move) via back-references.
///
/// Tradeoffs vs the grid backend:
/// - Usually fewer candidate returns for sparse/non-uniform distributions.
/// - Updates are `remove + insert` (log N) and may be slower for heavy drag workloads with many
///   moving items each frame.
#[derive(Debug, Clone)]
pub struct RStarIndexWithBackrefs<T> {
    tree: RTree<Entry<T>>,
    item_envelopes: HashMap<T, AABB<[f32; 2]>>,
}

impl<T: Copy + Eq + Hash> RStarIndexWithBackrefs<T> {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            item_envelopes: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tree = RTree::new();
        self.item_envelopes.clear();
    }

    pub fn insert_rect(&mut self, item: T, rect: Rect) {
        let Some(envelope) = rect_to_aabb(rect) else {
            let _ = self.remove(item);
            return;
        };
        self.remove(item);
        self.tree.insert(Entry { item, envelope });
        self.item_envelopes.insert(item, envelope);
    }

    pub fn update_rect(&mut self, item: T, rect: Rect) {
        let Some(envelope) = rect_to_aabb(rect) else {
            let _ = self.remove(item);
            return;
        };
        if self
            .item_envelopes
            .get(&item)
            .is_some_and(|e| *e == envelope)
        {
            return;
        }
        self.remove(item);
        self.tree.insert(Entry { item, envelope });
        self.item_envelopes.insert(item, envelope);
    }

    pub fn remove(&mut self, item: T) -> bool {
        let Some(envelope) = self.item_envelopes.remove(&item) else {
            return false;
        };
        self.tree.remove(&Entry { item, envelope }).is_some()
    }

    pub fn query_radius(&self, pos: Point, radius: f32, out: &mut Vec<T>) {
        out.clear();
        let Some(env) = aabb_for_radius(pos, radius) else {
            return;
        };
        out.extend(
            self.tree
                .locate_in_envelope_intersecting(&env)
                .map(|e| e.item),
        );
    }

    pub fn query_rect(&self, rect: Rect, out: &mut Vec<T>) {
        out.clear();
        let Some(env) = rect_to_aabb(rect) else {
            return;
        };
        out.extend(
            self.tree
                .locate_in_envelope_intersecting(&env)
                .map(|e| e.item),
        );
    }
}

impl<T: Copy + Eq + Hash> Default for RStarIndexWithBackrefs<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Px, Rect, Size};

    use super::*;

    #[test]
    fn rstar_index_query_radius_returns_candidates() {
        let mut idx = RStarIndexWithBackrefs::new();
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
        );
        idx.insert_rect(
            2u32,
            Rect::new(
                Point::new(Px(100.0), Px(100.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
        );

        let mut out = Vec::new();
        idx.query_radius(Point::new(Px(5.0), Px(5.0)), 2.0, &mut out);
        assert!(out.contains(&1));
        assert!(!out.contains(&2));
    }

    #[test]
    fn rstar_index_update_and_remove_work() {
        let mut idx = RStarIndexWithBackrefs::new();
        idx.insert_rect(
            1u32,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
        );
        idx.update_rect(
            1u32,
            Rect::new(
                Point::new(Px(100.0), Px(100.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
        );

        let mut out = Vec::new();
        idx.query_radius(Point::new(Px(5.0), Px(5.0)), 5.0, &mut out);
        assert!(!out.contains(&1));
        idx.query_radius(Point::new(Px(105.0), Px(105.0)), 5.0, &mut out);
        assert!(out.contains(&1));

        assert!(idx.remove(1));
        assert!(!idx.remove(1));
        idx.query_radius(Point::new(Px(105.0), Px(105.0)), 5.0, &mut out);
        assert!(!out.contains(&1));
    }
}
