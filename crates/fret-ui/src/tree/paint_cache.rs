use super::*;
use fret_core::TextBlobId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PaintCacheKey {
    width_bits: u32,
    height_bits: u32,
    geometry_fingerprint: u64,
    scale_factor_bits: u32,
    theme_revision: u64,
    fg_present: bool,
    fg_r_bits: u32,
    fg_g_bits: u32,
    fg_b_bits: u32,
    fg_a_bits: u32,
    text_style_present: bool,
    text_style_fingerprint: u64,
    child_a_bits: u32,
    child_b_bits: u32,
    child_c_bits: u32,
    child_d_bits: u32,
    child_tx_bits: u32,
    child_ty_bits: u32,
}

impl PaintCacheKey {
    pub(super) fn new(
        bounds: Rect,
        geometry_fingerprint: u64,
        scale_factor: f32,
        theme_revision: u64,
        paint_style: crate::tree::paint_style::PaintStyleState,
        inherited_text_style_fingerprint: Option<u64>,
        child_transform: Transform2D,
    ) -> Self {
        let (fg_present, fg_r_bits, fg_g_bits, fg_b_bits, fg_a_bits) =
            if let Some(fg) = paint_style.foreground {
                (
                    true,
                    fg.r.to_bits(),
                    fg.g.to_bits(),
                    fg.b.to_bits(),
                    fg.a.to_bits(),
                )
            } else {
                (false, 0, 0, 0, 0)
            };

        Self {
            width_bits: bounds.size.width.0.to_bits(),
            height_bits: bounds.size.height.0.to_bits(),
            geometry_fingerprint,
            scale_factor_bits: scale_factor.to_bits(),
            theme_revision,
            fg_present,
            fg_r_bits,
            fg_g_bits,
            fg_b_bits,
            fg_a_bits,
            text_style_present: inherited_text_style_fingerprint.is_some(),
            text_style_fingerprint: inherited_text_style_fingerprint.unwrap_or(0),
            child_a_bits: child_transform.a.to_bits(),
            child_b_bits: child_transform.b.to_bits(),
            child_c_bits: child_transform.c.to_bits(),
            child_d_bits: child_transform.d.to_bits(),
            child_tx_bits: child_transform.tx.to_bits(),
            child_ty_bits: child_transform.ty.to_bits(),
        }
    }
}

