use super::{TextBlob, TextSystem};
use fret_core::TextBlobId;
use std::sync::Arc;

impl TextSystem {
    pub fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blob_state.blobs.get(id)
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let entries = fret_render_text::cache_tuning::released_blob_cache_entries();

        let Some(b) = self.blob_state.blobs.get_mut(blob) else {
            return;
        };

        if b.ref_count > 1 {
            b.ref_count = b.ref_count.saturating_sub(1);
            return;
        }

        if b.ref_count == 0 {
            return;
        }

        if entries > 0 {
            b.ref_count = 0;
            self.insert_released_blob(blob, entries);
            return;
        }

        self.evict_blob(blob);
    }

    pub(super) fn remove_released_blob(&mut self, id: TextBlobId) {
        if !self.blob_state.released_blob_set.remove(&id) {
            return;
        }
        if let Some(pos) = self
            .blob_state
            .released_blob_lru
            .iter()
            .position(|v| *v == id)
        {
            self.blob_state.released_blob_lru.remove(pos);
        }
    }

    fn insert_released_blob(&mut self, id: TextBlobId, entries: usize) {
        if entries == 0 {
            return;
        }

        if !self.blob_state.released_blob_set.insert(id)
            && let Some(pos) = self
                .blob_state
                .released_blob_lru
                .iter()
                .position(|v| *v == id)
        {
            self.blob_state.released_blob_lru.remove(pos);
        }
        self.blob_state.released_blob_lru.push_back(id);

        while self.blob_state.released_blob_lru.len() > entries {
            let Some(evict) = self.blob_state.released_blob_lru.pop_front() else {
                break;
            };
            self.blob_state.released_blob_set.remove(&evict);
            if self
                .blob_state
                .blobs
                .get(evict)
                .is_some_and(|b| b.ref_count > 0)
            {
                continue;
            }
            self.evict_blob(evict);
        }
    }

    fn evict_blob(&mut self, blob: TextBlobId) {
        self.remove_released_blob(blob);

        let remove_shape = self
            .blob_state
            .blobs
            .get(blob)
            .is_some_and(|b| Arc::strong_count(&b.shape) == 2);

        if let Some(key) = self.blob_state.blob_key_by_id.remove(&blob) {
            self.blob_state.blob_cache.remove(&key);
            if remove_shape {
                let shape_key = fret_render_text::cache_keys::TextShapeKey::from_blob_key(&key);
                self.layout_cache.shape_cache.remove(&shape_key);
            }
        }
        let _ = self.blob_state.blobs.remove(blob);
    }
}
