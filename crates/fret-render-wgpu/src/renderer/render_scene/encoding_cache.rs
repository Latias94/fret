use super::super::*;
use fret_core::time::Instant;

impl Renderer {
    pub(super) fn build_scene_encoding_cache_key(
        &self,
        format: wgpu::TextureFormat,
        viewport_size: (u32, u32),
        scale_factor: f32,
        scene: &Scene,
        text_atlas_revision: u64,
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
            },
        )
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

        let encode_start = perf_enabled.then(Instant::now);
        {
            let encode_span = if trace_enabled {
                tracing::trace_span!("fret.renderer.scene.encode", frame_index)
            } else {
                tracing::Span::none()
            };
            let _guard = encode_span.enter();
            self.encode_scene_ops_into(
                scene,
                scale_factor,
                viewport_size,
                format_is_srgb,
                &mut encoding,
            );
        }
        if let Some(encode_start) = encode_start {
            frame_perf.encode_scene += encode_start.elapsed();
        }

        self.scene_encoding_state.note_miss(key);
        (encoding, false)
    }
}
