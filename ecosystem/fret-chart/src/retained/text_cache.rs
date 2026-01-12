use fret_canvas::text::{PreparedText, TextCache};
use fret_core::{TextConstraints, TextStyle, UiServices};

#[derive(Debug, Default)]
pub(crate) struct TextCacheGroup {
    cache: TextCache,
    key: Option<u64>,
}

impl TextCacheGroup {
    pub(crate) fn clear(&mut self, services: &mut dyn UiServices) {
        self.cache.clear(services);
        self.key = None;
    }

    pub(crate) fn reset_if_key_changed(&mut self, services: &mut dyn UiServices, key: u64) {
        if self.key == Some(key) {
            return;
        }
        self.cache.clear(services);
        self.key = Some(key);
    }

    pub(crate) fn prepare(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        self.cache.prepare(services, text, style, constraints)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct KeyBuilder(u64);

impl KeyBuilder {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn finish(self) -> u64 {
        self.0
    }

    pub(crate) fn mix_u64(&mut self, v: u64) {
        self.0 ^= v
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(self.0 << 6)
            .wrapping_add(self.0 >> 2);
    }

    pub(crate) fn mix_bool(&mut self, v: bool) {
        self.mix_u64(u64::from(v));
    }

    pub(crate) fn mix_f32_bits(&mut self, v: f32) {
        self.mix_u64(u64::from(v.to_bits()));
    }

    pub(crate) fn mix_f64_bits(&mut self, v: f64) {
        self.mix_u64(v.to_bits());
    }

    pub(crate) fn mix_str(&mut self, s: &str) {
        for b in s.as_bytes() {
            self.mix_u64(u64::from(*b));
        }
    }
}
