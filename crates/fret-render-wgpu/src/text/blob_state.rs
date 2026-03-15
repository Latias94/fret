use super::TextBlob;
use fret_core::TextBlobId;
use fret_render_text::TextBlobKey;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet, VecDeque};

pub(crate) struct TextBlobState {
    pub(crate) blobs: SlotMap<TextBlobId, TextBlob>,
    pub(crate) blob_cache: HashMap<TextBlobKey, TextBlobId>,
    pub(crate) blob_key_by_id: HashMap<TextBlobId, TextBlobKey>,
    pub(crate) released_blob_lru: VecDeque<TextBlobId>,
    pub(crate) released_blob_set: HashSet<TextBlobId>,
}

impl TextBlobState {
    pub(crate) fn new() -> Self {
        Self {
            blobs: SlotMap::with_key(),
            blob_cache: HashMap::new(),
            blob_key_by_id: HashMap::new(),
            released_blob_lru: VecDeque::new(),
            released_blob_set: HashSet::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.blobs.clear();
        self.blob_cache.clear();
        self.blob_key_by_id.clear();
        self.clear_released();
    }

    pub(crate) fn clear_released(&mut self) {
        self.released_blob_lru.clear();
        self.released_blob_set.clear();
    }
}
