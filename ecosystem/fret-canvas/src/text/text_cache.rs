use fret_core::{TextBlobId, TextConstraints, TextInput, TextMetrics, TextStyle, UiServices};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

fn hash_value<T: Hash>(value: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A prepared text blob with metrics and a stable cache key.
#[derive(Debug, Clone, Copy)]
pub struct PreparedText {
    pub blob: TextBlobId,
    pub metrics: TextMetrics,
    pub key: u64,
}

/// A small keyed cache for prepared text blobs.
///
/// The cache owns the `TextBlobId`s and must be cleared (or dropped) with access to `UiServices`
/// so resources can be released deterministically.
#[derive(Debug, Default)]
pub struct TextCache {
    entries: HashMap<u64, PreparedText>,
}

impl TextCache {
    /// Releases all cached blobs.
    pub fn clear(&mut self, services: &mut dyn UiServices) {
        for t in self.entries.values() {
            services.text().release(t.blob);
        }
        self.entries.clear();
    }

    /// Prepares text and caches it by a stable key derived from `(text, style, constraints)`.
    ///
    /// Note: this currently includes `constraints.scale_factor` in the key; callers that apply
    /// additional view-zoom scaling should incorporate that into `constraints`.
    pub fn prepare(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        let key = Self::key_for(text, style, constraints);
        match self.entries.entry(key) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let (blob, metrics) = services
                    .text()
                    .prepare(TextInput::plain(text, style), constraints);
                let prepared = PreparedText { blob, metrics, key };
                e.insert(prepared);
                prepared
            }
        }
    }

    fn key_for(text: &str, style: &TextStyle, constraints: TextConstraints) -> u64 {
        let mut state = 0u64;
        for b in text.as_bytes() {
            state ^= u64::from(*b)
                .wrapping_add(0x9e3779b97f4a7c15)
                .wrapping_add(state << 6)
                .wrapping_add(state >> 2);
        }
        state ^= hash_value(&style.font)
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(style.weight.0)
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(style.slant as u8)
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(style.size.0.to_bits())
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(style.line_height.map(|v| v.0.to_bits()).unwrap_or(0))
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(style.letter_spacing_em.map(|v| v.to_bits()).unwrap_or(0))
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(constraints.scale_factor.to_bits())
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= u64::from(constraints.max_width.map(|v| v.0.to_bits()).unwrap_or(0))
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= hash_value(&constraints.wrap)
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state ^= hash_value(&constraints.overflow)
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, TextOverflow, TextWrap};

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

        let k_a = TextCache::key_for("hello", &style_a, TextConstraints::default());
        let k_b = TextCache::key_for("hello", &style_b, TextConstraints::default());
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

        let k_a = TextCache::key_for("hello", &style_a, TextConstraints::default());
        let k_b = TextCache::key_for("hello", &style_b, TextConstraints::default());
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

        let k_a = TextCache::key_for("hello", &TextStyle::default(), a);
        let k_b = TextCache::key_for("hello", &TextStyle::default(), b);
        assert_ne!(k_a, k_b);
    }
}
