use super::TextShape;
use fret_render_text::{TextMeasureCaches, TextShapeKey};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

pub(crate) struct TextLayoutCacheState {
    shape_cache: HashMap<TextShapeKey, TextShapeCacheEntry>,
    shape_cache_lru: VecDeque<(TextShapeKey, u64)>,
    shape_cache_limit: usize,
    shape_cache_generation: u64,
    pub(crate) measure: TextMeasureCaches,
}

struct TextShapeCacheEntry {
    shape: Arc<TextShape>,
    generation: u64,
}

impl TextLayoutCacheState {
    pub(crate) fn new() -> Self {
        Self::with_shape_cache_limit(fret_render_text::prepared_shape_cache_entries())
    }

    fn with_shape_cache_limit(shape_cache_limit: usize) -> Self {
        let shape_entries = shape_cache_limit.max(1);
        Self {
            shape_cache: HashMap::with_capacity(shape_entries.min(65_536)),
            shape_cache_lru: VecDeque::with_capacity(shape_entries.min(65_536)),
            shape_cache_limit: shape_entries,
            shape_cache_generation: 0,
            measure: TextMeasureCaches::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.shape_cache.clear();
        self.shape_cache_lru.clear();
        self.measure.clear();
    }

    pub(crate) fn shape_cache_limit(&self) -> usize {
        self.shape_cache_limit
    }

    pub(crate) fn shape_cache_len(&self) -> usize {
        self.shape_cache.len()
    }

    pub(crate) fn shapes(&self) -> impl Iterator<Item = &Arc<TextShape>> {
        self.shape_cache.values().map(|entry| &entry.shape)
    }

    #[cfg(test)]
    pub(crate) fn contains_shape(&self, key: &TextShapeKey) -> bool {
        self.shape_cache.contains_key(key)
    }

    #[cfg(test)]
    pub(crate) fn set_shape_cache_limit_for_tests(&mut self, shape_cache_limit: usize) -> u64 {
        self.shape_cache_limit = shape_cache_limit.max(1);
        self.prune_shape_cache()
    }

    pub(crate) fn get_shape(&mut self, key: &TextShapeKey) -> Option<Arc<TextShape>> {
        let shape = self.shape_cache.get(key)?.shape.clone();
        self.touch_shape_cache_key(key.clone());
        Some(shape)
    }

    pub(crate) fn insert_shape(&mut self, key: TextShapeKey, shape: Arc<TextShape>) -> u64 {
        let generation = self.next_shape_cache_generation();
        self.shape_cache
            .insert(key.clone(), TextShapeCacheEntry { shape, generation });
        self.shape_cache_lru.push_back((key, generation));
        self.compact_shape_cache_lru_if_needed();
        self.prune_shape_cache()
    }

    pub(crate) fn remove_shape(&mut self, key: &TextShapeKey) -> bool {
        self.shape_cache.remove(key).is_some()
    }

    fn touch_shape_cache_key(&mut self, key: TextShapeKey) {
        let generation = self.next_shape_cache_generation();
        if let Some(entry) = self.shape_cache.get_mut(&key) {
            entry.generation = generation;
            self.shape_cache_lru.push_back((key, generation));
            self.compact_shape_cache_lru_if_needed();
        }
    }

    fn next_shape_cache_generation(&mut self) -> u64 {
        self.shape_cache_generation = self.shape_cache_generation.wrapping_add(1);
        self.shape_cache_generation
    }

    fn compact_shape_cache_lru_if_needed(&mut self) {
        let threshold = self
            .shape_cache
            .len()
            .max(self.shape_cache_limit)
            .max(1)
            .saturating_mul(4)
            .max(64);
        if self.shape_cache_lru.len() <= threshold {
            return;
        }
        let shape_cache = &self.shape_cache;
        self.shape_cache_lru.retain(|(key, generation)| {
            shape_cache
                .get(key)
                .is_some_and(|entry| entry.generation == *generation)
        });
    }

    fn prune_shape_cache(&mut self) -> u64 {
        let mut evicted = 0_u64;
        while self.shape_cache.len() > self.shape_cache_limit {
            let Some((evict, generation)) = self.shape_cache_lru.pop_front() else {
                break;
            };
            if !self
                .shape_cache
                .get(&evict)
                .is_some_and(|entry| entry.generation == generation)
            {
                continue;
            }
            if self.shape_cache.remove(&evict).is_some() {
                evicted = evicted.saturating_add(1);
            }
        }
        evicted
    }
}
