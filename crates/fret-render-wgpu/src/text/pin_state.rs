use super::atlas::{GlyphKey, GlyphPinKeys};
use super::blob_state::TextBlobState;
use fret_core::{Scene, TextBlobId};
use rustc_hash::{FxHashMap, FxHashSet};

pub(crate) struct TextPinState {
    mask: Vec<Vec<GlyphKey>>,
    color: Vec<Vec<GlyphKey>>,
    subpixel: Vec<Vec<GlyphKey>>,
    mask_members: Vec<FxHashSet<GlyphKey>>,
    color_members: Vec<FxHashSet<GlyphKey>>,
    subpixel_members: Vec<FxHashSet<GlyphKey>>,
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
            mask_members: vec![FxHashSet::default(); ring_len],
            color_members: vec![FxHashSet::default(); ring_len],
            subpixel_members: vec![FxHashSet::default(); ring_len],
            bucket_signatures: vec![None; ring_len],
            atlas_reset_generation: 0,
            scene_cache: ScenePinKeyCache::default(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.mask.iter_mut().for_each(|bucket| bucket.clear());
        self.color.iter_mut().for_each(|bucket| bucket.clear());
        self.subpixel.iter_mut().for_each(|bucket| bucket.clear());
        self.mask_members
            .iter_mut()
            .for_each(|members| members.clear());
        self.color_members
            .iter_mut()
            .for_each(|members| members.clear());
        self.subpixel_members
            .iter_mut()
            .for_each(|members| members.clear());
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
            .min(self.mask_members.len())
            .min(self.color_members.len())
            .min(self.subpixel_members.len())
            .min(self.bucket_signatures.len())
    }

    pub(crate) fn apply_bucket_delta(
        &mut self,
        bucket: usize,
        added: (&[GlyphKey], &[GlyphKey], &[GlyphKey]),
        removed: (&[GlyphKey], &[GlyphKey], &[GlyphKey]),
    ) {
        apply_bucket_vector_delta(&mut self.mask[bucket], added.0, removed.0);
        apply_bucket_vector_delta(&mut self.color[bucket], added.1, removed.1);
        apply_bucket_vector_delta(&mut self.subpixel[bucket], added.2, removed.2);
        apply_member_delta(&mut self.mask_members[bucket], added.0, removed.0);
        apply_member_delta(&mut self.color_members[bucket], added.1, removed.1);
        apply_member_delta(&mut self.subpixel_members[bucket], added.2, removed.2);
    }

