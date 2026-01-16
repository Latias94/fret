use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{SvgId, UiServices};

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
}

impl SvgCache {
    /// Increments and returns the internal frame counter used for pruning.
    pub fn begin_frame(&mut self) -> u64 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn bytes_ready(&self) -> u64 {
        self.bytes_ready
    }

    /// Returns a cached SVG ID for `key` if present, updating `last_used_frame`.
    pub fn get(&mut self, key: u64) -> Option<SvgId> {
        let cache_key = SvgCacheKey { key };
        let entry = self.entries.get_mut(&cache_key)?;
        entry.last_used_frame = self.frame;
        entry.svg
    }

    /// Returns a cached SVG ID for `key` if present, without updating `last_used_frame`.
    pub fn peek(&self, key: u64) -> Option<SvgId> {
        self.entries.get(&SvgCacheKey { key })?.svg
    }

    /// Releases all cached SVG IDs.
    pub fn clear(&mut self, services: &mut dyn UiServices) {
        for entry in self.entries.values_mut() {
            if let Some(svg) = entry.svg.take() {
                let _ = services.svg().unregister_svg(svg);
            }
        }
        self.entries.clear();
        self.bytes_ready = 0;
    }

    pub fn evict(&mut self, services: &mut dyn UiServices, key: u64) -> bool {
        let Some(mut entry) = self.entries.remove(&SvgCacheKey { key }) else {
            return false;
        };
        self.bytes_ready = self.bytes_ready.saturating_sub(entry.bytes_len);
        if let Some(svg) = entry.svg.take() {
            let _ = services.svg().unregister_svg(svg);
        }
        true
    }

    /// Registers `bytes` as an SVG and caches the resulting `SvgId` by `key`.
    ///
    /// If `bytes` change for the same `key`, the cached `SvgId` is replaced and the previous one
    /// is unregistered immediately.
    pub fn prepare(&mut self, services: &mut dyn UiServices, key: u64, bytes: SvgBytes) -> SvgId {
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
            }
            self.bytes_ready = self
                .bytes_ready
                .saturating_sub(entry.bytes_len)
                .saturating_add(bytes_len);
            entry.bytes_len = bytes_len;
            entry.fingerprint = Some(fingerprint);
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
        let now = self.frame;

        self.entries.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                }
                self.bytes_ready = self.bytes_ready.saturating_sub(entry.bytes_len);
            }
            keep
        });

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
            let _ = self.evict(services, key);
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
}
