use super::TextShape;
use fret_render_text::cache_keys::TextShapeKey;
use fret_render_text::measure::TextMeasureCaches;
use std::{collections::HashMap, sync::Arc};

pub(crate) struct TextLayoutCacheState {
    pub(crate) shape_cache: HashMap<TextShapeKey, Arc<TextShape>>,
    pub(crate) measure: TextMeasureCaches,
}

impl TextLayoutCacheState {
    pub(crate) fn new() -> Self {
        Self {
            shape_cache: HashMap::new(),
            measure: TextMeasureCaches::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.shape_cache.clear();
        self.measure.clear();
    }
}
