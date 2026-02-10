use fret_core::{Point, Px, Rect, Scene, SceneOp};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use crate::budget::WorkBudget;

/// 2D tile coordinate for canvas-space tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    pub fn origin(self, tile_size_canvas: f32) -> Point {
        let s = tile_size_canvas;
        Point::new(Px(self.x as f32 * s), Px(self.y as f32 * s))
    }
}

/// Helper for mapping canvas-space rects into tile ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileGrid2D {
    tile_size_canvas: f32,
}

/// Helper for building deterministic tile cache keys without accidentally mixing in translation.
///
/// Callers should only hash *content-stable* parameters (model revision, zoom bits, style knobs,
/// mip level, tile size, etc.) and then combine with `TileCoord` via `tile_cache_key`.
#[derive(Debug, Default)]
pub struct TileCacheKeyBuilder {
    hasher: DefaultHasher,
}

impl TileCacheKeyBuilder {
    pub fn new(tag: &'static str) -> Self {
        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        Self { hasher }
    }

    pub fn add_u64(&mut self, v: u64) -> &mut Self {
        v.hash(&mut self.hasher);
        self
    }

    pub fn add_u32(&mut self, v: u32) -> &mut Self {
        v.hash(&mut self.hasher);
        self
    }

    pub fn add_i64(&mut self, v: i64) -> &mut Self {
        v.hash(&mut self.hasher);
        self
    }

    pub fn add_i32(&mut self, v: i32) -> &mut Self {
        v.hash(&mut self.hasher);
        self
    }

    pub fn add_f32_bits(&mut self, v: f32) -> &mut Self {
        v.to_bits().hash(&mut self.hasher);
        self
    }

    pub fn add_f64_bits(&mut self, v: f64) -> &mut Self {
        v.to_bits().hash(&mut self.hasher);
        self
    }

    pub fn add_str(&mut self, v: &'static str) -> &mut Self {
        v.hash(&mut self.hasher);
        self
    }

    pub fn finish(self) -> u64 {
        self.hasher.finish()
    }
}

pub fn tile_cache_key(base_key: u64, tile: TileCoord) -> u64 {
    let mut hasher = DefaultHasher::new();
    base_key.hash(&mut hasher);
    tile.hash(&mut hasher);
    hasher.finish()
}

impl TileGrid2D {
    pub fn new(tile_size_canvas: f32) -> Self {
        Self { tile_size_canvas }
    }

    pub fn tile_size_canvas(&self) -> f32 {
        self.tile_size_canvas
    }

    pub fn tiles_in_rect(&self, rect: Rect, out: &mut Vec<TileCoord>) {
        out.clear();

        let s = self.tile_size_canvas;
        if !s.is_finite() || s <= 0.0 {
            return;
        }

        let min_x = rect.origin.x.0;
        let min_y = rect.origin.y.0;
        let max_x = min_x + rect.size.width.0;
        let max_y = min_y + rect.size.height.0;

        let x0 = (min_x / s).floor() as i32;
        let y0 = (min_y / s).floor() as i32;
        let x1 = (max_x / s).floor() as i32;
        let y1 = (max_y / s).floor() as i32;

        for y in y0..=y1 {
            for x in x0..=x1 {
                out.push(TileCoord { x, y });
            }
        }
    }

