use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{SceneOp, SvgId, UiServices};

use super::CacheStats;

/// Bytes for registering an SVG in retained caches.
///
/// Callers should prefer `Static` or `Bytes(Arc<[u8]>)` so the underlying pointer is stable.
#[derive(Clone)]
pub enum SvgBytes {
    Static(&'static [u8]),
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SvgCacheKey {
    key: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SvgFingerprint {
    Static { ptr: usize, len: usize },
    Bytes { ptr: usize, len: usize },
}

impl SvgBytes {
    fn fingerprint(&self) -> SvgFingerprint {
        match self {
            SvgBytes::Static(bytes) => SvgFingerprint::Static {
                ptr: bytes.as_ptr() as usize,
                len: bytes.len(),
            },
            SvgBytes::Bytes(bytes) => SvgFingerprint::Bytes {
                ptr: bytes.as_ptr() as usize,
                len: bytes.len(),
            },
        }
    }

    fn bytes(&self) -> &[u8] {
        match self {
            SvgBytes::Static(bytes) => bytes,
            SvgBytes::Bytes(bytes) => bytes,
        }
    }
}

#[derive(Debug, Default)]
struct SvgCacheEntry {
    svg: Option<SvgId>,
    bytes_len: u64,
    fingerprint: Option<SvgFingerprint>,
    last_used_frame: u64,
}

/// A small keyed cache for registered SVG IDs.
///
/// The cache owns the `SvgId`s and must be cleared (or dropped) with access to `UiServices`
/// so resources can be released deterministically.
#[derive(Debug, Default)]
pub struct SvgCache {
    frame: u64,
    bytes_ready: u64,
    entries: HashMap<SvgCacheKey, SvgCacheEntry>,
    id_to_key: HashMap<SvgId, u64>,
    stats: CacheStats,
}

impl SvgCache {
    /// Increments and returns the internal frame counter used for pruning.
    pub fn begin_frame(&mut self) -> u64 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn bytes_ready(&self) -> u64 {
        self.bytes_ready
    }

    /// Returns a cached SVG ID for `key` if present, updating `last_used_frame`.
    pub fn get(&mut self, key: u64) -> Option<SvgId> {
        self.stats.get_calls = self.stats.get_calls.saturating_add(1);
        let cache_key = SvgCacheKey { key };
        let entry = match self.entries.get_mut(&cache_key) {
            Some(entry) => entry,
            None => {
                self.stats.get_misses = self.stats.get_misses.saturating_add(1);
                return None;
            }
        };
        entry.last_used_frame = self.frame;
        let svg = match entry.svg {
            Some(svg) => svg,
            None => {
                self.stats.get_misses = self.stats.get_misses.saturating_add(1);
                return None;
            }
        };
        self.stats.get_hits = self.stats.get_hits.saturating_add(1);
        Some(svg)
    }

    /// Returns a cached SVG ID for `key` if present, without updating `last_used_frame`.
    pub fn peek(&self, key: u64) -> Option<SvgId> {
        self.entries.get(&SvgCacheKey { key })?.svg
    }

    /// Releases all cached SVG IDs.
    pub fn clear(&mut self, services: &mut dyn UiServices) {
        self.stats.clear_calls = self.stats.clear_calls.saturating_add(1);
        for entry in self.entries.values_mut() {
            if let Some(svg) = entry.svg.take() {
                let _ = services.svg().unregister_svg(svg);
                self.stats.release_clear = self.stats.release_clear.saturating_add(1);
                self.id_to_key.remove(&svg);
            }
        }
        self.entries.clear();
        self.id_to_key.clear();
        self.bytes_ready = 0;
    }

    /// Touch an existing cached SVG ID so it is not pruned.
    pub fn touch_svg(&mut self, svg: SvgId) -> bool {
        let Some(key) = self.id_to_key.get(&svg).copied() else {
            return false;
        };
        let Some(entry) = self.entries.get_mut(&SvgCacheKey { key }) else {
            self.id_to_key.remove(&svg);
            return false;
        };
        entry.last_used_frame = self.frame;
        true
    }

    /// Touch any SVG IDs referenced by `SceneOp::SvgMaskIcon` / `SceneOp::SvgImage` so they are not pruned.
    pub fn touch_svgs_in_scene_ops(&mut self, ops: &[SceneOp]) -> u32 {
        let mut touched: u32 = 0;
        for op in ops {
            let svg = match *op {
                SceneOp::SvgMaskIcon { svg, .. } => svg,
                SceneOp::SvgImage { svg, .. } => svg,
                _ => continue,
            };
            if self.touch_svg(svg) {
                touched = touched.saturating_add(1);
            }
        }
        touched
    }

    pub fn evict(&mut self, services: &mut dyn UiServices, key: u64) -> bool {
        self.stats.evict_calls = self.stats.evict_calls.saturating_add(1);
        let Some(mut entry) = self.entries.remove(&SvgCacheKey { key }) else {
            return false;
        };
        self.bytes_ready = self.bytes_ready.saturating_sub(entry.bytes_len);
        if let Some(svg) = entry.svg.take() {
            let _ = services.svg().unregister_svg(svg);
            self.stats.release_evict = self.stats.release_evict.saturating_add(1);
            self.id_to_key.remove(&svg);
        }
        true
    }

    /// Registers `bytes` as an SVG and caches the resulting `SvgId` by `key`.
    ///
    /// If `bytes` change for the same `key`, the cached `SvgId` is replaced and the previous one
    /// is unregistered immediately.
    pub fn prepare(&mut self, services: &mut dyn UiServices, key: u64, bytes: SvgBytes) -> SvgId {
        self.stats.prepare_calls = self.stats.prepare_calls.saturating_add(1);
        let cache_key = SvgCacheKey { key };
        let entry = self.entries.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;

        let fingerprint = bytes.fingerprint();
        let needs_prepare = entry.svg.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            let bytes_len = bytes.bytes().len() as u64;
            let svg_id = services.svg().register_svg(bytes.bytes());
            if let Some(old) = entry.svg.replace(svg_id) {
                let _ = services.svg().unregister_svg(old);
                self.stats.release_replaced = self.stats.release_replaced.saturating_add(1);
                self.id_to_key.remove(&old);
            }
            self.id_to_key.insert(svg_id, key);
            self.bytes_ready = self
                .bytes_ready
                .saturating_sub(entry.bytes_len)
                .saturating_add(bytes_len);
            entry.bytes_len = bytes_len;
            entry.fingerprint = Some(fingerprint);
            self.stats.prepare_misses = self.stats.prepare_misses.saturating_add(1);
        } else {
            self.stats.prepare_hits = self.stats.prepare_hits.saturating_add(1);
        }

        entry.svg.unwrap_or_default()
    }

    /// Drops old cache entries and unregisters their SVG IDs.
    pub fn prune(
        &mut self,
        services: &mut dyn UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        self.prune_with_budget(services, max_age_frames, max_entries, u64::MAX);
    }

    pub fn prune_with_budget(
        &mut self,
        services: &mut dyn UiServices,
        max_age_frames: u64,
        max_entries: usize,
        max_bytes: u64,
    ) {
        self.stats.prune_calls = self.stats.prune_calls.saturating_add(1);
        let now = self.frame;

        let mut removed_ids: Vec<SvgId> = Vec::new();
        self.entries.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                    self.stats.release_prune_age = self.stats.release_prune_age.saturating_add(1);
                    removed_ids.push(svg);
                }
                self.bytes_ready = self.bytes_ready.saturating_sub(entry.bytes_len);
            }
            keep
        });
        for id in removed_ids {
            self.id_to_key.remove(&id);
        }

        if max_entries == 0 || max_bytes == 0 {
            self.clear(services);
            return;
        }

        if self.entries.len() <= max_entries && self.bytes_ready <= max_bytes {
            return;
        }

        let mut candidates: Vec<(u64, u64)> = self
            .entries
            .iter()
            .map(|(k, v)| (v.last_used_frame, k.key))
            .collect();
        candidates.sort_by_key(|(last_used, _)| *last_used);

        for (_, key) in candidates {
            if self.entries.len() <= max_entries && self.bytes_ready <= max_bytes {
                break;
            }
            if let Some(mut entry) = self.entries.remove(&SvgCacheKey { key }) {
                self.bytes_ready = self.bytes_ready.saturating_sub(entry.bytes_len);
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                    self.stats.release_prune_budget =
                        self.stats.release_prune_budget.saturating_add(1);
                    self.id_to_key.remove(&svg);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Px, Size,
        TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };

    #[derive(Default)]
    struct FakeServices {
        svg_register_calls: u64,
        svg_unregister_calls: u64,
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
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            self.svg_register_calls += 1;
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            self.svg_unregister_calls += 1;
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn prepare_hits_cache_for_same_key_and_fingerprint() {
        let mut cache = SvgCache::default();
        let mut services = FakeServices::default();
        cache.begin_frame();

        let bytes: Arc<[u8]> = Arc::from(&b"<svg/>"[..]);
        let a = cache.prepare(&mut services, 1, SvgBytes::Bytes(bytes.clone()));
        let b = cache.prepare(&mut services, 1, SvgBytes::Bytes(bytes));

        let _ = (a, b);
        assert_eq!(services.svg_register_calls, 1);
        assert_eq!(services.svg_unregister_calls, 0);
        assert_eq!(cache.stats().prepare_hits, 1);
        assert_eq!(cache.stats().prepare_misses, 1);
    }

    #[test]
    fn prepare_replaces_when_bytes_change_for_same_key() {
        let mut cache = SvgCache::default();
        let mut services = FakeServices::default();
        cache.begin_frame();

        let a = cache.prepare(&mut services, 1, SvgBytes::Static(b"<svg/>"));
        let b = cache.prepare(&mut services, 1, SvgBytes::Static(b"<svg2/>"));

        let _ = (a, b);
        assert_eq!(services.svg_register_calls, 2);
        assert_eq!(services.svg_unregister_calls, 1);
    }

    #[test]
    fn prune_evicts_by_age_and_budget() {
        let mut cache = SvgCache::default();
        let mut services = FakeServices::default();

        cache.begin_frame();
        cache.prepare(&mut services, 1, SvgBytes::Static(b"<svg/>"));
        cache.begin_frame();
        cache.prepare(&mut services, 2, SvgBytes::Static(b"<svg/>"));
        cache.begin_frame();

        cache.prune(&mut services, 1, 99);
        assert_eq!(services.svg_unregister_calls, 1);

        cache.prepare(&mut services, 3, SvgBytes::Static(b"<svg/>"));
        cache.prune(&mut services, 99, 1);
        assert!(services.svg_unregister_calls >= 2);
    }

    #[test]
    fn touch_svg_prevents_prune_age_release() {
        let mut cache = SvgCache::default();
        let mut services = FakeServices::default();

        cache.begin_frame(); // frame 1
        let id = cache.prepare(&mut services, 1, SvgBytes::Static(b"<svg/>"));
        assert_eq!(services.svg_register_calls, 1);

        cache.begin_frame(); // frame 2
        assert!(cache.touch_svg(id));
        cache.prune(&mut services, 0, 10);
        assert_eq!(services.svg_unregister_calls, 0);
    }

    #[test]
    fn touch_svgs_in_scene_ops_prevents_prune_age_release() {
        let mut cache = SvgCache::default();
        let mut services = FakeServices::default();

        cache.begin_frame(); // frame 1
        let id = cache.prepare(&mut services, 1, SvgBytes::Static(b"<svg/>"));

        let ops = [fret_core::SceneOp::SvgImage {
            order: fret_core::DrawOrder(0),
            rect: fret_core::Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(1.0), Px(1.0)),
            ),
            svg: id,
            fit: fret_core::SvgFit::Contain,
            opacity: 1.0,
        }];

        cache.begin_frame(); // frame 2
        let touched = cache.touch_svgs_in_scene_ops(&ops);
        assert_eq!(touched, 1);
        cache.prune(&mut services, 0, 10);
        assert_eq!(services.svg_unregister_calls, 0);
    }
}
