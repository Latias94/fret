use std::collections::HashMap;

use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathStyle, UiServices};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PathCacheKey {
    key: u64,
    scale_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PathFingerprint {
    commands_hash: u64,
    commands_len: usize,
    style: PathStyle,
    scale_bits: u32,
}

#[derive(Debug, Default)]
struct PathCacheEntry {
    path: Option<PathId>,
    metrics: Option<PathMetrics>,
    fingerprint: Option<PathFingerprint>,
    last_used_frame: u64,
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        1.0
    } else {
        scale_factor
    }
}

fn mix_u64(state: u64, v: u64) -> u64 {
    state
        ^ v.wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2)
}

fn hash_path_commands(commands: &[PathCommand]) -> u64 {
    let mut state = 0u64;
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p) => {
                state = mix_u64(state, 1);
                state = mix_u64(state, p.x.0.to_bits() as u64);
                state = mix_u64(state, p.y.0.to_bits() as u64);
            }
            PathCommand::LineTo(p) => {
                state = mix_u64(state, 2);
                state = mix_u64(state, p.x.0.to_bits() as u64);
                state = mix_u64(state, p.y.0.to_bits() as u64);
            }
            PathCommand::QuadTo { ctrl, to } => {
                state = mix_u64(state, 3);
                state = mix_u64(state, ctrl.x.0.to_bits() as u64);
                state = mix_u64(state, ctrl.y.0.to_bits() as u64);
                state = mix_u64(state, to.x.0.to_bits() as u64);
                state = mix_u64(state, to.y.0.to_bits() as u64);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                state = mix_u64(state, 4);
                state = mix_u64(state, ctrl1.x.0.to_bits() as u64);
                state = mix_u64(state, ctrl1.y.0.to_bits() as u64);
                state = mix_u64(state, ctrl2.x.0.to_bits() as u64);
                state = mix_u64(state, ctrl2.y.0.to_bits() as u64);
                state = mix_u64(state, to.x.0.to_bits() as u64);
                state = mix_u64(state, to.y.0.to_bits() as u64);
            }
            PathCommand::Close => {
                state = mix_u64(state, 5);
            }
        }
    }
    state
}

/// A small keyed cache for prepared paths.
///
/// The cache owns the `PathId`s and must be cleared (or dropped) with access to `UiServices` so
/// resources can be released deterministically.
#[derive(Debug, Default)]
pub struct PathCache {
    frame: u64,
    entries: HashMap<PathCacheKey, PathCacheEntry>,
}

impl PathCache {
    /// Increments and returns the internal frame counter used for pruning.
    pub fn begin_frame(&mut self) -> u64 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    /// Releases all cached paths.
    pub fn clear(&mut self, services: &mut dyn UiServices) {
        for entry in self.entries.values_mut() {
            if let Some(path) = entry.path.take() {
                services.path().release(path);
            }
        }
        self.entries.clear();
    }

    /// Returns a cached path for `(key, constraints.scale_factor)` if present.
    ///
    /// This updates the entry's `last_used_frame` for pruning purposes.
    pub fn get(&mut self, key: u64, constraints: PathConstraints) -> Option<(PathId, PathMetrics)> {
        let scale_factor = normalize_scale_factor(constraints.scale_factor);
        let scale_bits = scale_factor.to_bits();
        let cache_key = PathCacheKey { key, scale_bits };
        let entry = self.entries.get_mut(&cache_key)?;
        entry.last_used_frame = self.frame;
        Some((entry.path?, entry.metrics.unwrap_or_default()))
    }

    /// Prepares a path and caches it by a stable key derived from `(key, constraints.scale_factor)`.
    ///
    /// If the commands or style change for the same stable key, the cached `PathId` is replaced
    /// and the previous one is released immediately.
    pub fn prepare(
        &mut self,
        services: &mut dyn UiServices,
        key: u64,
        commands: &[PathCommand],
        style: PathStyle,
        constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        let scale_factor = normalize_scale_factor(constraints.scale_factor);
        let scale_bits = scale_factor.to_bits();
        let cache_key = PathCacheKey { key, scale_bits };

        let entry = self.entries.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;

        let fingerprint = PathFingerprint {
            commands_hash: hash_path_commands(commands),
            commands_len: commands.len(),
            style,
            scale_bits,
        };

        let needs_prepare =
            entry.path.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            if let Some(path) = entry.path.take() {
                services.path().release(path);
            }
            let (path, metrics) =
                services
                    .path()
                    .prepare(commands, style, PathConstraints { scale_factor });
            entry.path = Some(path);
            entry.metrics = Some(metrics);
            entry.fingerprint = Some(fingerprint);
        }