    /// Sort tiles so those nearest the rect center come first.
    ///
    /// This is useful for incremental warmup under a per-frame budget: partial work tends to
    /// prioritize the visible center area and degrades more gracefully.
    pub fn sort_tiles_center_first(&self, rect: Rect, tiles: &mut [TileCoord]) {
        if tiles.len() <= 1 {
            return;
        }

        let s = self.tile_size_canvas;
        if !s.is_finite() || s <= 0.0 {
            return;
        }

        let center_x = rect.origin.x.0 + 0.5 * rect.size.width.0;
        let center_y = rect.origin.y.0 + 0.5 * rect.size.height.0;
        if !center_x.is_finite() || !center_y.is_finite() {
            return;
        }

        let center_tile = TileCoord {
            x: (center_x / s).floor() as i32,
            y: (center_y / s).floor() as i32,
        };

        tiles.sort_unstable_by_key(|t| {
            let dx = (i64::from(t.x) - i64::from(center_tile.x)).unsigned_abs();
            let dy = (i64::from(t.y) - i64::from(center_tile.y)).unsigned_abs();
            dx.saturating_add(dy).min(u64::from(u32::MAX)) as u32
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneOpTileWarmupOutcome {
    pub requested_tiles: usize,
    pub built_tiles: u32,
    pub skipped_tiles: u32,
}

/// Warm a tile cache incrementally under a per-frame budget.
///
/// This helper is policy-light:
/// - Callers choose the tile list ordering (e.g. center-first).
/// - Callers define how replay deltas and tile ops are computed.
/// - One budget "unit" typically corresponds to building one tile (configurable via `units_per_tile`).
pub fn warm_scene_op_tiles_u64<ReplayDeltaForTile, OpsForTile>(
    cache: &mut SceneOpTileCache<u64>,
    scene: &mut Scene,
    tiles: &[TileCoord],
    base_key: u64,
    units_per_tile: u32,
    budget: &mut WorkBudget,
    mut replay_delta_for_tile: ReplayDeltaForTile,
    mut ops_for_tile: OpsForTile,
) -> SceneOpTileWarmupOutcome
where
    ReplayDeltaForTile: FnMut(TileCoord) -> Point,
    OpsForTile: FnMut(TileCoord) -> Vec<SceneOp>,
{
    let mut built_tiles: u32 = 0;
    let mut skipped_tiles: u32 = 0;

    for tile in tiles.iter().copied() {
        let key = tile_cache_key(base_key, tile);
        let replay_delta = replay_delta_for_tile(tile);

        if cache.try_replay(key, scene, replay_delta) {
            continue;
        }

        if !budget.try_consume(units_per_tile) {
            skipped_tiles = skipped_tiles.saturating_add(1);
            continue;
        }

        let ops = ops_for_tile(tile);
        scene.replay_ops_translated(&ops, replay_delta);
        cache.store_ops(key, ops);
        built_tiles = built_tiles.saturating_add(1);
    }

    SceneOpTileWarmupOutcome {
        requested_tiles: tiles.len(),
        built_tiles,
        skipped_tiles,
    }
}

/// Warm a tile cache incrementally under a per-frame budget, with an on-hit hook.
///
/// This is useful for replay caches that must keep renderer-owned resources alive (e.g. by touching
/// `TextBlobId`/`PathId`/`SvgId` caches) before replaying ops.
pub fn warm_scene_op_tiles_u64_with<ReplayDeltaForTile, OnHit, OpsForTile>(
    cache: &mut SceneOpTileCache<u64>,
    scene: &mut Scene,
    tiles: &[TileCoord],
    base_key: u64,
    units_per_tile: u32,
    budget: &mut WorkBudget,
    mut replay_delta_for_tile: ReplayDeltaForTile,
    mut on_hit: OnHit,
    mut ops_for_tile: OpsForTile,
) -> SceneOpTileWarmupOutcome
where
    ReplayDeltaForTile: FnMut(TileCoord) -> Point,
    OnHit: FnMut(&[SceneOp]),
    OpsForTile: FnMut(TileCoord) -> Vec<SceneOp>,
{
    let mut built_tiles: u32 = 0;
    let mut skipped_tiles: u32 = 0;

    for tile in tiles.iter().copied() {
        let key = tile_cache_key(base_key, tile);
        let replay_delta = replay_delta_for_tile(tile);

        if cache.try_replay_with(key, scene, replay_delta, |ops| on_hit(ops)) {
            continue;
        }

        if !budget.try_consume(units_per_tile) {
            skipped_tiles = skipped_tiles.saturating_add(1);
            continue;
        }

        let ops = ops_for_tile(tile);
        scene.replay_ops_translated(&ops, replay_delta);
        cache.store_ops(key, ops);
        built_tiles = built_tiles.saturating_add(1);
    }

    SceneOpTileWarmupOutcome {
        requested_tiles: tiles.len(),
        built_tiles,
        skipped_tiles,
    }
}

/// Cache for recorded `SceneOp`s split into fixed-size tiles (retained-canvas replay caching).
///
/// This is intended for large canvases where:
/// - the scene is spatially local (only a fraction is visible),
/// - pan/scroll changes the visible set, but most tiles remain reusable,
/// - a single monolithic "static layer" cache would miss too often.
///
/// Like `SceneOpCache`, this cache does **not** manage renderer-owned resources referenced by ops.
/// Callers must ensure referenced resources remain valid for as long as cached ops are used.
#[derive(Debug, Default)]
pub struct SceneOpTileCache<K> {
    entries: HashMap<K, SceneOpTileEntry>,
    frame: u64,
    stats: SceneOpTileCacheStats,
}

#[derive(Debug, Default)]
struct SceneOpTileEntry {
    ops: Vec<SceneOp>,
    last_used_frame: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneOpTileCacheStats {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub stored_tiles: u64,
    pub recorded_ops: u64,
    pub replayed_ops: u64,
    pub clear_calls: u64,
    pub prune_calls: u64,
    pub evict_calls: u64,
    pub evict_prune_age: u64,
    pub evict_prune_budget: u64,
}

impl<K: Eq + Hash + Copy> SceneOpTileCache<K> {
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub fn stats(&self) -> SceneOpTileCacheStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = SceneOpTileCacheStats::default();
    }

    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_key(&self, key: K) -> bool {
        self.entries.contains_key(&key)
    }

    pub fn ops_for_key(&self, key: K) -> Option<&[SceneOp]> {
        self.entries.get(&key).map(|e| e.ops.as_slice())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.clear_calls = self.stats.clear_calls.saturating_add(1);
    }

    /// Replay cached ops for `key` when present.
    ///
    /// `replay_delta` is applied via `Scene::replay_ops_translated()`.
    /// Returns `true` when the cache was hit.
    pub fn try_replay(&mut self, key: K, scene: &mut Scene, replay_delta: Point) -> bool {
        self.stats.calls = self.stats.calls.saturating_add(1);

        let Some(entry) = self.entries.get_mut(&key) else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            return false;
        };

        entry.last_used_frame = self.frame;
        self.stats.hits = self.stats.hits.saturating_add(1);
        self.stats.replayed_ops = self
            .stats
            .replayed_ops
            .saturating_add(entry.ops.len().min(u64::MAX as usize) as u64);
        scene.replay_ops_translated(&entry.ops, replay_delta);
        true
    }

    /// Replay cached ops for `key` when present, with an additional on-hit hook.
    ///
    /// This is useful for replay caches that must keep renderer-owned resources alive (e.g. by
    /// touching `TextBlobId`/`PathId`/`SvgId` caches) before replaying ops.
    ///
    /// `replay_delta` is applied via `Scene::replay_ops_translated()`.
    /// Returns `true` when the cache was hit.
    pub fn try_replay_with(
        &mut self,
        key: K,
        scene: &mut Scene,
        replay_delta: Point,
        on_hit: impl FnOnce(&[SceneOp]),
    ) -> bool {
        self.stats.calls = self.stats.calls.saturating_add(1);

        let Some(entry) = self.entries.get_mut(&key) else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            return false;
        };

        entry.last_used_frame = self.frame;
        self.stats.hits = self.stats.hits.saturating_add(1);
        self.stats.replayed_ops = self
            .stats
            .replayed_ops
            .saturating_add(entry.ops.len().min(u64::MAX as usize) as u64);
        on_hit(&entry.ops);
        scene.replay_ops_translated(&entry.ops, replay_delta);
        true
    }

    /// Replace cached ops for `key`.
    pub fn store_ops(&mut self, key: K, ops: Vec<SceneOp>) {
        self.stats.stored_tiles = self.stats.stored_tiles.saturating_add(1);
        self.stats.recorded_ops = self
            .stats
            .recorded_ops
            .saturating_add(ops.len().min(u64::MAX as usize) as u64);

        let entry = self.entries.entry(key).or_default();
        entry.ops = ops;
        entry.last_used_frame = self.frame;
    }

    /// Evict unused entries by age and budget.
    ///
    /// - `max_age_frames`: entries unused for longer than this are removed.
    /// - `max_entries`: hard cap; extra entries are evicted by LRU (oldest `last_used_frame`).
    pub fn prune(&mut self, max_age_frames: u64, max_entries: usize) {
        self.stats.prune_calls = self.stats.prune_calls.saturating_add(1);

        if max_age_frames > 0 {
            let cutoff = self.frame.saturating_sub(max_age_frames);
            let before = self.entries.len();
            self.entries.retain(|_, v| v.last_used_frame >= cutoff);
            let evicted = before.saturating_sub(self.entries.len());
            if evicted > 0 {
                self.stats.evict_calls = self.stats.evict_calls.saturating_add(1);
                self.stats.evict_prune_age = self
                    .stats
                    .evict_prune_age
                    .saturating_add(evicted.min(u64::MAX as usize) as u64);
            }
        }

        if max_entries > 0 && self.entries.len() > max_entries {
            let mut items: Vec<(Reverse<u64>, K)> = Vec::with_capacity(self.entries.len());
            for (&k, v) in &self.entries {
                items.push((Reverse(v.last_used_frame), k));
            }
            items.sort_unstable_by_key(|(frame, _)| *frame);

            let to_evict = self.entries.len() - max_entries;
            for (_, k) in items.into_iter().take(to_evict) {
                if self.entries.remove(&k).is_some() {
                    self.stats.evict_calls = self.stats.evict_calls.saturating_add(1);
                    self.stats.evict_prune_budget = self.stats.evict_prune_budget.saturating_add(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Color, DrawOrder, Size};
    use std::cell::Cell;

    use super::*;

    #[test]
    fn tile_grid_rect_maps_to_expected_tiles() {
        let grid = TileGrid2D::new(10.0);
        let rect = Rect::new(
            Point::new(Px(-1.0), Px(-1.0)),
            Size::new(Px(12.0), Px(12.0)),
        );
        let mut tiles = Vec::new();
        grid.tiles_in_rect(rect, &mut tiles);
        assert!(tiles.contains(&TileCoord { x: -1, y: -1 }));
        assert!(tiles.contains(&TileCoord { x: 0, y: 0 }));
        assert!(tiles.contains(&TileCoord { x: 1, y: 1 }));
    }

    #[test]
    fn try_replay_hits_and_updates_last_used() {
        let mut cache: SceneOpTileCache<u64> = SceneOpTileCache::default();
        cache.begin_frame();
        cache.store_ops(
            1,
            vec![SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
                background: fret_core::Paint::Solid(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            }],
        );

        cache.begin_frame();
        let mut scene = Scene::default();
        assert!(cache.try_replay(1, &mut scene, Point::new(Px(0.0), Px(0.0))));
        assert_eq!(scene.ops_len(), 1);
    }

    #[test]
    fn prune_evicts_by_age_then_budget() {
        let mut cache: SceneOpTileCache<u64> = SceneOpTileCache::default();

        cache.begin_frame();
        cache.store_ops(1, Vec::new());

        cache.begin_frame();
        cache.store_ops(2, Vec::new());

        cache.begin_frame();
        cache.prune(1, 999);
        assert!(!cache.entries.contains_key(&1));
        assert!(cache.entries.contains_key(&2));

        cache.store_ops(3, Vec::new());
        cache.prune(999, 1);
        assert_eq!(cache.entries_len(), 1);
    }

    #[test]
    fn warm_scene_op_tiles_respects_budget_and_hits_cache() {
        let tiles = [TileCoord { x: 0, y: 0 }];
        let base_key = 123u64;

        let mut cache: SceneOpTileCache<u64> = SceneOpTileCache::default();
        cache.begin_frame();

        let mut scene = Scene::default();
        let mut budget = WorkBudget::new(0);

        let out = warm_scene_op_tiles_u64(
            &mut cache,
            &mut scene,
            &tiles,
            base_key,
            1,
            &mut budget,
            |_tile| Point::new(Px(0.0), Px(0.0)),
            |_tile| panic!("should not build ops when budget is exhausted"),
        );
        assert_eq!(out.requested_tiles, 1);
        assert_eq!(out.built_tiles, 0);
        assert_eq!(out.skipped_tiles, 1);

        let mut scene = Scene::default();
        let mut budget = WorkBudget::new(1);
        let out = warm_scene_op_tiles_u64(
            &mut cache,
            &mut scene,
            &tiles,
            base_key,
            1,
            &mut budget,
            |_tile| Point::new(Px(0.0), Px(0.0)),
            |_tile| {
                vec![SceneOp::Quad {
                    order: DrawOrder(0),
                    rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
                    background: fret_core::Paint::Solid(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                }]
            },
        );
        assert_eq!(out.built_tiles, 1);
        assert_eq!(out.skipped_tiles, 0);
        assert_eq!(cache.entries_len(), 1);
        assert_eq!(scene.ops_len(), 1);

        cache.begin_frame();
        let mut scene = Scene::default();
        let mut budget = WorkBudget::new(0);
        let out = warm_scene_op_tiles_u64(
            &mut cache,
            &mut scene,
            &tiles,
            base_key,
            1,
            &mut budget,
            |_tile| Point::new(Px(0.0), Px(0.0)),
            |_tile| panic!("should hit cache and never rebuild ops"),
        );
        assert_eq!(out.built_tiles, 0);
        assert_eq!(out.skipped_tiles, 0);
        assert_eq!(scene.ops_len(), 1);
    }

    #[test]
    fn tile_cache_try_replay_with_calls_hook_only_on_hit() {
        let mut cache: SceneOpTileCache<u64> = SceneOpTileCache::default();
        cache.begin_frame();
        cache.store_ops(1, vec![SceneOp::PopTransform]);

        cache.begin_frame();
        let calls = Cell::new(0usize);
        let mut scene = Scene::default();
        assert!(
            cache.try_replay_with(1, &mut scene, Point::new(Px(0.0), Px(0.0)), |_ops| {
                calls.set(calls.get() + 1);
            })
        );
        assert_eq!(calls.get(), 1);

        let mut scene = Scene::default();
        assert!(
            !cache.try_replay_with(2, &mut scene, Point::new(Px(0.0), Px(0.0)), |_ops| {
                calls.set(calls.get() + 1);
            })
        );
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn warm_scene_op_tiles_calls_on_hit_for_cached_tiles() {
        let tiles = [TileCoord { x: 0, y: 0 }];
        let base_key = 123u64;

        let mut cache: SceneOpTileCache<u64> = SceneOpTileCache::default();
        cache.begin_frame();

        let mut scene = Scene::default();
        let mut budget = WorkBudget::new(1);
        let _ = warm_scene_op_tiles_u64(
            &mut cache,
            &mut scene,
            &tiles,
            base_key,
            1,
            &mut budget,
            |_tile| Point::new(Px(0.0), Px(0.0)),
            |_tile| vec![SceneOp::PopTransform],
        );

        cache.begin_frame();
        let calls = Cell::new(0usize);
        let mut scene = Scene::default();
        let mut budget = WorkBudget::new(0);
        let _ = warm_scene_op_tiles_u64_with(
            &mut cache,
            &mut scene,
            &tiles,
            base_key,
            1,
            &mut budget,
            |_tile| Point::new(Px(0.0), Px(0.0)),
            |_ops| {
                calls.set(calls.get() + 1);
            },
            |_tile| panic!("should hit cache and never rebuild ops"),
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(scene.ops_len(), 1);
    }
}