    pub(crate) fn replace_bucket_signature(
        &mut self,
        bucket: usize,
        signature: Option<ScenePinBucketSignature>,
    ) {
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

    pub(crate) fn collect_scene_pin_snapshot(
        &mut self,
        scene: &Scene,
        blob_state: &TextBlobState,
    ) -> ScenePinSnapshot {
        self.scene_cache.collect_snapshot(scene, blob_state)
    }

    pub(crate) fn current_scene_delta_for_bucket(
        &self,
        bucket: usize,
    ) -> Option<super::atlas::GlyphPinBucketDelta> {
        if bucket >= self.ring_len() {
            return None;
        }
        Some(self.scene_cache.delta_from_bucket(
            &self.mask[bucket],
            &self.mask_members[bucket],
            &self.color[bucket],
            &self.color_members[bucket],
            &self.subpixel[bucket],
            &self.subpixel_members[bucket],
        ))
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

pub(crate) struct ScenePinSnapshot {
    pub(crate) scene_text_blobs: usize,
    pub(crate) pinned_glyph_keys: usize,
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
    blob_entries: FxHashMap<TextBlobId, ScenePinBlobEntry>,
    current_counts: FxHashMap<TextBlobId, u32>,
    current_blobs: Vec<TextBlobId>,
    stale_blobs: Vec<TextBlobId>,
    mask_ref_counts: FxHashMap<GlyphKey, u32>,
    color_ref_counts: FxHashMap<GlyphKey, u32>,
    subpixel_ref_counts: FxHashMap<GlyphKey, u32>,
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

    fn collect_snapshot(&mut self, scene: &Scene, blob_state: &TextBlobState) -> ScenePinSnapshot {
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
        ScenePinSnapshot {
            scene_text_blobs: signature.len(),
            pinned_glyph_keys: self.current_pin_key_count(),
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

    fn current_pin_key_count(&self) -> usize {
        self.mask_ref_counts
            .len()
            .saturating_add(self.color_ref_counts.len())
            .saturating_add(self.subpixel_ref_counts.len())
    }

    fn delta_from_bucket(
        &self,
        existing_mask: &[GlyphKey],
        existing_mask_members: &FxHashSet<GlyphKey>,
        existing_color: &[GlyphKey],
        existing_color_members: &FxHashSet<GlyphKey>,
        existing_subpixel: &[GlyphKey],
        existing_subpixel_members: &FxHashSet<GlyphKey>,
    ) -> super::atlas::GlyphPinBucketDelta {
        let (retain_mask_len, add_mask, remove_mask) = bucket_delta_from_ref_counts(
            &self.mask_ref_counts,
            existing_mask,
            existing_mask_members,
        );
        let (retain_color_len, add_color, remove_color) = bucket_delta_from_ref_counts(
            &self.color_ref_counts,
            existing_color,
            existing_color_members,
        );
        let (retain_subpixel_len, add_subpixel, remove_subpixel) = bucket_delta_from_ref_counts(
            &self.subpixel_ref_counts,
            existing_subpixel,
            existing_subpixel_members,
        );

        super::atlas::GlyphPinBucketDelta {
            retained_len: retain_mask_len
                .saturating_add(retain_color_len)
                .saturating_add(retain_subpixel_len),
            added: (add_mask, add_color, add_subpixel),
            removed: (remove_mask, remove_color, remove_subpixel),
        }
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

fn inc_ref_counts(counts: &mut FxHashMap<GlyphKey, u32>, keys: &[GlyphKey]) {
    for &key in keys {
        let count = counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }
}

fn dec_ref_counts(counts: &mut FxHashMap<GlyphKey, u32>, keys: &[GlyphKey]) {
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

fn bucket_delta_from_ref_counts(
    current_counts: &FxHashMap<GlyphKey, u32>,
    existing: &[GlyphKey],
    existing_members: &FxHashSet<GlyphKey>,
) -> (usize, Vec<GlyphKey>, Vec<GlyphKey>) {
    if current_counts.len() == existing.len()
        && existing.iter().all(|key| current_counts.contains_key(key))
    {
        return (existing.len(), Vec::new(), Vec::new());
    }

    let mut retained_len = 0_usize;
    let mut removed = Vec::new();
    for &key in existing {
        if current_counts.contains_key(&key) {
            retained_len = retained_len.saturating_add(1);
        } else {
            removed.push(key);
        }
    }

    let mut added = Vec::new();
    for &key in current_counts.keys() {
        if !existing_members.contains(&key) {
            added.push(key);
        }
    }

    (retained_len, added, removed)
}

fn apply_bucket_vector_delta(bucket: &mut Vec<GlyphKey>, added: &[GlyphKey], removed: &[GlyphKey]) {
    if !removed.is_empty() {
        if removed.len() == 1 {
            let key = removed[0];
            bucket.retain(|existing| *existing != key);
        } else {
            let removed: FxHashSet<GlyphKey> = removed.iter().copied().collect();
            bucket.retain(|existing| !removed.contains(existing));
        }
    }
    bucket.extend_from_slice(added);
}

fn apply_member_delta(members: &mut FxHashSet<GlyphKey>, added: &[GlyphKey], removed: &[GlyphKey]) {
    for key in removed {
        members.remove(key);
    }
    members.extend(added.iter().copied());
}
