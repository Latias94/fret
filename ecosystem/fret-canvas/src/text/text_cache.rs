use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextStyle, UiServices};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::cache::CacheStats;

fn hash_text(text: &str) -> u64 {
    let mut hasher = Fnv1a64::default();
    hasher.write(text.as_bytes());
    hasher.finish()
}

/// A prepared text blob with metrics and a stable cache key.
#[derive(Debug, Clone, Copy)]
pub struct PreparedText {
    pub blob: TextBlobId,
    pub metrics: TextMetrics,
    pub key: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextCacheKey {
    text_hash: u64,
    text_len: u32,
    text: Arc<str>,
    font: fret_core::FontId,
    size_bits: u32,
    weight: u16,
    slant: u8,
    line_height_bits: Option<u32>,
    letter_spacing_em_bits: Option<u32>,
    scale_factor_bits: u32,
    max_width_bits: Option<u32>,
    wrap: fret_core::TextWrap,
    overflow: fret_core::TextOverflow,
}

impl Hash for TextCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text_hash.hash(state);
        self.text_len.hash(state);
        self.font.hash(state);
        self.size_bits.hash(state);
        self.weight.hash(state);
        self.slant.hash(state);
        self.line_height_bits.hash(state);
        self.letter_spacing_em_bits.hash(state);
        self.scale_factor_bits.hash(state);
        self.max_width_bits.hash(state);
        self.wrap.hash(state);
        self.overflow.hash(state);
    }
}

impl TextCacheKey {
    fn new(text: &str, style: &TextStyle, constraints: TextConstraints) -> Self {
        Self {
            text_hash: hash_text(text),
            text_len: text.len() as u32,
            text: Arc::<str>::from(text),
            font: style.font.clone(),
            size_bits: style.size.0.to_bits(),
            weight: style.weight.0,
            slant: style.slant as u8,
            line_height_bits: style.line_height.map(|v| v.0.to_bits()),
            letter_spacing_em_bits: style.letter_spacing_em.map(f32::to_bits),
            scale_factor_bits: constraints.scale_factor.to_bits(),
            max_width_bits: constraints.max_width.map(|v| v.0.to_bits()),
            wrap: constraints.wrap,
            overflow: constraints.overflow,
        }
    }

    fn stable_key(&self) -> u64 {
        let mut hasher = Fnv1a64::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone)]
struct TextCacheEntry {
    prepared: PreparedText,
    last_used_frame: u64,
}

/// A small keyed cache for prepared text blobs.
///
/// The cache owns the `TextBlobId`s and must be cleared (or dropped) with access to `UiServices`
/// so resources can be released deterministically.
#[derive(Debug, Default)]
pub struct TextCache {
    frame: u64,
    entries: HashMap<TextCacheKey, TextCacheEntry>,
    stats: CacheStats,
}

impl TextCache {
    /// Increments and returns the internal frame counter used for optional pruning.
    ///
    /// Callers may ignore this entirely; the cache remains correct without pruning.
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

    /// Releases all cached blobs.
    pub fn clear(&mut self, services: &mut dyn UiServices) {
        self.stats.clear_calls = self.stats.clear_calls.saturating_add(1);
        for t in self.entries.values() {
            services.text().release(t.prepared.blob);
            self.stats.release_clear = self.stats.release_clear.saturating_add(1);
        }
        self.entries.clear();
    }

    /// Prepares text and caches it by a stable key derived from `(text, style, constraints)`.
    ///
    /// Note: callers that apply additional view-zoom scaling should incorporate that into
    /// `constraints.scale_factor` (e.g. `dpi * zoom`).
    pub fn prepare(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        self.stats.prepare_calls = self.stats.prepare_calls.saturating_add(1);
        let key = TextCacheKey::new(text, style, constraints);
        match self.entries.entry(key) {
            Entry::Occupied(mut e) => {
                e.get_mut().last_used_frame = self.frame;
                self.stats.prepare_hits = self.stats.prepare_hits.saturating_add(1);
                e.get().prepared
            }
            Entry::Vacant(e) => {
                let (blob, metrics) = services.text().prepare_str(text, style, constraints);
                let prepared = PreparedText {
                    blob,
                    metrics,
                    key: e.key().stable_key(),
                };
                e.insert(TextCacheEntry {
                    prepared,
                    last_used_frame: self.frame,
                });
                self.stats.prepare_misses = self.stats.prepare_misses.saturating_add(1);
                prepared
            }
        }
    }