        (
            entry.path.unwrap_or_default(),
            entry.metrics.unwrap_or_default(),
        )
    }

    /// Drops old cache entries and releases their paths.
    pub fn prune(
        &mut self,
        services: &mut dyn UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        let now = self.frame;

        self.entries.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
            keep
        });

        if max_entries == 0 {
            self.clear(services);
            return;
        }

        if self.entries.len() <= max_entries {
            return;
        }

        let mut candidates: Vec<(u64, PathCacheKey)> = self
            .entries
            .iter()
            .map(|(k, v)| (v.last_used_frame, *k))
            .collect();
        candidates.sort_by_key(|(last_used, _)| *last_used);

        let over = self.entries.len().saturating_sub(max_entries);
        for (_, key) in candidates.into_iter().take(over) {
            if let Some(mut entry) = self.entries.remove(&key) {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        FillStyle, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextService,
    };

    #[derive(Default)]
    struct FakeServices {
        path_prepare_calls: u64,
        path_release_calls: u64,
    }

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::default(),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            self.path_prepare_calls += 1;
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {
            self.path_release_calls += 1;
        }
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn prepare_hits_cache_for_same_key_and_fingerprint() {
        let mut cache = PathCache::default();
        let mut services = FakeServices::default();
        cache.begin_frame();

        let cmds = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::Close,
        ];
        let constraints = PathConstraints { scale_factor: 1.0 };
        let (a, _) = cache.prepare(
            &mut services,
            1,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        let (b, _) = cache.prepare(
            &mut services,
            1,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );

        let _ = (a, b);
        assert_eq!(services.path_prepare_calls, 1);
        assert_eq!(services.path_release_calls, 0);
    }

    #[test]
    fn prepare_replaces_when_fingerprint_changes() {
        let mut cache = PathCache::default();
        let mut services = FakeServices::default();
        cache.begin_frame();

        let cmds_a = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::Close,
        ];
        let cmds_b = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(11.0), Px(0.0))),
            PathCommand::Close,
        ];
        let constraints = PathConstraints { scale_factor: 1.0 };

        let (a, _) = cache.prepare(
            &mut services,
            1,
            &cmds_a,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        let (b, _) = cache.prepare(
            &mut services,
            1,
            &cmds_b,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );

        let _ = (a, b);
        assert_eq!(services.path_prepare_calls, 2);
        assert_eq!(services.path_release_calls, 1);
    }

    #[test]
    fn prune_evicts_by_age_and_budget() {
        let mut cache = PathCache::default();
        let mut services = FakeServices::default();

        let cmds = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::Close,
        ];
        let constraints = PathConstraints { scale_factor: 1.0 };

        cache.begin_frame();
        cache.prepare(
            &mut services,
            1,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        cache.begin_frame();
        cache.prepare(
            &mut services,
            2,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        cache.begin_frame();

        // Drop key=1 by age (last used two frames ago).
        cache.prune(&mut services, 1, 99);
        assert_eq!(services.path_release_calls, 1);

        // Add one more and enforce budget.
        cache.prepare(
            &mut services,
            3,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        cache.prune(&mut services, 99, 1);
        assert!(services.path_release_calls >= 2);
    }

    #[test]
    fn get_hits_without_preparing() {
        let mut cache = PathCache::default();
        let mut services = FakeServices::default();

        let cmds = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::Close,
        ];
        let constraints = PathConstraints { scale_factor: 1.0 };

        cache.begin_frame();
        let _ = cache.prepare(
            &mut services,
            1,
            &cmds,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        assert_eq!(services.path_prepare_calls, 1);

        cache.begin_frame();
        let hit = cache.get(1, constraints);
        assert!(hit.is_some());
        assert_eq!(services.path_prepare_calls, 1);
    }
}
