use super::atlas::{GlyphKey, GlyphPinKeys};
use fret_core::TextBlobId;
use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::Key;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub(crate) struct TextPinState {
    mask: Vec<Vec<GlyphKey>>,
    color: Vec<Vec<GlyphKey>>,
    subpixel: Vec<Vec<GlyphKey>>,
    mask_members: Vec<FxHashSet<GlyphKey>>,
    color_members: Vec<FxHashSet<GlyphKey>>,
    subpixel_members: Vec<FxHashSet<GlyphKey>>,
    bucket_signatures: Vec<Option<TextBlobPinBucketSignature>>,
    atlas_reset_generation: u64,
    text_blob_cache: TextBlobPinKeyCache,
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
            text_blob_cache: TextBlobPinKeyCache::default(),
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
        self.text_blob_cache.clear();
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
        signature: Option<TextBlobPinBucketSignature>,
    ) {
        self.bucket_signatures[bucket] = signature;
    }

    pub(crate) fn try_reuse_text_residency_bucket(
        &self,
        bucket: usize,
        residency: &TextFrameResidency,
    ) -> Option<TextBlobPinBucketReuse> {
        let signature = self.bucket_signatures.get(bucket)?.as_ref()?;
        let current = residency.signature()?;
        (signature.text_blobs == current).then_some(TextBlobPinBucketReuse {
            scene_text_blobs: current.len(),
            pinned_glyph_keys: signature.pinned_glyph_keys,
        })
    }

    pub(crate) fn collect_text_residency_pin_snapshot(
        &mut self,
        residency: &TextFrameResidency,
    ) -> TextBlobPinSnapshot {
        self.text_blob_cache.collect_snapshot(residency)
    }

    pub(crate) fn current_scene_delta_for_bucket(
        &self,
        bucket: usize,
    ) -> Option<super::atlas::GlyphPinBucketDelta> {
        if bucket >= self.ring_len() {
            return None;
        }
        Some(self.text_blob_cache.delta_from_bucket(
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
pub(crate) struct TextBlobPinBucketSignature {
    text_blobs: TextBlobResidencySignature,
    pinned_glyph_keys: usize,
}

impl TextBlobPinBucketSignature {
    pub(crate) fn new(text_blobs: TextBlobResidencySignature, pinned_glyph_keys: usize) -> Self {
        Self {
            text_blobs,
            pinned_glyph_keys,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextBlobPinBucketReuse {
    pub(crate) scene_text_blobs: usize,
    pub(crate) pinned_glyph_keys: usize,
}

pub(crate) struct TextBlobPinSnapshot {
    pub(crate) scene_text_blobs: usize,
    pub(crate) pinned_glyph_keys: usize,
    pub(crate) signature: Option<TextBlobResidencySignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextBlobResidencySignature {
    entries: Vec<TextResidencyEntryKey>,
}

impl TextBlobResidencySignature {
    fn empty() -> Self {
        Self { entries: vec![] }
    }

    fn push(&mut self, key: TextResidencyEntryKey) {
        self.entries.push(key);
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct TextResidencyEntryKey {
    text_blob: TextBlobId,
    glyph_fingerprint: u64,
    glyphs: u32,
}

#[derive(Debug)]
pub(crate) struct TextFrameResidency {
    entries: Vec<TextResidencyEntry>,
    all_entries_live: bool,
}

impl TextFrameResidency {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            all_entries_live: true,
        }
    }

    pub(super) fn push_glyphs(
        &mut self,
        text_blob: TextBlobId,
        glyphs: impl IntoIterator<Item = GlyphKey>,
    ) {
        let glyphs: Vec<GlyphKey> = glyphs.into_iter().collect();
        if glyphs.is_empty() {
            return;
        }

        let mut hasher = DefaultHasher::new();
        text_blob.data().as_ffi().hash(&mut hasher);
        glyphs.len().hash(&mut hasher);
        for glyph in &glyphs {
            glyph.hash(&mut hasher);
        }

        let glyphs_len = glyphs.len().min(u32::MAX as usize) as u32;
        let key = TextResidencyEntryKey {
            text_blob,
            glyph_fingerprint: hasher.finish(),
            glyphs: glyphs_len,
        };
        let pin_keys = GlyphPinKeys::from_keys(glyphs.iter().copied());
        self.entries.push(TextResidencyEntry {
            key,
            glyphs,
            pin_keys,
        });
    }

    pub(super) fn note_missing_entry(&mut self) {
        self.all_entries_live = false;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn entries(&self) -> &[TextResidencyEntry] {
        &self.entries
    }

    pub(super) fn signature(&self) -> Option<TextBlobResidencySignature> {
        if !self.all_entries_live {
            return None;
        }
        let mut signature = TextBlobResidencySignature::empty();
        for entry in &self.entries {
            signature.push(entry.key);
        }
        Some(signature)
    }

    #[cfg(test)]
    pub(crate) fn text_blob_ids(&self) -> Vec<TextBlobId> {
        self.entries
            .iter()
            .map(|entry| entry.key.text_blob)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn glyph_count(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.glyphs.len())
            .sum::<usize>()
    }
}

impl Default for TextFrameResidency {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(super) struct TextResidencyEntry {
    key: TextResidencyEntryKey,
    glyphs: Vec<GlyphKey>,
    pin_keys: GlyphPinKeys,
}

impl TextResidencyEntry {
    pub(super) fn key(&self) -> TextResidencyEntryKey {
        self.key
    }

    pub(super) fn text_blob(&self) -> TextBlobId {
        self.key.text_blob
    }

    pub(super) fn glyphs(&self) -> &[GlyphKey] {
        &self.glyphs
    }
}

#[derive(Default)]
struct TextBlobPinKeyCache {
    blob_entries: FxHashMap<TextResidencyEntryKey, TextBlobPinEntry>,
    current_counts: FxHashMap<TextResidencyEntryKey, u32>,
    current_blobs: Vec<TextResidencyEntryKey>,
    stale_blobs: Vec<TextResidencyEntryKey>,
    mask_ref_counts: FxHashMap<GlyphKey, u32>,
    color_ref_counts: FxHashMap<GlyphKey, u32>,
    subpixel_ref_counts: FxHashMap<GlyphKey, u32>,
}

impl TextBlobPinKeyCache {
    fn clear(&mut self) {
        self.blob_entries.clear();
        self.current_counts.clear();
        self.current_blobs.clear();
        self.stale_blobs.clear();
        self.mask_ref_counts.clear();
        self.color_ref_counts.clear();
        self.subpixel_ref_counts.clear();
    }

    fn collect_snapshot(&mut self, residency: &TextFrameResidency) -> TextBlobPinSnapshot {
        self.current_counts.clear();
        let signature = residency.signature();
        for entry in residency.entries() {
            let count = self.current_counts.entry(entry.key()).or_insert(0);
            *count = count.saturating_add(1);
        }

        self.reconcile(residency);
        TextBlobPinSnapshot {
            scene_text_blobs: residency.entries().len(),
            pinned_glyph_keys: self.current_pin_key_count(),
            signature,
        }
    }

    fn reconcile(&mut self, residency: &TextFrameResidency) {
        self.stale_blobs.clear();
        for &key in self.blob_entries.keys() {
            if !self.current_counts.contains_key(&key) {
                self.stale_blobs.push(key);
            }
        }

        let mut stale_blobs = std::mem::take(&mut self.stale_blobs);
        for key in stale_blobs.drain(..) {
            if let Some(entry) = self.blob_entries.remove(&key) {
                self.dec_pin_keys(&entry.pin_keys);
            }
        }
        self.stale_blobs = stale_blobs;

        self.current_blobs.clear();
        self.current_blobs
            .extend(self.current_counts.keys().copied());
        let mut current_blobs = std::mem::take(&mut self.current_blobs);
        for key in current_blobs.drain(..) {
            if self.blob_entries.contains_key(&key) {
                continue;
            }

            let Some(entry) = residency.entries().iter().find(|entry| entry.key() == key) else {
                continue;
            };
            let pin_keys = entry.pin_keys.clone();
            self.inc_pin_keys(&pin_keys);
            self.blob_entries.insert(key, TextBlobPinEntry { pin_keys });
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

struct TextBlobPinEntry {
    pin_keys: GlyphPinKeys,
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