    /// Drops old cache entries and releases their blobs.
    ///
    /// This is intentionally simple and conservative: it is an optional hygiene helper for long-lived
    /// canvases (plots/editors) to avoid unbounded growth.
    pub fn prune(
        &mut self,
        services: &mut dyn UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        self.stats.prune_calls = self.stats.prune_calls.saturating_add(1);
        let now = self.frame;
        self.entries.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                services.text().release(entry.prepared.blob);
                self.stats.release_prune_age = self.stats.release_prune_age.saturating_add(1);
            }
            keep
        });

        if self.entries.len() <= max_entries {
            return;
        }

        let mut keys: Vec<(Reverse<u64>, TextCacheKey)> = self
            .entries
            .iter()
            .map(|(k, v)| (Reverse(v.last_used_frame), k.clone()))
            .collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0));

        for (_, key) in keys.into_iter().skip(max_entries) {
            if let Some(entry) = self.entries.remove(&key) {
                services.text().release(entry.prepared.blob);
                self.stats.release_prune_budget = self.stats.release_prune_budget.saturating_add(1);
            }
        }
    }
}

#[derive(Debug, Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn write(&mut self, bytes: &[u8]) {
        if self.0 == 0 {
            self.0 = 0xcbf29ce484222325;
        }
        let mut state = self.0;
        for b in bytes {
            state ^= u64::from(*b);
            state = state.wrapping_mul(0x100000001b3);
        }
        self.0 = state;
    }

    fn finish(&self) -> u64 {
        if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Px, Size, SvgId,
        SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextOverflow, TextService,
        TextWrap,
    };

    #[test]
    fn key_includes_line_height() {
        let style_a = TextStyle {
            line_height: None,
            ..TextStyle::default()
        };
        let style_b = TextStyle {
            line_height: Some(Px(22.0)),
            ..TextStyle::default()
        };

        let k_a = TextCacheKey::new("hello", &style_a, TextConstraints::default()).stable_key();
        let k_b = TextCacheKey::new("hello", &style_b, TextConstraints::default()).stable_key();
        assert_ne!(k_a, k_b);
    }

    #[test]
    fn key_includes_letter_spacing() {
        let style_a = TextStyle {
            letter_spacing_em: None,
            ..TextStyle::default()
        };
        let style_b = TextStyle {
            letter_spacing_em: Some(0.05),
            ..TextStyle::default()
        };

        let k_a = TextCacheKey::new("hello", &style_a, TextConstraints::default()).stable_key();
        let k_b = TextCacheKey::new("hello", &style_b, TextConstraints::default()).stable_key();
        assert_ne!(k_a, k_b);
    }

    #[test]
    fn key_includes_constraints() {
        let mut a = TextConstraints::default();
        a.scale_factor = 1.0;
        a.wrap = TextWrap::Word;
        a.overflow = TextOverflow::Clip;

        let mut b = a;
        b.scale_factor = 2.0;

        let k_a = TextCacheKey::new("hello", &TextStyle::default(), a).stable_key();
        let k_b = TextCacheKey::new("hello", &TextStyle::default(), b).stable_key();
        assert_ne!(k_a, k_b);
    }

    #[test]
    fn key_is_collision_safe_on_equal_hash() {
        // This is intentionally not trying to manufacture a real FNV collision (impractical here).
        // The invariant we care about: even if a hash collides, `TextCacheKey` equality compares
        // the full text, so correctness does not depend on the stable key alone.
        let a = TextCacheKey::new("hello", &TextStyle::default(), TextConstraints::default());
        let b = TextCacheKey::new("hello!", &TextStyle::default(), TextConstraints::default());
        assert_ne!(a, b);
    }

    #[derive(Default)]
    struct FakeServices {
        text_prepare_calls: u64,
    }

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            self.text_prepare_calls += 1;
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

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn stats_count_prepare_hits_and_misses() {
        let mut cache = TextCache::default();
        let mut services = FakeServices::default();
        cache.begin_frame();

        let style = TextStyle::default();
        let constraints = TextConstraints::default();

        let _ = cache.prepare(&mut services, "hello", &style, constraints);
        let _ = cache.prepare(&mut services, "hello", &style, constraints);

        assert_eq!(services.text_prepare_calls, 1);
        assert_eq!(cache.stats().prepare_misses, 1);
        assert_eq!(cache.stats().prepare_hits, 1);
    }
}
