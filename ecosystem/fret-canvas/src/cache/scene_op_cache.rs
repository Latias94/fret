use fret_core::{Point, Scene, SceneOp};

/// Lightweight cache for recorded `SceneOp`s (retained-canvas "static layer" caching).
///
/// This intentionally does **not** manage renderer-owned resources referenced by ops (e.g. text
/// blobs, paths, images). Callers should only cache ops that remain valid across frames, or make
/// sure referenced resources are kept alive independently.
#[derive(Debug, Default)]
pub struct SceneOpCache<K> {
    key: Option<K>,
    ops: Vec<SceneOp>,
    stats: SceneOpCacheStats,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneOpCacheStats {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub recorded_ops: u64,
    pub replayed_ops: u64,
    pub clear_calls: u64,
}

impl<K: Copy + PartialEq> SceneOpCache<K> {
    pub fn ops(&self) -> &[SceneOp] {
        &self.ops
    }

    pub fn matches(&self, key: K) -> bool {
        self.key == Some(key)
    }

    pub fn clear(&mut self) {
        self.key = None;
        self.ops.clear();
        self.stats.clear_calls = self.stats.clear_calls.saturating_add(1);
    }

    pub fn stats(&self) -> SceneOpCacheStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = SceneOpCacheStats::default();
    }

    pub fn ops_len(&self) -> usize {
        self.ops.len()
    }

    /// Replay cached ops when `key` matches.
    ///
    /// `replay_delta` is applied via `Scene::replay_ops_translated()`.
    /// Returns `true` when the cache was hit.
    pub fn try_replay(&mut self, key: K, scene: &mut Scene, replay_delta: Point) -> bool {
        self.stats.calls = self.stats.calls.saturating_add(1);

        if self.key == Some(key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            self.stats.replayed_ops = self
                .stats
                .replayed_ops
                .saturating_add(self.ops.len().min(u64::MAX as usize) as u64);
            scene.replay_ops_translated(&self.ops, replay_delta);
            true
        } else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            false
        }
    }

    /// Replay cached ops when `key` matches, with an additional on-hit hook.
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

        if self.key == Some(key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            self.stats.replayed_ops = self
                .stats
                .replayed_ops
                .saturating_add(self.ops.len().min(u64::MAX as usize) as u64);
            on_hit(&self.ops);
            scene.replay_ops_translated(&self.ops, replay_delta);
            true
        } else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            false
        }
    }

    /// Replace cached ops for `key`.
    pub fn store_ops(&mut self, key: K, ops: Vec<SceneOp>) {
        self.key = Some(key);
        self.stats.recorded_ops = self
            .stats
            .recorded_ops
            .saturating_add(ops.len().min(u64::MAX as usize) as u64);
        self.ops = ops;
    }

    /// Replay cached ops when `key` matches; otherwise run `record` and replace the cache.
    ///
    /// `replay_delta` is applied via `Scene::replay_ops_translated()`.
    /// Returns `true` when the cache was hit.
    pub fn replay_or_record(
        &mut self,
        key: K,
        scene: &mut Scene,
        replay_delta: Point,
        record: impl FnOnce(&mut Scene),
    ) -> bool {
        if self.try_replay(key, scene, replay_delta) {
            return true;
        }

        self.key = Some(key);
        self.ops.clear();

        let start = scene.ops_len();
        record(scene);
        let end = scene.ops_len();

        if end > start {
            self.ops.extend_from_slice(&scene.ops()[start..end]);
            self.stats.recorded_ops = self
                .stats
                .recorded_ops
                .saturating_add((end - start).min(u64::MAX as usize) as u64);
        }

        false
    }

    /// Replay cached ops when `key` matches; otherwise run `record` and replace the cache.
    ///
    /// Like `replay_or_record`, but calls `on_hit` with the cached ops before replaying.
    /// Returns `true` when the cache was hit.
    pub fn replay_or_record_with(
        &mut self,
        key: K,
        scene: &mut Scene,
        replay_delta: Point,
        on_hit: impl FnOnce(&[SceneOp]),
        record: impl FnOnce(&mut Scene),
    ) -> bool {
        if self.try_replay_with(key, scene, replay_delta, on_hit) {
            return true;
        }

        self.key = Some(key);
        self.ops.clear();

        let start = scene.ops_len();
        record(scene);
        let end = scene.ops_len();

        if end > start {
            self.ops.extend_from_slice(&scene.ops()[start..end]);
            self.stats.recorded_ops = self
                .stats
                .recorded_ops
                .saturating_add((end - start).min(u64::MAX as usize) as u64);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Color, DrawOrder, Px, Rect, Size};
    use std::cell::Cell;

    use super::*;

    #[test]
    fn scene_op_cache_replays_when_key_matches() {
        let mut cache: SceneOpCache<u64> = SceneOpCache::default();

        let mut scene = Scene::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

        let hit0 = cache.replay_or_record(1, &mut scene, Point::new(Px(0.0), Px(0.0)), |scene| {
            scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: bounds,
                background: fret_core::Paint::Solid(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        });
        assert!(!hit0);
        assert_eq!(cache.ops_len(), 1);

        let mut scene2 = Scene::default();
        let hit1 = cache.replay_or_record(1, &mut scene2, Point::new(Px(2.0), Px(3.0)), |_scene| {
            unreachable!("record should not run on hit")
        });
        assert!(hit1);
        // replay_ops_translated wraps ops in a transform stack when delta != 0.
        assert_eq!(scene2.ops_len(), 3);
    }

    #[test]
    fn scene_op_cache_store_ops_then_try_replay() {
        let mut cache: SceneOpCache<u64> = SceneOpCache::default();
        cache.store_ops(
            9,
            vec![SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
                background: fret_core::Paint::Solid(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            }],
        );

        let mut scene = Scene::default();
        assert!(cache.try_replay(9, &mut scene, Point::new(Px(0.0), Px(0.0))));
        assert_eq!(scene.ops_len(), 1);
    }

    #[test]
    fn scene_op_cache_try_replay_with_calls_hook_only_on_hit() {
        let mut cache: SceneOpCache<u64> = SceneOpCache::default();
        cache.store_ops(
            9,
            vec![SceneOp::Quad {
                order: DrawOrder(0),
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
                background: fret_core::Paint::Solid(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            }],
        );

        let calls = Cell::new(0usize);
        let mut scene = Scene::default();
        assert!(
            cache.try_replay_with(9, &mut scene, Point::new(Px(0.0), Px(0.0)), |ops| {
                assert_eq!(ops.len(), 1);
                calls.set(calls.get() + 1);
            })
        );
        assert_eq!(calls.get(), 1);

        let mut scene = Scene::default();
        assert!(
            !cache.try_replay_with(10, &mut scene, Point::new(Px(0.0), Px(0.0)), |_ops| {
                calls.set(calls.get() + 1);
            })
        );
        assert_eq!(calls.get(), 1);
    }
}
