use super::types::{SceneEncoding, SceneEncodingCacheKey};

#[derive(Default)]
pub(super) struct SceneEncodingCache {
    key: Option<SceneEncodingCacheKey>,
    cache: SceneEncoding,
    scratch: SceneEncoding,
}

impl SceneEncodingCache {
    pub(super) fn is_hit(&self, key: SceneEncodingCacheKey) -> bool {
        self.key == Some(key)
    }

    pub(super) fn key(&self) -> Option<SceneEncodingCacheKey> {
        self.key
    }

    pub(super) fn take_for_frame(&mut self, cache_hit: bool) -> SceneEncoding {
        if cache_hit {
            std::mem::take(&mut self.cache)
        } else {
            std::mem::take(&mut self.scratch)
        }
    }

    pub(super) fn note_miss(&mut self, key: SceneEncodingCacheKey) {
        // Preserve the old cache's allocations for reuse.
        self.scratch = std::mem::take(&mut self.cache);
        self.key = Some(key);
    }

    pub(super) fn store_after_frame(
        &mut self,
        key: SceneEncodingCacheKey,
        cache_hit: bool,
        encoding: SceneEncoding,
    ) {
        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.key = Some(key);
        }
        self.cache = encoding;
    }
}

#[cfg(test)]
impl SceneEncodingCache {
    pub(super) fn cache_key(&self) -> Option<SceneEncodingCacheKey> {
        self.key()
    }

    pub(super) fn cache(&self) -> &SceneEncoding {
        &self.cache
    }
}
