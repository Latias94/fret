use super::*;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub(super) struct SceneChunkEncodingState {
    cached_keys: Vec<SceneChunkEncodingKey>,
    previous_counts: HashMap<SceneChunkEncodingKey, u32>,
    next_keys: Vec<SceneChunkEncodingKey>,
    payloads: HashMap<SceneChunkEncodingKey, CachedSceneChunkEncoding>,
    live_payload_keys: HashSet<SceneChunkEncodingKey>,
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
    pub(super) payload_cache_hits: u64,
    pub(super) payload_cache_misses: u64,
    pub(super) payload_chunks_encoded: u64,
    pub(super) payload_bytes_estimate: u64,
    pub(super) payload_entries_live: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SceneChunkEncodingKey {
    context: SceneChunkEncodingContext,
    entry_fingerprint: u64,
    chunk_fingerprint: u64,
    chunk_ops_len: usize,
}

#[derive(Default)]
pub(super) struct CachedSceneChunkEncoding {
    encoding: SceneEncoding,
}

impl CachedSceneChunkEncoding {
    pub(super) fn new(encoding: SceneEncoding) -> Self {
        Self { encoding }
    }

    fn estimated_bytes(&self) -> u64 {
        estimate_slice_bytes(&self.encoding.instances)
            .saturating_add(estimate_slice_bytes(&self.encoding.path_paints))
            .saturating_add(estimate_slice_bytes(&self.encoding.text_paints))
            .saturating_add(estimate_slice_bytes(&self.encoding.viewport_vertices))
            .saturating_add(estimate_slice_bytes(&self.encoding.text_glyph_instances))
            .saturating_add(estimate_slice_bytes(&self.encoding.text_vertices))
            .saturating_add(estimate_slice_bytes(&self.encoding.path_vertices))
            .saturating_add(estimate_slice_bytes(&self.encoding.clip_path_masks))
            .saturating_add(estimate_slice_bytes(&self.encoding.clips))
            .saturating_add(estimate_slice_bytes(&self.encoding.masks))
            .saturating_add(estimate_slice_bytes(&self.encoding.uniforms))
            .saturating_add(estimate_slice_bytes(&self.encoding.uniform_mask_images))
            .saturating_add(estimate_slice_bytes(&self.encoding.ordered_draws))
            .saturating_add(estimate_slice_bytes(&self.encoding.effect_markers))
    }
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

    pub(super) fn begin_frame_with_payloads(
        &mut self,
        manifest: Option<&fret_core::SceneChunkManifest>,
        context: SceneChunkEncodingContext,
        mut build_payload: impl FnMut(&fret_core::SceneChunkManifestEntry) -> CachedSceneChunkEncoding,
    ) -> SceneChunkEncodingFrameStats {
        self.previous_counts.clear();
        for key in &self.cached_keys {
            let count = self.previous_counts.entry(*key).or_default();
            *count = count.saturating_add(1);
        }

        self.next_keys.clear();
        self.live_payload_keys.clear();

        let mut stats = SceneChunkEncodingFrameStats::default();
        if let Some(manifest) = manifest {
            stats.entries = manifest.len() as u64;
            if !manifest.is_empty() {
                stats.key_cache_context_fingerprint = context.fingerprint();
            }

            self.next_keys.reserve(manifest.len());
            self.live_payload_keys.reserve(manifest.len());
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

                if self.payloads.contains_key(&key) {
                    stats.payload_cache_hits = stats.payload_cache_hits.saturating_add(1);
                } else {
                    let payload = build_payload(entry);
                    self.payloads.insert(key, payload);
                    stats.payload_cache_misses = stats.payload_cache_misses.saturating_add(1);
                    stats.payload_chunks_encoded = stats.payload_chunks_encoded.saturating_add(1);
                }

                if let Some(payload) = self.payloads.get(&key) {
                    stats.payload_bytes_estimate = stats
                        .payload_bytes_estimate
                        .saturating_add(payload.estimated_bytes());
                }

                self.live_payload_keys.insert(key);
                self.next_keys.push(key);
            }
        }

        stats.key_cache_stale_entries = self
            .previous_counts
            .values()
            .fold(0u64, |total, count| total.saturating_add(u64::from(*count)));

        self.payloads
            .retain(|key, _| self.live_payload_keys.contains(key));
        stats.payload_entries_live = self.payloads.len() as u64;

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
        scale_factor: f32,
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        let mut state = std::mem::take(&mut self.scene_chunk_encoding_state);
        let stats = if perf_enabled {
            let viewport_size = context.viewport_size;
            let output_is_srgb = context.format.is_srgb();
            state.begin_frame_with_payloads(scene_chunks, context, |entry| {
                self.encode_scene_chunk_entry_payload(
                    entry,
                    scale_factor,
                    viewport_size,
                    output_is_srgb,
                )
            })
        } else {
            state.begin_frame(scene_chunks, context)
        };
        self.scene_chunk_encoding_state = state;
        if perf_enabled {
            frame_perf.scene_chunk_encoding_key_cache_entries = stats.entries;
            frame_perf.scene_chunk_encoding_key_cache_hits = stats.key_cache_hits;
            frame_perf.scene_chunk_encoding_key_cache_misses = stats.key_cache_misses;
            frame_perf.scene_chunk_encoding_key_cache_stale_entries = stats.key_cache_stale_entries;
            frame_perf.scene_chunk_encoding_key_cache_context_fingerprint =
                stats.key_cache_context_fingerprint;
            frame_perf.scene_chunk_encoding_payload_cache_hits = stats.payload_cache_hits;
            frame_perf.scene_chunk_encoding_payload_cache_misses = stats.payload_cache_misses;
            frame_perf.scene_chunk_encoding_payload_chunks_encoded = stats.payload_chunks_encoded;
            frame_perf.scene_chunk_encoding_payload_bytes_estimate = stats.payload_bytes_estimate;
            frame_perf.scene_chunk_encoding_payload_entries_live = stats.payload_entries_live;
        }
    }

    fn encode_scene_chunk_entry_payload(
        &mut self,
        entry: &fret_core::SceneChunkManifestEntry,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
    ) -> CachedSceneChunkEncoding {
        let mut scene = fret_core::Scene::default();
        entry
            .chunk()
            .replay_translated_into(&mut scene, entry.scene_origin());
        let mut encoding = SceneEncoding::default();
        let mut ignored_perf = RenderPerfStats::default();
        self.encode_scene_ops_into(
            &scene,
            scale_factor,
            viewport_size,
            output_is_srgb,
            &mut encoding,
            false,
            false,
            &mut ignored_perf,
        );
        CachedSceneChunkEncoding::new(encoding)
    }
}

fn hash_value<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn estimate_slice_bytes<T>(slice: &[T]) -> u64 {
    std::mem::size_of_val(slice) as u64
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

    #[test]
    fn chunk_encoding_payload_cache_builds_only_misses_and_evicts_stale_payloads() {
        let mut state = SceneChunkEncodingState::default();
        let first = entry(0.0);
        let second = entry(20.0);
        let mut builds = 0u64;

        let frame = manifest(std::slice::from_ref(&first));
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 0);
        assert_eq!(stats.payload_cache_misses, 1);
        assert_eq!(stats.payload_chunks_encoded, 1);
        assert_eq!(stats.payload_entries_live, 1);
        assert_eq!(builds, 1);

        let frame = manifest(&[first.clone(), second]);
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 1);
        assert_eq!(stats.payload_cache_misses, 1);
        assert_eq!(stats.payload_chunks_encoded, 1);
        assert_eq!(stats.payload_entries_live, 2);
        assert_eq!(builds, 2);

        let frame = manifest(&[first]);
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 1);
        assert_eq!(stats.payload_cache_misses, 0);
        assert_eq!(stats.payload_chunks_encoded, 0);
        assert_eq!(stats.payload_entries_live, 1);
        assert_eq!(builds, 2);
    }
}
