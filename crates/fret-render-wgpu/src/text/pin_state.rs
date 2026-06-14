use super::atlas::{GlyphKey, GlyphKeyBuckets, GlyphPinKeys};
use super::blob_state::TextBlobState;
use fret_core::{Scene, TextBlobId};
use std::collections::HashMap;

pub(crate) struct TextPinState {
    mask: Vec<Vec<GlyphKey>>,
    color: Vec<Vec<GlyphKey>>,
    subpixel: Vec<Vec<GlyphKey>>,
    bucket_signatures: Vec<Option<ScenePinBucketSignature>>,
    atlas_reset_generation: u64,
    scene_cache: ScenePinKeyCache,
}

impl TextPinState {
    pub(crate) fn with_ring_len(ring_len: usize) -> Self {
        Self {
            mask: vec![Vec::new(); ring_len],
            color: vec![Vec::new(); ring_len],
            subpixel: vec![Vec::new(); ring_len],
            bucket_signatures: vec![None; ring_len],
            atlas_reset_generation: 0,
            scene_cache: ScenePinKeyCache::default(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.mask.iter_mut().for_each(|bucket| bucket.clear());
        self.color.iter_mut().for_each(|bucket| bucket.clear());
        self.subpixel.iter_mut().for_each(|bucket| bucket.clear());
        self.bucket_signatures
            .iter_mut()
            .for_each(|signature| *signature = None);
        self.scene_cache.clear();
    }

    pub(crate) fn clear_for_atlas_reset_generation(&mut self, generation: u64) {
        if self.atlas_reset_generation == generation {
            return;
        }
        self.clear();
        self.atlas_reset_generation = generation;
    }

    pub(crate) fn ring_len(&self) -> usize {
        self.mask
            .len()
            .min(self.color.len())
            .min(self.subpixel.len())
            .min(self.bucket_signatures.len())
    }

    pub(crate) fn bucket(&self, bucket: usize) -> Option<(&[GlyphKey], &[GlyphKey], &[GlyphKey])> {
        if bucket >= self.ring_len() {
            return None;
        }
        Some((
            &self.mask[bucket],
            &self.color[bucket],
            &self.subpixel[bucket],
        ))
    }

    pub(crate) fn replace_bucket(
        &mut self,
        bucket: usize,
        mask: Vec<GlyphKey>,
        color: Vec<GlyphKey>,
        subpixel: Vec<GlyphKey>,
        signature: Option<ScenePinBucketSignature>,
    ) {
        self.mask[bucket] = mask;
        self.color[bucket] = color;
        self.subpixel[bucket] = subpixel;
        self.bucket_signatures[bucket] = signature;
    }

    pub(crate) fn try_reuse_scene_bucket(
        &self,
        bucket: usize,
        scene: &Scene,
        blob_state: &TextBlobState,
    ) -> Option<ScenePinBucketReuse> {
        let signature = self.bucket_signatures.get(bucket)?.as_ref()?;
        let current = scene_text_signature_if_all_live(scene, blob_state)?;
        (signature.scene == current).then_some(ScenePinBucketReuse {
            scene_text_blobs: current.len(),
            pinned_glyph_keys: signature.pinned_glyph_keys,
        })
    }

    pub(crate) fn collect_scene_pinned_keys(
        &mut self,
        scene: &Scene,
        blob_state: &TextBlobState,
    ) -> ScenePinCollection {
        self.scene_cache.collect(scene, blob_state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScenePinBucketSignature {
    scene: SceneTextSignature,
    pinned_glyph_keys: usize,
}

impl ScenePinBucketSignature {
    pub(crate) fn new(scene: SceneTextSignature, pinned_glyph_keys: usize) -> Self {
        Self {
            scene,
            pinned_glyph_keys,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScenePinBucketReuse {
    pub(crate) scene_text_blobs: usize,
    pub(crate) pinned_glyph_keys: usize,
}

pub(crate) struct ScenePinCollection {
    pub(crate) buckets: GlyphKeyBuckets,
    pub(crate) scene_text_blobs: usize,
    pub(crate) signature: Option<SceneTextSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SceneTextSignature {
    text_blobs: Vec<TextBlobId>,
}

impl SceneTextSignature {
    fn empty() -> Self {
        Self { text_blobs: vec![] }
    }

    fn push(&mut self, text: TextBlobId) {
        self.text_blobs.push(text);
    }

    fn len(&self) -> usize {
        self.text_blobs.len()
    }
}

#[derive(Default)]
struct ScenePinKeyCache {
    blob_entries: HashMap<TextBlobId, ScenePinBlobEntry>,
    current_counts: HashMap<TextBlobId, u32>,
    current_blobs: Vec<TextBlobId>,
    stale_blobs: Vec<TextBlobId>,
    mask_ref_counts: HashMap<GlyphKey, u32>,
    color_ref_counts: HashMap<GlyphKey, u32>,
    subpixel_ref_counts: HashMap<GlyphKey, u32>,
}

impl ScenePinKeyCache {
    fn clear(&mut self) {
        self.blob_entries.clear();
        self.current_counts.clear();
        self.current_blobs.clear();
        self.stale_blobs.clear();
        self.mask_ref_counts.clear();
        self.color_ref_counts.clear();
        self.subpixel_ref_counts.clear();
    }

    fn collect(&mut self, scene: &Scene, blob_state: &TextBlobState) -> ScenePinCollection {
        self.current_counts.clear();
        let mut signature = SceneTextSignature::empty();
        let mut all_text_blobs_live = true;
        for &text in scene.text_blob_ids() {
            if !blob_state.blobs.contains_key(text) {
                all_text_blobs_live = false;
                continue;
            }
            signature.push(text);
            let count = self.current_counts.entry(text).or_insert(0);
            *count = count.saturating_add(1);
        }

        self.reconcile(blob_state);
        ScenePinCollection {
            buckets: self.to_buckets(),
            scene_text_blobs: signature.len(),
            signature: all_text_blobs_live.then_some(signature),
        }
    }

    fn reconcile(&mut self, blob_state: &TextBlobState) {
        self.stale_blobs.clear();
        for &text in self.blob_entries.keys() {
            if !self.current_counts.contains_key(&text) {
                self.stale_blobs.push(text);
            }
        }

        let mut stale_blobs = std::mem::take(&mut self.stale_blobs);
        for text in stale_blobs.drain(..) {
            if let Some(entry) = self.blob_entries.remove(&text) {
                self.dec_pin_keys(&entry.pin_keys);
            }
        }
        self.stale_blobs = stale_blobs;

        self.current_blobs.clear();
        self.current_blobs
            .extend(self.current_counts.keys().copied());
        let mut current_blobs = std::mem::take(&mut self.current_blobs);
        for text in current_blobs.drain(..) {
            if self.blob_entries.contains_key(&text) {
                continue;
            }

            let Some(blob) = blob_state.blobs.get(text) else {
                continue;
            };
            let pin_keys = blob.shape().pin_keys().clone();
            self.inc_pin_keys(&pin_keys);
            self.blob_entries
                .insert(text, ScenePinBlobEntry { pin_keys });
        }
        self.current_blobs = current_blobs;
    }

    fn to_buckets(&self) -> GlyphKeyBuckets {
        GlyphKeyBuckets::from_unique_key_iters(
            self.mask_ref_counts.keys().copied(),
            self.color_ref_counts.keys().copied(),
            self.subpixel_ref_counts.keys().copied(),
        )
    }

    fn inc_pin_keys(&mut self, keys: &GlyphPinKeys) {
        inc_ref_counts(&mut self.mask_ref_counts, keys.mask_keys());
        inc_ref_counts(&mut self.color_ref_counts, keys.color_keys());
        inc_ref_counts(&mut self.subpixel_ref_counts, keys.subpixel_keys());
    }

    fn dec_pin_keys(&mut self, keys: &GlyphPinKeys) {
        dec_ref_counts(&mut self.mask_ref_counts, keys.mask_keys());
        dec_ref_counts(&mut self.color_ref_counts, keys.color_keys());
        dec_ref_counts(&mut self.subpixel_ref_counts, keys.subpixel_keys());
    }
}

struct ScenePinBlobEntry {
    pin_keys: GlyphPinKeys,
}

fn scene_text_signature_if_all_live(
    scene: &Scene,
    blob_state: &TextBlobState,
) -> Option<SceneTextSignature> {
    let mut signature = SceneTextSignature::empty();
    for &text in scene.text_blob_ids() {
        if !blob_state.blobs.contains_key(text) {
            return None;
        }
        signature.push(text);
    }
    Some(signature)
}

fn inc_ref_counts(counts: &mut HashMap<GlyphKey, u32>, keys: &[GlyphKey]) {
    for &key in keys {
        let count = counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }
}

fn dec_ref_counts(counts: &mut HashMap<GlyphKey, u32>, keys: &[GlyphKey]) {
    for &key in keys {
        let Some(count) = counts.get_mut(&key) else {
            continue;
        };
        if *count <= 1 {
            counts.remove(&key);
        } else {
            *count -= 1;
        }
    }
}
