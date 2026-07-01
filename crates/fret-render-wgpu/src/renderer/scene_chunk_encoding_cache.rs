use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default)]
pub(super) struct SceneChunkEncodingState {
    cached_keys: Vec<SceneChunkEncodingKey>,
    previous_counts: HashMap<SceneChunkEncodingKey, u32>,
    next_keys: Vec<SceneChunkEncodingKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneChunkEncodingContext {
    pub(super) format: wgpu::TextureFormat,
    pub(super) viewport_size: (u32, u32),
    pub(super) scale_factor_bits: u32,
    pub(super) render_targets_generation: u64,
    pub(super) images_generation: u64,
    pub(super) text_atlas_revision: u64,
    pub(super) text_quality_key: u64,
    pub(super) materials_generation: u64,
    pub(super) material_paint_budget_per_frame: u64,
    pub(super) material_distinct_budget_per_frame: usize,
    pub(super) custom_effects_generation: u64,
}

impl SceneChunkEncodingContext {
    fn fingerprint(self) -> u64 {
        hash_value(self)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct SceneChunkEncodingFrameStats {
    pub(super) entries: u64,
    pub(super) key_cache_hits: u64,
    pub(super) key_cache_misses: u64,
    pub(super) key_cache_stale_entries: u64,
    pub(super) key_cache_context_fingerprint: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SceneChunkEncodingKey {
    context: SceneChunkEncodingContext,
    entry_fingerprint: u64,
    chunk_fingerprint: u64,
    chunk_ops_len: usize,
}

impl SceneChunkEncodingKey {
    fn new(context: SceneChunkEncodingContext, entry: &fret_core::SceneChunkManifestEntry) -> Self {
        Self {
            context,
            entry_fingerprint: entry.fingerprint(),
            chunk_fingerprint: entry.chunk().fingerprint(),
            chunk_ops_len: entry.chunk().ops_len(),
        }
    }
}

impl SceneChunkEncodingState {
    pub(super) fn begin_frame(
        &mut self,
        manifest: Option<&fret_core::SceneChunkManifest>,
        context: SceneChunkEncodingContext,
    ) -> SceneChunkEncodingFrameStats {
        self.previous_counts.clear();
        for key in &self.cached_keys {
            let count = self.previous_counts.entry(*key).or_default();
            *count = count.saturating_add(1);
        }

        self.next_keys.clear();

        let mut stats = SceneChunkEncodingFrameStats::default();
        if let Some(manifest) = manifest {
            stats.entries = manifest.len() as u64;
            if !manifest.is_empty() {
                stats.key_cache_context_fingerprint = context.fingerprint();
            }

            self.next_keys.reserve(manifest.len());
            for entry in manifest.entries() {
                let key = SceneChunkEncodingKey::new(context, entry);
                if let Some(count) = self.previous_counts.get_mut(&key) {
                    if *count > 0 {
                        *count -= 1;
                        stats.key_cache_hits = stats.key_cache_hits.saturating_add(1);
                    } else {
                        stats.key_cache_misses = stats.key_cache_misses.saturating_add(1);
                    }
                } else {
                    stats.key_cache_misses = stats.key_cache_misses.saturating_add(1);
                }
                self.next_keys.push(key);
            }
        }

        stats.key_cache_stale_entries = self
            .previous_counts
            .values()
            .fold(0u64, |total, count| total.saturating_add(u64::from(*count)));

        std::mem::swap(&mut self.cached_keys, &mut self.next_keys);
        self.next_keys.clear();
        stats
    }
}

impl Renderer {
    pub(super) fn build_scene_chunk_encoding_context(
        &self,
        format: wgpu::TextureFormat,
        viewport_size: (u32, u32),
        scale_factor: f32,
        text_atlas_revision: u64,
    ) -> SceneChunkEncodingContext {
        let (render_targets_generation, images_generation) = self.gpu_resources.generations();
        SceneChunkEncodingContext {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            render_targets_generation,
            images_generation,
            text_atlas_revision,
            text_quality_key: self.text_system.text_quality_key(),
            materials_generation: self.material_effect_state.materials_generation,
            material_paint_budget_per_frame: self
                .material_effect_state
                .material_paint_budget_per_frame,
            material_distinct_budget_per_frame: self
                .material_effect_state
                .material_distinct_budget_per_frame,
            custom_effects_generation: self.material_effect_state.custom_effects_generation,
        }
    }

    pub(super) fn record_scene_chunk_encoding_key_cache_for_frame(
        &mut self,
        scene_chunks: Option<&fret_core::SceneChunkManifest>,
        context: SceneChunkEncodingContext,
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        let stats = self
            .scene_chunk_encoding_state
            .begin_frame(scene_chunks, context);
        if perf_enabled {
            frame_perf.scene_chunk_encoding_key_cache_entries = stats.entries;
            frame_perf.scene_chunk_encoding_key_cache_hits = stats.key_cache_hits;
            frame_perf.scene_chunk_encoding_key_cache_misses = stats.key_cache_misses;
            frame_perf.scene_chunk_encoding_key_cache_stale_entries = stats.key_cache_stale_entries;
            frame_perf.scene_chunk_encoding_key_cache_context_fingerprint =
                stats.key_cache_context_fingerprint;
        }
    }
}

fn hash_value<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Rect, SceneChunk, SceneChunkManifest, SceneChunkManifestEntry, Size};
    use std::sync::Arc;

    fn context(text_atlas_revision: u64) -> SceneChunkEncodingContext {
        SceneChunkEncodingContext {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            viewport_size: (320, 200),
            scale_factor_bits: 1.0f32.to_bits(),
            render_targets_generation: 1,
            images_generation: 2,
            text_atlas_revision,
            text_quality_key: 3,
            materials_generation: 4,
            material_paint_budget_per_frame: 5,
            material_distinct_budget_per_frame: 6,
            custom_effects_generation: 7,
        }
    }

    fn entry(origin_x: f32) -> SceneChunkManifestEntry {
        let chunk = SceneChunk::from_ops(Arc::from([fret_core::SceneOp::PushLayer { layer: 1 }]));
        SceneChunkManifestEntry::new(
            chunk,
            Rect::new(
                Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            Point::new(fret_core::Px(origin_x), fret_core::Px(0.0)),
        )
    }

    fn manifest(entries: &[SceneChunkManifestEntry]) -> SceneChunkManifest {
        let mut manifest = SceneChunkManifest::default();
        for entry in entries {
            manifest.push(entry.clone());
        }
        manifest
    }

    #[test]
    fn chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots() {
        let mut state = SceneChunkEncodingState::default();
        let first = entry(0.0);
        let second = entry(20.0);
        let frame = manifest(&[first.clone(), second.clone()]);

        let stats = state.begin_frame(Some(&frame), context(1));
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.key_cache_hits, 0);
        assert_eq!(stats.key_cache_misses, 2);
        assert_eq!(stats.key_cache_stale_entries, 0);
        assert_ne!(stats.key_cache_context_fingerprint, 0);

        let stats = state.begin_frame(Some(&frame), context(1));
        assert_eq!(stats.key_cache_hits, 2);
        assert_eq!(stats.key_cache_misses, 0);
        assert_eq!(stats.key_cache_stale_entries, 0);

        let moved = entry(40.0);
        let frame = manifest(&[first, moved]);
        let stats = state.begin_frame(Some(&frame), context(1));
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 1);
        assert_eq!(stats.key_cache_stale_entries, 1);
    }

    #[test]
    fn chunk_encoding_key_cache_accounts_for_duplicate_entries_by_slot() {
        let mut state = SceneChunkEncodingState::default();
        let entry = entry(0.0);
        let one = manifest(std::slice::from_ref(&entry));
        let duplicated = manifest(&[entry.clone(), entry.clone()]);

        let stats = state.begin_frame(Some(&one), context(1));
        assert_eq!(stats.key_cache_misses, 1);

        let stats = state.begin_frame(Some(&duplicated), context(1));
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 1);
        assert_eq!(stats.key_cache_stale_entries, 0);

        let stats = state.begin_frame(Some(&one), context(1));
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 0);
        assert_eq!(stats.key_cache_stale_entries, 1);
    }

    #[test]
    fn chunk_encoding_key_cache_context_changes_invalidate_entries() {
        let mut state = SceneChunkEncodingState::default();
        let entry = entry(0.0);
        let frame = manifest(&[entry]);

        state.begin_frame(Some(&frame), context(1));
        let stats = state.begin_frame(Some(&frame), context(2));
        assert_eq!(stats.key_cache_hits, 0);
        assert_eq!(stats.key_cache_misses, 1);
        assert_eq!(stats.key_cache_stale_entries, 1);
    }
}
