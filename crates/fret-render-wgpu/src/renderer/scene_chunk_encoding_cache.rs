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
    pub(super) payload_plan_candidate_segments: u64,
    pub(super) payload_plan_shape_matches: u64,
    pub(super) payload_plan_shape_mismatches: u64,
    pub(super) payload_plan_stream_fingerprint_matches: u64,
    pub(super) payload_plan_stream_fingerprint_mismatches: u64,
    pub(super) payload_reassembly_dry_run_candidates: u64,
    pub(super) payload_reassembly_append_only_matches: u64,
    pub(super) payload_reassembly_blocked_by_shape_mismatch: u64,
    pub(super) payload_reassembly_blocked_by_stream_fingerprint_mismatch: u64,
    pub(super) payload_reassembly_blocked_by_non_quad_draws: u64,
    pub(super) payload_reassembly_blocked_by_side_tables: u64,
    pub(super) payload_reassembly_blocked_by_material_state: u64,
    pub(super) payload_entries_without_plan_candidate: u64,
    pub(super) payload_plan_candidates_without_payload: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct SceneChunkPayloadReassemblyPlan {
    safe_segment_indices: Vec<usize>,
}

impl SceneChunkPayloadReassemblyPlan {
    #[cfg(test)]
    pub(super) fn from_safe_segment_indices(indices: Vec<usize>) -> Self {
        Self {
            safe_segment_indices: indices,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.safe_segment_indices.is_empty()
    }

    pub(super) fn safe_segment_indices(&self) -> &[usize] {
        &self.safe_segment_indices
    }

    fn push_safe_segment_index(&mut self, index: usize) {
        self.safe_segment_indices.push(index);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct SceneChunkPayloadPlanAlignment {
    pub(super) stats: SceneChunkEncodingFrameStats,
    pub(super) reassembly_plan: SceneChunkPayloadReassemblyPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SceneChunkEncodingKey {
    context: SceneChunkEncodingContext,
    entry_fingerprint: u64,
    chunk_fingerprint: u64,
    chunk_ops_len: usize,
    chunk_text_resource_key: u64,
}

#[derive(Default)]
pub(super) struct CachedSceneChunkEncoding {
    encoding: SceneEncoding,
    plan_shape: SceneChunkPayloadPlanShape,
    stream_fingerprint: u64,
}

impl CachedSceneChunkEncoding {
    pub(super) fn new(encoding: SceneEncoding) -> Self {
        let plan_shape = SceneChunkPayloadPlanShape::from_ordered_draws(&encoding.ordered_draws);
        let stream_fingerprint =
            SceneChunkPayloadStreamFingerprint::from_payload_encoding(&encoding).fingerprint;
        Self {
            encoding,
            plan_shape,
            stream_fingerprint,
        }
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

    fn append_only_reassembly_blocker(&self) -> Option<SceneChunkPayloadReassemblyBlocker> {
        if self.encoding.material_quad_ops > 0
            || self.encoding.material_sampled_quad_ops > 0
            || self.encoding.material_distinct > 0
            || self.encoding.material_unknown_ids > 0
            || self.encoding.material_degraded_due_to_budget > 0
            || self.encoding.path_material_paints_degraded_to_solid_base > 0
        {
            return Some(SceneChunkPayloadReassemblyBlocker::MaterialState);
        }

        if !self.encoding.clip_path_masks.is_empty()
            || !self.encoding.clips.is_empty()
            || !self.encoding.masks.is_empty()
            || self
                .encoding
                .uniform_mask_images
                .iter()
                .any(Option::is_some)
            || !self.encoding.effect_markers.is_empty()
        {
            return Some(SceneChunkPayloadReassemblyBlocker::SideTables);
        }

        if self
            .encoding
            .ordered_draws
            .iter()
            .any(|draw| !matches!(draw, OrderedDraw::Quad(_) | OrderedDraw::VertexColor(_)))
        {
            return Some(SceneChunkPayloadReassemblyBlocker::NonQuadDraws);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneChunkPayloadReassemblyBlocker {
    ShapeMismatch,
    StreamFingerprintMismatch,
    NonQuadDraws,
    SideTables,
    MaterialState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SceneChunkPayloadStreamFingerprint {
    fingerprint: u64,
}

impl SceneChunkPayloadStreamFingerprint {
    fn from_payload_encoding(encoding: &SceneEncoding) -> Self {
        let mut hasher = DefaultHasher::new();
        hash_pod_slice(&mut hasher, 0, &encoding.instances);
        hash_pod_slice(&mut hasher, 1, &encoding.path_paints);
        hash_pod_slice(&mut hasher, 2, &encoding.text_paints);
        hash_pod_slice(&mut hasher, 3, &encoding.viewport_vertices);
        hash_pod_slice(&mut hasher, 4, &encoding.text_glyph_instances);
        hash_pod_slice(&mut hasher, 5, &encoding.text_vertices);
        hash_pod_slice(&mut hasher, 6, &encoding.path_vertices);
        Self {
            fingerprint: hasher.finish(),
        }
    }

    fn from_flat_encoding_segment(
        encoding: &SceneEncoding,
        ranges: RenderPlanSegmentStreamRanges,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        hash_pod_range(&mut hasher, 0, &encoding.instances, ranges.quad_instances);
        hash_pod_range(&mut hasher, 1, &encoding.path_paints, ranges.path_paints);
        hash_pod_range(&mut hasher, 2, &encoding.text_paints, ranges.text_paints);
        hash_pod_range(
            &mut hasher,
            3,
            &encoding.viewport_vertices,
            ranges.viewport_vertices,
        );
        hash_pod_range(
            &mut hasher,
            4,
            &encoding.text_glyph_instances,
            ranges.text_glyph_instances,
        );
        hash_pod_range(
            &mut hasher,
            5,
            &encoding.text_vertices,
            ranges.text_vertices,
        );
        hash_pod_range(
            &mut hasher,
            6,
            &encoding.path_vertices,
            ranges.path_vertices,
        );
        Self {
            fingerprint: hasher.finish(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SceneChunkPayloadPlanShape {
    draw_count: u32,
    flags_mask: u8,
    stream_shape: RenderPlanSegmentStreamShape,
}

impl SceneChunkPayloadPlanShape {
    fn from_ordered_draws(draws: &[OrderedDraw]) -> Self {
        Self {
            draw_count: u32::try_from(draws.len()).unwrap_or(u32::MAX),
            flags_mask: RenderPlanSegmentFlags::for_ordered_draws(draws).diagnostics_mask(),
            stream_shape: RenderPlanSegmentStreamRanges::for_ordered_draws(draws).shape(),
        }
    }

    fn from_segment(segment: &RenderPlanSegment) -> Self {
        Self {
            draw_count: segment.scene_chunk_candidate.draw_count,
            flags_mask: segment.flags.diagnostics_mask(),
            stream_shape: segment.stream_ranges.shape(),
        }
    }
}

impl SceneChunkEncodingKey {
    fn new(
        context: SceneChunkEncodingContext,
        entry: &fret_core::SceneChunkManifestEntry,
        chunk_text_resource_key: u64,
    ) -> Self {
        Self {
            context,
            entry_fingerprint: entry.fingerprint(),
            chunk_fingerprint: entry.chunk().fingerprint(),
            chunk_ops_len: entry.chunk().ops_len(),
            chunk_text_resource_key,
        }
    }
}

impl SceneChunkEncodingState {
    pub(super) fn assemble_resource_free_quad_frame_encoding(
        &self,
        manifest: &fret_core::SceneChunkManifest,
        context: SceneChunkEncodingContext,
    ) -> Option<SceneEncoding> {
        let mut encoding = SceneEncoding::default();
        for entry in manifest.entries() {
            if !entry.chunk().closure().is_resource_free_quad_only() {
                return None;
            }
            let key = SceneChunkEncodingKey::new(context, entry, 0);
            let payload = self.payloads.get(&key)?;
            if payload.append_only_reassembly_blocker().is_some() {
                return None;
            }
            append_quad_payload_encoding(&mut encoding, &payload.encoding)?;
        }
        Some(encoding)
    }

    pub(super) fn begin_frame_with_payloads(
        &mut self,
        manifest: Option<&fret_core::SceneChunkManifest>,
        context: SceneChunkEncodingContext,
        entry_text_resource_keys: &[u64],
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
            debug_assert_eq!(
                entry_text_resource_keys.len(),
                manifest.len(),
                "scene chunk text resource keys must match manifest entries"
            );
            stats.entries = manifest.len() as u64;
            if !manifest.is_empty() {
                stats.key_cache_context_fingerprint = context.fingerprint();
            }

            self.next_keys.reserve(manifest.len());
            self.live_payload_keys.reserve(manifest.len());
            for (index, entry) in manifest.entries().iter().enumerate() {
                let key = SceneChunkEncodingKey::new(
                    context,
                    entry,
                    entry_text_resource_keys.get(index).copied().unwrap_or(0),
                );
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

    pub(super) fn record_payload_plan_alignment(
        &self,
        plan: &RenderPlan,
        flat_encoding: &SceneEncoding,
    ) -> SceneChunkPayloadPlanAlignment {
        let mut stats = SceneChunkEncodingFrameStats::default();
        let mut reassembly_plan = SceneChunkPayloadReassemblyPlan::default();
        let mut candidates = plan
            .segments
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.scene_chunk_candidate.eligible);

        for key in &self.cached_keys {
            let Some((segment_index, segment)) = candidates.next() else {
                stats.payload_entries_without_plan_candidate = stats
                    .payload_entries_without_plan_candidate
                    .saturating_add(1);
                continue;
            };

            let Some(payload) = self.payloads.get(key) else {
                stats.payload_plan_shape_mismatches =
                    stats.payload_plan_shape_mismatches.saturating_add(1);
                continue;
            };

            let segment_shape = SceneChunkPayloadPlanShape::from_segment(segment);
            let shape_matches = payload.plan_shape == segment_shape;
            if shape_matches {
                stats.payload_plan_shape_matches =
                    stats.payload_plan_shape_matches.saturating_add(1);
            } else {
                stats.payload_plan_shape_mismatches =
                    stats.payload_plan_shape_mismatches.saturating_add(1);
            }

            let segment_stream_fingerprint =
                SceneChunkPayloadStreamFingerprint::from_flat_encoding_segment(
                    flat_encoding,
                    segment.stream_ranges,
                );
            let stream_fingerprint_matches =
                payload.stream_fingerprint == segment_stream_fingerprint.fingerprint;
            if stream_fingerprint_matches {
                stats.payload_plan_stream_fingerprint_matches = stats
                    .payload_plan_stream_fingerprint_matches
                    .saturating_add(1);
            } else {
                stats.payload_plan_stream_fingerprint_mismatches = stats
                    .payload_plan_stream_fingerprint_mismatches
                    .saturating_add(1);
            }

            stats.payload_reassembly_dry_run_candidates = stats
                .payload_reassembly_dry_run_candidates
                .saturating_add(1);
            let blocker = if !shape_matches {
                Some(SceneChunkPayloadReassemblyBlocker::ShapeMismatch)
            } else if !stream_fingerprint_matches {
                Some(SceneChunkPayloadReassemblyBlocker::StreamFingerprintMismatch)
            } else {
                payload.append_only_reassembly_blocker()
            };
            match blocker {
                None => {
                    stats.payload_reassembly_append_only_matches = stats
                        .payload_reassembly_append_only_matches
                        .saturating_add(1);
                    reassembly_plan.push_safe_segment_index(segment_index);
                }
                Some(SceneChunkPayloadReassemblyBlocker::ShapeMismatch) => {
                    stats.payload_reassembly_blocked_by_shape_mismatch = stats
                        .payload_reassembly_blocked_by_shape_mismatch
                        .saturating_add(1);
                }
                Some(SceneChunkPayloadReassemblyBlocker::StreamFingerprintMismatch) => {
                    stats.payload_reassembly_blocked_by_stream_fingerprint_mismatch = stats
                        .payload_reassembly_blocked_by_stream_fingerprint_mismatch
                        .saturating_add(1);
                }
                Some(SceneChunkPayloadReassemblyBlocker::NonQuadDraws) => {
                    stats.payload_reassembly_blocked_by_non_quad_draws = stats
                        .payload_reassembly_blocked_by_non_quad_draws
                        .saturating_add(1);
                }
                Some(SceneChunkPayloadReassemblyBlocker::SideTables) => {
                    stats.payload_reassembly_blocked_by_side_tables = stats
                        .payload_reassembly_blocked_by_side_tables
                        .saturating_add(1);
                }
                Some(SceneChunkPayloadReassemblyBlocker::MaterialState) => {
                    stats.payload_reassembly_blocked_by_material_state = stats
                        .payload_reassembly_blocked_by_material_state
                        .saturating_add(1);
                }
            }
        }

        let mut remaining_candidates = 0u64;
        for _ in candidates {
            remaining_candidates = remaining_candidates.saturating_add(1);
        }
        stats.payload_plan_candidates_without_payload = remaining_candidates;
        stats.payload_plan_candidate_segments = stats
            .payload_plan_shape_matches
            .saturating_add(stats.payload_plan_shape_mismatches)
            .saturating_add(stats.payload_plan_candidates_without_payload);
        SceneChunkPayloadPlanAlignment {
            stats,
            reassembly_plan,
        }
    }
}

fn append_quad_payload_encoding(dst: &mut SceneEncoding, src: &SceneEncoding) -> Option<()> {
    let instance_base = u32::try_from(dst.instances.len()).ok()?;
    let uniform_base = u32::try_from(dst.uniforms.len()).ok()?;

    dst.instances.extend_from_slice(&src.instances);
    dst.uniforms.extend_from_slice(&src.uniforms);
    dst.uniform_mask_images
        .extend_from_slice(&src.uniform_mask_images);

    if src.path_paints.is_empty()
        && src.text_paints.is_empty()
        && src.viewport_vertices.is_empty()
        && src.text_glyph_instances.is_empty()
        && src.text_vertices.is_empty()
        && src.path_vertices.is_empty()
        && src.clip_path_masks.is_empty()
        && src.clips.is_empty()
        && src.masks.is_empty()
        && src.effect_markers.is_empty()
    {
        for draw in &src.ordered_draws {
            let OrderedDraw::Quad(mut quad) = *draw else {
                return None;
            };
            quad.first_instance = quad.first_instance.checked_add(instance_base)?;
            quad.uniform_index = quad.uniform_index.checked_add(uniform_base)?;
            dst.ordered_draws.push(OrderedDraw::Quad(quad));
        }
        return Some(());
    }

    None
}

impl Renderer {
    pub(super) fn build_scene_chunk_encoding_context(
        &self,
        format: wgpu::TextureFormat,
        viewport_size: (u32, u32),
        scale_factor: f32,
    ) -> SceneChunkEncodingContext {
        let (render_targets_generation, images_generation) = self.gpu_resources.generations();
        SceneChunkEncodingContext {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            render_targets_generation,
            images_generation,
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
        let viewport_size = context.viewport_size;
        let entry_text_resource_keys = scene_chunks
            .map(|manifest| {
                manifest
                    .entries()
                    .iter()
                    .map(|entry| {
                        let residency =
                            render_scene::visible_text::visible_text_residency_for_chunk_entry(
                                entry,
                                &self.text_system,
                                scale_factor,
                                viewport_size,
                            );
                        self.text_system
                            .text_resource_snapshot_for_residency(&residency)
                            .fingerprint
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut frame_assembler = std::mem::take(&mut self.frame_assembler);
        let output_is_srgb = context.format.is_srgb();
        let stats = frame_assembler.begin_frame_with_payloads(
            scene_chunks,
            context,
            &entry_text_resource_keys,
            |entry| {
                self.encode_scene_chunk_entry_payload(
                    entry,
                    scale_factor,
                    viewport_size,
                    output_is_srgb,
                )
            },
        );
        self.frame_assembler = frame_assembler;
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

    pub(super) fn record_scene_chunk_payload_plan_alignment_for_frame(
        &mut self,
        plan: &RenderPlan,
        flat_encoding: &SceneEncoding,
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> SceneChunkPayloadPlanAlignment {
        let alignment = self
            .frame_assembler
            .record_payload_plan_alignment(plan, flat_encoding);
        if perf_enabled {
            let stats = alignment.stats;
            frame_perf.scene_chunk_encoding_payload_plan_candidate_segments =
                stats.payload_plan_candidate_segments;
            frame_perf.scene_chunk_encoding_payload_plan_shape_matches =
                stats.payload_plan_shape_matches;
            frame_perf.scene_chunk_encoding_payload_plan_shape_mismatches =
                stats.payload_plan_shape_mismatches;
            frame_perf.scene_chunk_encoding_payload_plan_stream_fingerprint_matches =
                stats.payload_plan_stream_fingerprint_matches;
            frame_perf.scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches =
                stats.payload_plan_stream_fingerprint_mismatches;
            frame_perf.scene_chunk_encoding_payload_reassembly_dry_run_candidates =
                stats.payload_reassembly_dry_run_candidates;
            frame_perf.scene_chunk_encoding_payload_reassembly_append_only_matches =
                stats.payload_reassembly_append_only_matches;
            frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_shape_mismatch =
                stats.payload_reassembly_blocked_by_shape_mismatch;
            frame_perf
                .scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch =
                stats.payload_reassembly_blocked_by_stream_fingerprint_mismatch;
            frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_non_quad_draws =
                stats.payload_reassembly_blocked_by_non_quad_draws;
            frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_side_tables =
                stats.payload_reassembly_blocked_by_side_tables;
            frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_material_state =
                stats.payload_reassembly_blocked_by_material_state;
            frame_perf.scene_chunk_encoding_payload_entries_without_plan_candidate =
                stats.payload_entries_without_plan_candidate;
            frame_perf.scene_chunk_encoding_payload_plan_candidates_without_payload =
                stats.payload_plan_candidates_without_payload;
        }
        alignment
    }

    fn encode_scene_chunk_entry_payload(
        &mut self,
        entry: &fret_core::SceneChunkManifestEntry,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
    ) -> CachedSceneChunkEncoding {
        let mut encoding = SceneEncoding::default();
        if !entry.chunk().closure().is_resource_free_quad_only()
            && !entry.chunk().closure().is_resource_free_vertex_color_only()
        {
            return CachedSceneChunkEncoding::new(encoding);
        }

        let mut ignored_perf = RenderPerfStats::default();
        self.encode_scene_op_slice_into(
            entry.chunk().ops(),
            Some(fret_core::Transform2D::translation(entry.scene_origin())),
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

fn hash_pod_slice<T: bytemuck::Pod>(hasher: &mut DefaultHasher, tag: u8, slice: &[T]) {
    tag.hash(hasher);
    u64::try_from(slice.len()).unwrap_or(u64::MAX).hash(hasher);
    true.hash(hasher);
    bytemuck::cast_slice::<T, u8>(slice).hash(hasher);
}

fn hash_pod_range<T: bytemuck::Pod>(
    hasher: &mut DefaultHasher,
    tag: u8,
    slice: &[T],
    range: RenderPlanStreamRange,
) {
    tag.hash(hasher);
    u64::from(range.len()).hash(hasher);
    let start = usize::try_from(range.start).ok();
    let end = usize::try_from(range.end).ok();
    if let (Some(start), Some(end)) = (start, end)
        && let Some(slice) = slice.get(start..end)
    {
        true.hash(hasher);
        bytemuck::cast_slice::<T, u8>(slice).hash(hasher);
        return;
    }

    false.hash(hasher);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        Color, DrawOrder, Paint, Point, Rect, Scene, SceneChunk, SceneChunkManifest,
        SceneChunkManifestEntry, SceneOp, Size, TextBlobId, TextConstraints, TextStyle,
    };
    use std::sync::Arc;

    fn context(text_quality_key: u64) -> SceneChunkEncodingContext {
        SceneChunkEncodingContext {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            viewport_size: (320, 200),
            scale_factor_bits: 1.0f32.to_bits(),
            render_targets_generation: 1,
            images_generation: 2,
            text_quality_key,
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

    fn begin_frame(
        state: &mut SceneChunkEncodingState,
        manifest: Option<&SceneChunkManifest>,
        context: SceneChunkEncodingContext,
    ) -> SceneChunkEncodingFrameStats {
        let entry_text_resource_keys = manifest
            .map(|manifest| vec![0; manifest.len()])
            .unwrap_or_default();
        state.begin_frame_with_payloads(manifest, context, &entry_text_resource_keys, |_| {
            CachedSceneChunkEncoding::new(SceneEncoding::default())
        })
    }

    fn quad_payload_encoding() -> SceneEncoding {
        let mut encoding = SceneEncoding::default();
        let instance: QuadInstance = bytemuck::Zeroable::zeroed();
        let uniform: ViewportUniform = bytemuck::Zeroable::zeroed();
        encoding.instances.push(instance);
        encoding.uniforms.push(uniform);
        encoding.uniform_mask_images.push(None);
        encoding.ordered_draws.push(OrderedDraw::Quad(QuadDraw {
            scissor: ScissorRect::full(320, 200),
            uniform_index: 0,
            first_instance: 0,
            instance_count: 1,
            pipeline: QuadPipelineKey {
                fill_kind: 0,
                border_kind: 0,
                border_present: false,
                dash_enabled: false,
                fill_material_sampled: false,
                border_material_sampled: false,
                shadow_mode: false,
            },
        }));
        encoding
    }

    #[test]
    fn resource_free_quad_payloads_assemble_frame_encoding_with_relocated_indices() {
        let mut state = SceneChunkEncodingState::default();
        let frame = manifest(&[
            SceneChunkManifestEntry::new(
                SceneChunk::from_ops(Arc::from([quad_scene_op()])),
                Rect::new(
                    Point::default(),
                    Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                ),
                Point::default(),
            ),
            SceneChunkManifestEntry::new(
                SceneChunk::from_ops(Arc::from([quad_scene_op()])),
                Rect::new(
                    Point::default(),
                    Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                ),
                Point::new(fret_core::Px(20.0), fret_core::Px(0.0)),
            ),
        ]);
        state.begin_frame_with_payloads(Some(&frame), context(1), &[0, 0], |_| {
            CachedSceneChunkEncoding::new(quad_payload_encoding())
        });

        let encoding = state
            .assemble_resource_free_quad_frame_encoding(&frame, context(1))
            .expect("resource-free quad chunks should assemble");

        assert_eq!(encoding.instances.len(), 2);
        assert_eq!(encoding.uniforms.len(), 2);
        assert_eq!(encoding.uniform_mask_images.len(), 2);
        assert_eq!(encoding.ordered_draws.len(), 2);
        let OrderedDraw::Quad(first) = encoding.ordered_draws[0] else {
            panic!("expected first quad draw");
        };
        let OrderedDraw::Quad(second) = encoding.ordered_draws[1] else {
            panic!("expected second quad draw");
        };
        assert_eq!(first.first_instance, 0);
        assert_eq!(first.uniform_index, 0);
        assert_eq!(second.first_instance, 1);
        assert_eq!(second.uniform_index, 1);
    }

    fn quad_scene_op() -> SceneOp {
        SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(
                Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            background: Color {
                r: 0.25,
                g: 0.5,
                b: 0.75,
                a: 1.0,
            }
            .into(),
            border: fret_core::Edges::all(fret_core::Px(0.0)),
            border_paint: Color::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::all(fret_core::Px(0.0)),
        }
    }

    fn vertex_color_quad_scene_op() -> SceneOp {
        SceneOp::VertexColorQuad {
            order: DrawOrder(0),
            points: [
                Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                Point::new(fret_core::Px(10.0), fret_core::Px(0.0)),
                Point::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                Point::new(fret_core::Px(0.0), fret_core::Px(10.0)),
            ],
            colors: [
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                },
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                },
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
            ],
        }
    }

    fn viewport_vertex_payload_encoding(draw: OrderedDraw) -> SceneEncoding {
        let mut encoding = SceneEncoding::default();
        encoding
            .viewport_vertices
            .push(bytemuck::Zeroable::zeroed());
        encoding.ordered_draws.push(draw);
        encoding
    }

    fn image_viewport_payload_encoding() -> SceneEncoding {
        viewport_vertex_payload_encoding(OrderedDraw::Image(ImageDraw {
            scissor: ScissorRect::full(64, 64),
            uniform_index: 0,
            first_vertex: 0,
            vertex_count: 1,
            image: Default::default(),
            sampling: fret_core::scene::ImageSamplingHint::Default,
        }))
    }

    fn viewport_surface_payload_encoding() -> SceneEncoding {
        viewport_vertex_payload_encoding(OrderedDraw::Viewport(ViewportDraw {
            scissor: ScissorRect::full(64, 64),
            uniform_index: 0,
            first_vertex: 0,
            vertex_count: 1,
            target: Default::default(),
        }))
    }

    fn single_viewport_segment_plan(
        flags: RenderPlanSegmentFlags,
        vertex_count: u32,
    ) -> RenderPlan {
        RenderPlan {
            segments: vec![RenderPlanSegment {
                id: SceneSegmentId(0),
                draw_range: 0..1,
                start_uniform_index: Some(0),
                start_uniform_fingerprint: 0,
                flags,
                scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                    eligible: true,
                    draw_count: 1,
                    fingerprint: 0,
                },
                stream_ranges: RenderPlanSegmentStreamRanges {
                    viewport_vertices: RenderPlanStreamRange::new(0, vertex_count),
                    ..Default::default()
                },
            }],
            passes: Vec::new(),
            compile_stats: RenderPlanCompileStats::default(),
            degradations: Vec::new(),
        }
    }

    fn text_scene_op(origin: Point, text: TextBlobId) -> SceneOp {
        SceneOp::Text {
            order: DrawOrder(0),
            origin,
            text,
            paint: Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            })
            .into(),
            outline: None,
            shadow: None,
        }
    }

    fn text_chunk_entry(text: TextBlobId) -> SceneChunkManifestEntry {
        SceneChunkManifestEntry::new(
            SceneChunk::from_ops(Arc::from([text_scene_op(
                Point::new(fret_core::Px(0.0), fret_core::Px(32.0)),
                text,
            )])),
            Rect::new(
                Point::default(),
                Size::new(fret_core::Px(320.0), fret_core::Px(48.0)),
            ),
            Point::default(),
        )
    }

    #[test]
    fn quad_chunk_payload_encodes_native_ops_with_scene_origin() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let chunk = SceneChunk::from_ops(Arc::from([quad_scene_op()]));
        assert!(chunk.closure().is_resource_free_quad_only());
        let entry = SceneChunkManifestEntry::new(
            chunk.clone(),
            Rect::new(
                Point::default(),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            Point::new(fret_core::Px(2.0), fret_core::Px(3.0)),
        );

        let payload = renderer.encode_scene_chunk_entry_payload(&entry, 1.0, (64, 64), true);

        let mut flat_scene = Scene::default();
        chunk.replay_translated_into(&mut flat_scene, entry.scene_origin());
        let mut flat_encoding = SceneEncoding::default();
        let mut ignored_perf = RenderPerfStats::default();
        renderer.encode_scene_ops_into(
            &flat_scene,
            1.0,
            (64, 64),
            true,
            &mut flat_encoding,
            false,
            false,
            &mut ignored_perf,
        );

        assert_eq!(
            payload.plan_shape,
            SceneChunkPayloadPlanShape::from_ordered_draws(&flat_encoding.ordered_draws)
        );
        assert_eq!(
            payload.stream_fingerprint,
            SceneChunkPayloadStreamFingerprint::from_payload_encoding(&flat_encoding).fingerprint
        );
    }

    #[test]
    fn vertex_color_chunk_payload_alignment_allows_viewport_vertex_reassembly() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let chunk = SceneChunk::from_ops(Arc::from([vertex_color_quad_scene_op()]));
        assert!(chunk.closure().is_resource_free_vertex_color_only());
        let entry = SceneChunkManifestEntry::new(
            chunk.clone(),
            Rect::new(
                Point::default(),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            Point::new(fret_core::Px(2.0), fret_core::Px(3.0)),
        );
        let frame = manifest(std::slice::from_ref(&entry));

        let preview_payload =
            renderer.encode_scene_chunk_entry_payload(&entry, 1.0, (64, 64), true);
        assert!(!preview_payload.encoding.viewport_vertices.is_empty());
        assert!(matches!(
            preview_payload.encoding.ordered_draws.as_slice(),
            [OrderedDraw::VertexColor(_)]
        ));
        let mut payload_for_cache =
            Some(renderer.encode_scene_chunk_entry_payload(&entry, 1.0, (64, 64), true));

        let mut state = SceneChunkEncodingState::default();
        state.begin_frame_with_payloads(Some(&frame), context(1), &[0], |_| {
            payload_for_cache.take().expect("single payload build")
        });

        let mut flat_scene = Scene::default();
        chunk.replay_translated_into(&mut flat_scene, entry.scene_origin());
        let mut flat_encoding = SceneEncoding::default();
        let mut ignored_perf = RenderPerfStats::default();
        renderer.encode_scene_ops_into(
            &flat_scene,
            1.0,
            (64, 64),
            true,
            &mut flat_encoding,
            false,
            false,
            &mut ignored_perf,
        );
        let plan = single_viewport_segment_plan(
            RenderPlanSegmentFlags {
                has_vertex_color: true,
                ..Default::default()
            },
            u32::try_from(flat_encoding.viewport_vertices.len()).expect("viewport vertex count"),
        );

        let alignment = state.record_payload_plan_alignment(&plan, &flat_encoding);
        let stats = alignment.stats;

        assert_eq!(stats.payload_plan_candidate_segments, 1);
        assert_eq!(stats.payload_plan_shape_matches, 1);
        assert_eq!(stats.payload_plan_stream_fingerprint_matches, 1);
        assert_eq!(stats.payload_reassembly_dry_run_candidates, 1);
        assert_eq!(stats.payload_reassembly_append_only_matches, 1);
        assert_eq!(stats.payload_reassembly_blocked_by_non_quad_draws, 0);
        assert_eq!(alignment.reassembly_plan.safe_segment_indices(), &[0]);
    }

    #[test]
    fn image_and_viewport_surface_payloads_remain_blocked_until_resource_closure() {
        let cases: [(RenderPlanSegmentFlags, fn() -> SceneEncoding); 2] = [
            (
                RenderPlanSegmentFlags {
                    has_image: true,
                    ..Default::default()
                },
                image_viewport_payload_encoding,
            ),
            (
                RenderPlanSegmentFlags {
                    has_viewport: true,
                    ..Default::default()
                },
                viewport_surface_payload_encoding,
            ),
        ];

        for (flags, payload_encoding) in cases {
            let mut state = SceneChunkEncodingState::default();
            let frame = manifest(&[entry(0.0)]);
            let flat_encoding = payload_encoding();
            let mut payload_for_cache = Some(payload_encoding());
            state.begin_frame_with_payloads(Some(&frame), context(1), &[0], |_| {
                CachedSceneChunkEncoding::new(payload_for_cache.take().expect("single payload"))
            });
            let plan = single_viewport_segment_plan(flags, 1);

            let alignment = state.record_payload_plan_alignment(&plan, &flat_encoding);

            assert_eq!(alignment.stats.payload_plan_shape_matches, 1);
            assert_eq!(alignment.stats.payload_plan_stream_fingerprint_matches, 1);
            assert_eq!(alignment.stats.payload_reassembly_append_only_matches, 0);
            assert_eq!(
                alignment.stats.payload_reassembly_blocked_by_non_quad_draws,
                1
            );
            assert!(alignment.reassembly_plan.is_empty());
        }
    }

    #[test]
    fn unsupported_scope_chunk_payload_is_not_replayed_into_draw_streams() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let chunk = SceneChunk::from_ops(Arc::from([
            SceneOp::PushClipRect {
                rect: Rect::new(
                    Point::default(),
                    Size::new(fret_core::Px(8.0), fret_core::Px(8.0)),
                ),
            },
            quad_scene_op(),
        ]));
        assert!(!chunk.closure().is_resource_free_quad_only());
        assert_eq!(
            chunk.closure().scope_unsupported_reasons(),
            &[fret_core::SceneChunkClosureUnsupportedReason::OpenScope(
                fret_core::SceneChunkScopeKind::Clip
            )]
        );
        let entry = SceneChunkManifestEntry::new(
            chunk,
            Rect::new(
                Point::default(),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            Point::default(),
        );

        let payload = renderer.encode_scene_chunk_entry_payload(&entry, 1.0, (64, 64), true);

        assert!(payload.encoding.ordered_draws.is_empty());
        assert!(payload.encoding.instances.is_empty());
        assert_eq!(
            payload.stream_fingerprint,
            SceneChunkPayloadStreamFingerprint::from_payload_encoding(&SceneEncoding::default())
                .fingerprint
        );
    }

    #[test]
    fn text_chunk_key_ignores_offscreen_suffix_glyph_residency() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let style = TextStyle {
            size: fret_core::Px(24.0),
            ..Default::default()
        };
        let (blob, _) = renderer.text_system.prepare(
            "abcdefghijklmnopqrstuvwxyz",
            &style,
            TextConstraints::default(),
        );
        let frame = manifest(&[text_chunk_entry(blob)]);

        let mut visible_residency = crate::text::TextFrameResidency::new();
        assert!(renderer.text_system.push_cluster_residency_for_blob(
            &mut visible_residency,
            blob,
            |cluster| cluster.visual_bounds()[0] < 36.0
        ));
        let visible_prepare =
            renderer
                .text_system
                .prepare_for_text_residency_with_perf(&visible_residency, 0, true);
        assert!(visible_prepare.added_glyph_keys > 0);
        let visible_snapshot = renderer
            .text_system
            .text_resource_snapshot_for_residency(&visible_residency);
        assert!(visible_snapshot.glyphs > 0);
        assert_eq!(visible_snapshot.missing_glyph_resources, 0);

        let full_before = renderer
            .text_system
            .test_full_blob_text_resource_snapshot(&[blob]);
        assert!(
            full_before.glyphs > visible_snapshot.glyphs,
            "test setup expects the narrow viewport to leave a suffix outside chunk residency"
        );
        assert!(
            full_before.missing_glyph_resources > 0,
            "test setup expects the offscreen suffix to remain absent before full-blob residency"
        );

        let context = renderer.build_scene_chunk_encoding_context(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            (36, 80),
            1.0,
        );
        let mut first_perf = RenderPerfStats::default();
        renderer.record_scene_chunk_encoding_key_cache_for_frame(
            Some(&frame),
            context,
            1.0,
            true,
            &mut first_perf,
        );
        assert_eq!(first_perf.scene_chunk_encoding_key_cache_misses, 1);

        let full_prepare =
            renderer
                .text_system
                .test_prepare_full_blob_text_with_perf(&[blob], 1, true);
        assert!(full_prepare.added_glyph_keys > 0);
        let full_after = renderer
            .text_system
            .test_full_blob_text_resource_snapshot(&[blob]);
        assert_eq!(full_after.missing_glyph_resources, 0);
        assert_ne!(
            full_before.fingerprint, full_after.fingerprint,
            "full-blob keys would churn when offscreen suffix glyphs become resident"
        );

        let mut second_perf = RenderPerfStats::default();
        renderer.record_scene_chunk_encoding_key_cache_for_frame(
            Some(&frame),
            context,
            1.0,
            true,
            &mut second_perf,
        );

        assert_eq!(
            second_perf.scene_chunk_encoding_key_cache_hits, 1,
            "chunk keys must depend on visible glyph residency, not the full text blob"
        );
        assert_eq!(second_perf.scene_chunk_encoding_key_cache_misses, 0);
    }

    #[test]
    fn balanced_clip_chunk_payload_matches_flat_side_tables_and_blocks_reassembly() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let clip_rect = Rect::new(
            Point::new(fret_core::Px(1.0), fret_core::Px(1.0)),
            Size::new(fret_core::Px(8.0), fret_core::Px(8.0)),
        );
        let chunk = SceneChunk::from_ops(Arc::from([
            SceneOp::PushClipRRect {
                rect: clip_rect,
                corner_radii: fret_core::Corners::all(fret_core::Px(2.0)),
            },
            quad_scene_op(),
            SceneOp::PopClip,
        ]));
        assert!(
            chunk
                .closure()
                .scope(fret_core::SceneChunkScopeKind::Clip)
                .is_balanced()
        );
        let entry = SceneChunkManifestEntry::new(
            chunk.clone(),
            Rect::new(
                Point::default(),
                Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
            ),
            Point::new(fret_core::Px(2.0), fret_core::Px(3.0)),
        );

        let payload = renderer.encode_scene_chunk_entry_payload(&entry, 1.0, (64, 64), true);
        let mut flat_scene = Scene::default();
        chunk.replay_translated_into(&mut flat_scene, entry.scene_origin());
        let mut flat_encoding = SceneEncoding::default();
        let mut ignored_perf = RenderPerfStats::default();
        renderer.encode_scene_ops_into(
            &flat_scene,
            1.0,
            (64, 64),
            true,
            &mut flat_encoding,
            false,
            false,
            &mut ignored_perf,
        );

        assert_eq!(
            payload.stream_fingerprint,
            SceneChunkPayloadStreamFingerprint::from_payload_encoding(&flat_encoding).fingerprint
        );
        assert_eq!(
            payload.encoding.ordered_draws.len(),
            flat_encoding.ordered_draws.len()
        );
        assert_eq!(payload.encoding.clips.len(), flat_encoding.clips.len());
        assert_eq!(
            payload.encoding.uniforms.len().saturating_add(1),
            flat_encoding.uniforms.len()
        );
        assert_eq!(
            payload.append_only_reassembly_blocker(),
            Some(SceneChunkPayloadReassemblyBlocker::SideTables)
        );
    }

    #[test]
    fn chunk_encoding_key_cache_tracks_hits_misses_and_stale_slots() {
        let mut state = SceneChunkEncodingState::default();
        let first = entry(0.0);
        let second = entry(20.0);
        let frame = manifest(&[first.clone(), second.clone()]);

        let stats = begin_frame(&mut state, Some(&frame), context(1));
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.key_cache_hits, 0);
        assert_eq!(stats.key_cache_misses, 2);
        assert_eq!(stats.key_cache_stale_entries, 0);
        assert_ne!(stats.key_cache_context_fingerprint, 0);

        let stats = begin_frame(&mut state, Some(&frame), context(1));
        assert_eq!(stats.key_cache_hits, 2);
        assert_eq!(stats.key_cache_misses, 0);
        assert_eq!(stats.key_cache_stale_entries, 0);

        let moved = entry(40.0);
        let frame = manifest(&[first, moved]);
        let stats = begin_frame(&mut state, Some(&frame), context(1));
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

        let stats = begin_frame(&mut state, Some(&one), context(1));
        assert_eq!(stats.key_cache_misses, 1);

        let stats = begin_frame(&mut state, Some(&duplicated), context(1));
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 1);
        assert_eq!(stats.key_cache_stale_entries, 0);

        let stats = begin_frame(&mut state, Some(&one), context(1));
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 0);
        assert_eq!(stats.key_cache_stale_entries, 1);
    }

    #[test]
    fn chunk_encoding_key_cache_context_changes_invalidate_entries() {
        let mut state = SceneChunkEncodingState::default();
        let entry = entry(0.0);
        let frame = manifest(&[entry]);

        begin_frame(&mut state, Some(&frame), context(1));
        let stats = begin_frame(&mut state, Some(&frame), context(2));
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
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), &[0], |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 0);
        assert_eq!(stats.payload_cache_misses, 1);
        assert_eq!(stats.payload_chunks_encoded, 1);
        assert_eq!(stats.payload_entries_live, 1);
        assert_eq!(builds, 1);

        let frame = manifest(&[first.clone(), second]);
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), &[0, 0], |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 1);
        assert_eq!(stats.payload_cache_misses, 1);
        assert_eq!(stats.payload_chunks_encoded, 1);
        assert_eq!(stats.payload_entries_live, 2);
        assert_eq!(builds, 2);

        let frame = manifest(&[first]);
        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), &[0], |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.payload_cache_hits, 1);
        assert_eq!(stats.payload_cache_misses, 0);
        assert_eq!(stats.payload_chunks_encoded, 0);
        assert_eq!(stats.payload_entries_live, 1);
        assert_eq!(builds, 2);
    }

    #[test]
    fn chunk_encoding_payload_cache_uses_entry_local_text_resource_keys() {
        let mut state = SceneChunkEncodingState::default();
        let first = entry(0.0);
        let second = entry(20.0);
        let frame = manifest(&[first, second]);
        let mut builds = 0u64;

        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), &[10, 20], |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.key_cache_hits, 0);
        assert_eq!(stats.key_cache_misses, 2);
        assert_eq!(stats.payload_cache_hits, 0);
        assert_eq!(stats.payload_cache_misses, 2);
        assert_eq!(builds, 2);

        let stats = state.begin_frame_with_payloads(Some(&frame), context(1), &[10, 21], |_| {
            builds += 1;
            CachedSceneChunkEncoding::default()
        });
        assert_eq!(stats.key_cache_hits, 1);
        assert_eq!(stats.key_cache_misses, 1);
        assert_eq!(stats.key_cache_stale_entries, 1);
        assert_eq!(stats.payload_cache_hits, 1);
        assert_eq!(stats.payload_cache_misses, 1);
        assert_eq!(stats.payload_chunks_encoded, 1);
        assert_eq!(stats.payload_entries_live, 2);
        assert_eq!(builds, 3);
    }

    #[test]
    fn payload_plan_alignment_compares_cached_payloads_to_candidate_segments_in_order() {
        let mut state = SceneChunkEncodingState::default();
        let frame = manifest(&[entry(0.0)]);
        state.begin_frame_with_payloads(Some(&frame), context(1), &[0], |_| {
            CachedSceneChunkEncoding::new(quad_payload_encoding())
        });
        let mut flat_encoding = quad_payload_encoding();
        let ranges = RenderPlanSegmentStreamRanges {
            quad_instances: RenderPlanStreamRange::new(0, 1),
            ..Default::default()
        };
        assert_eq!(
            SceneChunkPayloadStreamFingerprint::from_payload_encoding(&flat_encoding).fingerprint,
            SceneChunkPayloadStreamFingerprint::from_flat_encoding_segment(&flat_encoding, ranges)
                .fingerprint
        );

        let plan = RenderPlan {
            segments: vec![RenderPlanSegment {
                id: SceneSegmentId(0),
                draw_range: 0..1,
                start_uniform_index: Some(0),
                start_uniform_fingerprint: 0,
                flags: RenderPlanSegmentFlags {
                    has_quad: true,
                    ..Default::default()
                },
                scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                    eligible: true,
                    draw_count: 1,
                    fingerprint: 0,
                },
                stream_ranges: ranges,
            }],
            passes: Vec::new(),
            compile_stats: RenderPlanCompileStats::default(),
            degradations: Vec::new(),
        };

        let alignment = state.record_payload_plan_alignment(&plan, &flat_encoding);
        let stats = alignment.stats;

        assert_eq!(stats.payload_plan_candidate_segments, 1);
        assert_eq!(stats.payload_plan_shape_matches, 1);
        assert_eq!(stats.payload_plan_shape_mismatches, 0);
        assert_eq!(stats.payload_plan_stream_fingerprint_matches, 1);
        assert_eq!(stats.payload_plan_stream_fingerprint_mismatches, 0);
        assert_eq!(stats.payload_reassembly_dry_run_candidates, 1);
        assert_eq!(stats.payload_reassembly_append_only_matches, 1);
        assert_eq!(stats.payload_reassembly_blocked_by_shape_mismatch, 0);
        assert_eq!(
            stats.payload_reassembly_blocked_by_stream_fingerprint_mismatch,
            0
        );
        assert_eq!(stats.payload_reassembly_blocked_by_non_quad_draws, 0);
        assert_eq!(stats.payload_reassembly_blocked_by_side_tables, 0);
        assert_eq!(stats.payload_reassembly_blocked_by_material_state, 0);
        assert_eq!(stats.payload_entries_without_plan_candidate, 0);
        assert_eq!(stats.payload_plan_candidates_without_payload, 0);
        assert_eq!(alignment.reassembly_plan.safe_segment_indices(), &[0]);

        flat_encoding.instances[0].rect[0] = 1.0;
        let alignment = state.record_payload_plan_alignment(&plan, &flat_encoding);
        let stats = alignment.stats;
        assert_eq!(stats.payload_plan_shape_matches, 1);
        assert_eq!(stats.payload_plan_stream_fingerprint_matches, 0);
        assert_eq!(stats.payload_plan_stream_fingerprint_mismatches, 1);
        assert_eq!(stats.payload_reassembly_dry_run_candidates, 1);
        assert_eq!(stats.payload_reassembly_append_only_matches, 0);
        assert_eq!(
            stats.payload_reassembly_blocked_by_stream_fingerprint_mismatch,
            1
        );
        assert!(alignment.reassembly_plan.is_empty());
    }

    #[test]
    fn payload_plan_alignment_returns_exact_safe_segment_indices() {
        let mut state = SceneChunkEncodingState::default();
        let frame = manifest(&[entry(0.0), entry(20.0)]);
        state.begin_frame_with_payloads(Some(&frame), context(1), &[0, 0], |_| {
            CachedSceneChunkEncoding::new(quad_payload_encoding())
        });

        let mut flat_encoding = quad_payload_encoding();
        flat_encoding.instances[0].rect[0] = 1.0;
        let instance: QuadInstance = bytemuck::Zeroable::zeroed();
        flat_encoding.instances.push(instance);
        let plan = RenderPlan {
            segments: vec![
                RenderPlanSegment {
                    id: SceneSegmentId(0),
                    draw_range: 0..1,
                    start_uniform_index: Some(0),
                    start_uniform_fingerprint: 0,
                    flags: RenderPlanSegmentFlags {
                        has_quad: true,
                        ..Default::default()
                    },
                    scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                        eligible: true,
                        draw_count: 1,
                        fingerprint: 10,
                    },
                    stream_ranges: RenderPlanSegmentStreamRanges {
                        quad_instances: RenderPlanStreamRange::new(0, 1),
                        ..Default::default()
                    },
                },
                RenderPlanSegment {
                    id: SceneSegmentId(1),
                    draw_range: 1..2,
                    start_uniform_index: Some(0),
                    start_uniform_fingerprint: 0,
                    flags: RenderPlanSegmentFlags {
                        has_quad: true,
                        ..Default::default()
                    },
                    scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                        eligible: true,
                        draw_count: 1,
                        fingerprint: 20,
                    },
                    stream_ranges: RenderPlanSegmentStreamRanges {
                        quad_instances: RenderPlanStreamRange::new(1, 2),
                        ..Default::default()
                    },
                },
            ],
            passes: Vec::new(),
            compile_stats: RenderPlanCompileStats::default(),
            degradations: Vec::new(),
        };

        let alignment = state.record_payload_plan_alignment(&plan, &flat_encoding);

        assert_eq!(alignment.stats.payload_reassembly_dry_run_candidates, 2);
        assert_eq!(alignment.stats.payload_reassembly_append_only_matches, 1);
        assert_eq!(
            alignment
                .stats
                .payload_reassembly_blocked_by_stream_fingerprint_mismatch,
            1
        );
        assert_eq!(alignment.reassembly_plan.safe_segment_indices(), &[1]);
    }
}
