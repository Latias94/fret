use super::super::*;

impl Renderer {
    pub(super) fn build_scene_encoding_cache_key_for_scene_chunks(
        &self,
        format: wgpu::TextureFormat,
        viewport_size: (u32, u32),
        scale_factor: f32,
        scene_chunks: &fret_core::SceneChunkManifest,
        text_scene_resource_key: u64,
    ) -> SceneEncodingCacheKey {
        let (render_targets_generation, images_generation) = self.gpu_resources.generations();
        super::super::scene_encoding_cache::SceneEncodingState::build_key(
            super::super::scene_encoding_cache::SceneEncodingKeyInputs {
                format,
                viewport_size,
                scale_factor_bits: scale_factor.to_bits(),
                scene_fingerprint: chunk_native_scene_fingerprint(scene_chunks),
                scene_ops_len: scene_chunks.ops_len(),
                render_targets_generation,
                images_generation,
                text_scene_resource_key,
                text_quality_key: self.text_system.text_quality_key(),
                materials_generation: self.material_effect_state.materials_generation,
                material_paint_budget_per_frame: self
                    .material_effect_state
                    .material_paint_budget_per_frame,
                material_distinct_budget_per_frame: self
                    .material_effect_state
                    .material_distinct_budget_per_frame,
                custom_effects_generation: self.material_effect_state.custom_effects_generation,
            },
        )
    }

    pub(super) fn build_scene_encoding_cache_key(
        &self,
        format: wgpu::TextureFormat,
        viewport_size: (u32, u32),
        scale_factor: f32,
        scene: &Scene,
        text_scene_resource_key: u64,
    ) -> SceneEncodingCacheKey {
        let (render_targets_generation, images_generation) = self.gpu_resources.generations();
        super::super::scene_encoding_cache::SceneEncodingState::build_key(
            super::super::scene_encoding_cache::SceneEncodingKeyInputs {
                format,
                viewport_size,
                scale_factor_bits: scale_factor.to_bits(),
                scene_fingerprint: scene.fingerprint(),
                scene_ops_len: scene.ops_len(),
                render_targets_generation,
                images_generation,
                text_scene_resource_key,
                text_quality_key: self.text_system.text_quality_key(),
                materials_generation: self.material_effect_state.materials_generation,
                material_paint_budget_per_frame: self
                    .material_effect_state
                    .material_paint_budget_per_frame,
                material_distinct_budget_per_frame: self
                    .material_effect_state
                    .material_distinct_budget_per_frame,
                custom_effects_generation: self.material_effect_state.custom_effects_generation,
            },
        )
    }

    pub(super) fn acquire_scene_encoding_from_chunk_payloads_for_frame(
        &mut self,
        key: SceneEncodingCacheKey,
        scene_chunks: &fret_core::SceneChunkManifest,
        context: SceneChunkEncodingContext,
        stream_class: ChunkLaunchStreamClass,
        perf_enabled: bool,
        trace_enabled: bool,
        render_scene_span: &tracing::Span,
        frame_perf: &mut RenderPerfStats,
    ) -> Option<(SceneEncoding, bool)> {
        let assembled = self.frame_assembler.assemble_supported_frame_encoding(
            scene_chunks,
            context,
            stream_class,
        )?;
        let (cached_encoding, cache_hit) = self.scene_encoding_state.begin_frame(
            key,
            perf_enabled,
            trace_enabled,
            render_scene_span,
            frame_perf,
        );
        if cache_hit {
            return Some((cached_encoding, true));
        }

        self.scene_encoding_state.note_miss(key);
        Some((assembled, false))
    }

    pub(super) fn acquire_scene_encoding_for_frame(
        &mut self,
        key: SceneEncodingCacheKey,
        frame_index: u64,
        scene: &Scene,
        scale_factor: f32,
        viewport_size: (u32, u32),
        format_is_srgb: bool,
        perf_enabled: bool,
        trace_enabled: bool,
        render_scene_span: &tracing::Span,
        frame_perf: &mut RenderPerfStats,
    ) -> (SceneEncoding, bool) {
        let (mut encoding, cache_hit) = self.scene_encoding_state.begin_frame(
            key,
            perf_enabled,
            trace_enabled,
            render_scene_span,
            frame_perf,
        );
        if cache_hit {
            return (encoding, true);
        }

        let encode_family_profile_enabled = perf_enabled
            && std::env::var_os("FRET_DIAG_RENDERER_ENCODE_FAMILY_PROFILE")
                .is_some_and(|value| !value.is_empty());
        let (_, encode_elapsed) = fret_perf::measure_span(
            perf_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.renderer.scene.encode", frame_index),
            || {
                self.encode_scene_ops_into(
                    scene,
                    scale_factor,
                    viewport_size,
                    format_is_srgb,
                    &mut encoding,
                    perf_enabled,
                    encode_family_profile_enabled,
                    frame_perf,
                );
            },
        );
        if let Some(encode_elapsed) = encode_elapsed {
            frame_perf.encode_scene += encode_elapsed;
        }

        self.scene_encoding_state.note_miss(key);
        (encoding, false)
    }
}

fn chunk_native_scene_fingerprint(scene_chunks: &fret_core::SceneChunkManifest) -> u64 {
    scene_chunks
        .fingerprint()
        .wrapping_add(0x41c6_4e6d_37a1_91f5)
        .rotate_left(17)
}