fn mix_geometry_hash(hash: &mut u64, value: u64) {
    *hash ^= value
        .wrapping_add(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(*hash << 6)
        .wrapping_add(*hash >> 2);
}

fn px_hash_bits(value: Px) -> u64 {
    value.0.to_bits() as u64
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn recompute_paint_geometry_fingerprint(&mut self, node: NodeId) -> u64 {
        let Some(entry) = self.nodes.get(node) else {
            return 0;
        };
        let bounds = entry.bounds;
        let measured_size = entry.measured_size;
        let mut children = SmallNodeList::<16>::default();
        children.set(entry.children.as_slice());

        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        mix_geometry_hash(&mut hash, px_hash_bits(bounds.size.width));
        mix_geometry_hash(&mut hash, px_hash_bits(bounds.size.height));
        mix_geometry_hash(&mut hash, px_hash_bits(measured_size.width));
        mix_geometry_hash(&mut hash, px_hash_bits(measured_size.height));
        mix_geometry_hash(&mut hash, children.as_slice().len() as u64);

        for &child in children.as_slice() {
            let Some(child_entry) = self.nodes.get(child) else {
                continue;
            };
            let local_x = child_entry.bounds.origin.x - bounds.origin.x;
            let local_y = child_entry.bounds.origin.y - bounds.origin.y;
            mix_geometry_hash(&mut hash, px_hash_bits(local_x));
            mix_geometry_hash(&mut hash, px_hash_bits(local_y));
            mix_geometry_hash(&mut hash, px_hash_bits(child_entry.bounds.size.width));
            mix_geometry_hash(&mut hash, px_hash_bits(child_entry.bounds.size.height));
            mix_geometry_hash(&mut hash, child_entry.paint_geometry_fingerprint);
        }

        let hash = if hash == 0 { 1 } else { hash };
        if let Some(entry) = self.nodes.get_mut(node) {
            entry.paint_geometry_fingerprint = hash;
        }
        hash
    }

    #[cfg(test)]
    pub(crate) fn test_recompute_paint_geometry_fingerprint_subtree(
        &mut self,
        node: NodeId,
    ) -> u64 {
        let mut children = SmallNodeList::<16>::default();
        if let Some(entry) = self.nodes.get(node) {
            children.set(entry.children.as_slice());
        }
        for &child in children.as_slice() {
            self.test_recompute_paint_geometry_fingerprint_subtree(child);
        }
        self.recompute_paint_geometry_fingerprint(node)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PaintCacheEntry {
    pub(super) generation: u64,
    pub(super) key: PaintCacheKey,
    pub(super) origin: Point,
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) text_blob_start: u32,
    pub(super) text_blob_end: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PaintCachePolicy {
    /// Enable caching unless the UI is in an inspection/probe mode (e.g. picking, semantics).
    #[default]
    Auto,
    /// Always enable caching.
    Enabled,
    /// Always disable caching.
    Disabled,
}

#[derive(Debug, Default)]
pub(super) struct PaintCacheState {
    pub(super) generation: u64,
    previous_frame: PreviousFramePaintRecording,
    pub(super) source_generation: u64,
    pub(super) target_generation: u64,
    pub(super) hits: u32,
    pub(super) misses: u32,
    pub(super) replayed_ops: u32,
}

#[derive(Debug, Default)]
pub(super) struct PreviousFramePaintRecording {
    ops: Vec<SceneOp>,
    text_blob_ids: Vec<TextBlobId>,
    fingerprint: u64,
}

impl PreviousFramePaintRecording {
    #[cfg(test)]
    pub(super) fn ops_len(&self) -> usize {
        self.ops.len()
    }

    pub(super) fn ingest_scene(&mut self, scene: &mut Scene) {
        scene.swap_storage(
            &mut self.ops,
            &mut self.text_blob_ids,
            &mut self.fingerprint,
        );
    }

    pub(super) fn clear(&mut self) {
        self.ops.clear();
        self.text_blob_ids.clear();
        self.fingerprint = 0;
    }

    pub(super) fn is_entry_replayable(&self, entry: PaintCacheEntry) -> bool {
        self.entry_ranges(entry).is_some()
    }

    pub(super) fn replay_entry_translated(
        &self,
        scene: &mut Scene,
        entry: PaintCacheEntry,
        delta: Point,
    ) -> Option<usize> {
        let ranges = self.entry_ranges(entry)?;
        let start = scene.ops_len();
        scene.replay_ops_translated_with_text_blob_ids(
            &self.ops[ranges.ops],
            delta,
            &self.text_blob_ids[ranges.text_blobs],
        );
        Some(scene.ops_len().saturating_sub(start))
    }

    fn entry_ranges(&self, entry: PaintCacheEntry) -> Option<PreviousFramePaintRecordingRanges> {
        let op_start = entry.start as usize;
        let op_end = entry.end as usize;
        if op_start > op_end || op_end > self.ops.len() {
            return None;
        }

        let text_blob_start = entry.text_blob_start as usize;
        let text_blob_end = entry.text_blob_end as usize;
        if text_blob_start > text_blob_end || text_blob_end > self.text_blob_ids.len() {
            return None;
        }

        Some(PreviousFramePaintRecordingRanges {
            ops: op_start..op_end,
            text_blobs: text_blob_start..text_blob_end,
        })
    }
}

struct PreviousFramePaintRecordingRanges {
    ops: std::ops::Range<usize>,
    text_blobs: std::ops::Range<usize>,
}

impl PaintCacheState {
    pub(super) fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_ops = 0;
    }

    pub(super) fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    pub(super) fn invalidate_recording(&mut self) {
        self.previous_frame.clear();
        self.generation = self.generation.saturating_add(1);
    }

    pub(super) fn ingest_previous_frame_scene(&mut self, scene: &mut Scene) {
        self.previous_frame.ingest_scene(scene);
    }

    pub(super) fn is_entry_replayable_in_previous_frame(&self, entry: PaintCacheEntry) -> bool {
        self.previous_frame.is_entry_replayable(entry)
    }

    pub(super) fn replay_previous_frame_entry_translated(
        &self,
        scene: &mut Scene,
        entry: PaintCacheEntry,
        delta: Point,
    ) -> Option<usize> {
        self.previous_frame
            .replay_entry_translated(scene, entry, delta)
    }

    pub(super) fn rebase_entry_from_replayed_parent(
        &self,
        parent_previous: PaintCacheEntry,
        parent_current_start: u32,
        parent_current_text_blob_start: u32,
        descendant_previous: PaintCacheEntry,
    ) -> Option<PaintCacheEntry> {
        if descendant_previous.generation != self.source_generation {
            return None;
        }

        let parent_ranges = self.previous_frame.entry_ranges(parent_previous)?;
        let descendant_ranges = self.previous_frame.entry_ranges(descendant_previous)?;
        if descendant_ranges.ops.start < parent_ranges.ops.start
            || descendant_ranges.ops.end > parent_ranges.ops.end
            || descendant_ranges.text_blobs.start < parent_ranges.text_blobs.start
            || descendant_ranges.text_blobs.end > parent_ranges.text_blobs.end
        {
            return None;
        }

        let op_start_offset =
            u32::try_from(descendant_ranges.ops.start - parent_ranges.ops.start).ok()?;
        let op_end_offset =
            u32::try_from(descendant_ranges.ops.end - parent_ranges.ops.start).ok()?;
        let text_blob_start_offset =
            u32::try_from(descendant_ranges.text_blobs.start - parent_ranges.text_blobs.start)
                .ok()?;
        let text_blob_end_offset =
            u32::try_from(descendant_ranges.text_blobs.end - parent_ranges.text_blobs.start)
                .ok()?;

        Some(PaintCacheEntry {
            generation: self.target_generation,
            key: descendant_previous.key,
            origin: descendant_previous.origin,
            start: parent_current_start.checked_add(op_start_offset)?,
            end: parent_current_start.checked_add(op_end_offset)?,
            text_blob_start: parent_current_text_blob_start.checked_add(text_blob_start_offset)?,
            text_blob_end: parent_current_text_blob_start.checked_add(text_blob_end_offset)?,
        })
    }

    #[cfg(test)]
    pub(super) fn retained_recording_ops_len(&self) -> usize {
        self.previous_frame.ops_len()
    }
}
