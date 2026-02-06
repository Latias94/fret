use fret_core::{Corners, Px, Rect, TextStyle};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Utility helpers for building view-cache cache keys.
///
/// The view-cache boundary (`ElementContext::view_cache`) relies on a deterministic `u64` key to
/// decide whether cached output can be reused. Callers should include any inputs that affect
/// rendered output but may not otherwise force a re-render (e.g. text style, clip/mask geometry,
/// view-specific configuration).
pub struct CacheKeyBuilder {
    hasher: DefaultHasher,
}

impl CacheKeyBuilder {
    pub fn new() -> Self {
        Self {
            hasher: DefaultHasher::new(),
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        self.hasher.write_u64(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.hasher.write_u32(value);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.hasher.write_u8(if value { 1 } else { 0 });
    }

    pub fn write_px(&mut self, value: Px) {
        self.hasher.write_u32(value.0.to_bits());
    }

    pub fn write_rect(&mut self, rect: Rect) {
        self.write_px(rect.origin.x);
        self.write_px(rect.origin.y);
        self.write_px(rect.size.width);
        self.write_px(rect.size.height);
    }

    pub fn write_corners(&mut self, corners: Corners) {
        self.write_px(corners.top_left);
        self.write_px(corners.top_right);
        self.write_px(corners.bottom_right);
        self.write_px(corners.bottom_left);
    }

    pub fn write_text_style(&mut self, style: &TextStyle) {
        // Note: `TextStyle` does not implement `Hash` directly. We intentionally hash the style's
        // fields in a stable, bitwise way.
        style.font.hash(&mut self.hasher);
        style.weight.hash(&mut self.hasher);
        style.slant.hash(&mut self.hasher);

        self.hasher.write_u32(style.size.0.to_bits());
        self.hasher
            .write_u32(style.line_height.map(|h| h.0.to_bits()).unwrap_or(0));
        self.hasher
            .write_u32(style.letter_spacing_em.map(f32::to_bits).unwrap_or(0));
    }

    pub fn finish(self) -> u64 {
        self.hasher.finish()
    }
}

impl Default for CacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn mix(seed: u64, value: u64) -> u64 {
    let mut b = CacheKeyBuilder::new();
    b.write_u64(seed);
    b.write_u64(value);
    b.finish()
}

pub fn rect_key(rect: Rect) -> u64 {
    let mut b = CacheKeyBuilder::new();
    b.write_rect(rect);
    b.finish()
}

pub fn corners_key(corners: Corners) -> u64 {
    let mut b = CacheKeyBuilder::new();
    b.write_corners(corners);
    b.finish()
}

pub fn text_style_key(style: &TextStyle) -> u64 {
    let mut b = CacheKeyBuilder::new();
    b.write_text_style(style);
    b.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::FontId;
    use fret_core::text::FontWeight;
    use fret_core::text::TextSlant;

    #[test]
    fn text_style_key_changes_when_style_changes() {
        let style = TextStyle {
            font: FontId::default(),
            size: Px(13.0),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: None,
            letter_spacing_em: None,
        };
        let a = text_style_key(&style);
        let b = text_style_key(&TextStyle {
            size: Px(14.0),
            ..style.clone()
        });
        assert_ne!(a, b);
    }

    #[test]
    fn mix_is_stable_for_same_inputs() {
        assert_eq!(mix(1, 2), mix(1, 2));
        assert_ne!(mix(1, 2), mix(2, 1));
    }
}
