use super::scene_encoding_cache_diagnostics::record_scene_encoding_cache_frame_result;
use super::*;

pub(super) struct SceneEncodingKeyInputs {
    pub(super) format: wgpu::TextureFormat,
    pub(super) viewport_size: (u32, u32),
    pub(super) scale_factor_bits: u32,
    pub(super) scene_fingerprint: u64,
    pub(super) scene_ops_len: usize,
    pub(super) render_targets_generation: u64,
    pub(super) images_generation: u64,
    pub(super) text_atlas_revision: u64,
    pub(super) text_quality_key: u64,
    pub(super) materials_generation: u64,
    pub(super) material_paint_budget_per_frame: u64,
    pub(super) material_distinct_budget_per_frame: usize,
    pub(super) custom_effects_generation: u64,
}

#[derive(Default)]
pub(super) struct SceneEncodingState {
    key: Option<SceneEncodingCacheKey>,
    cache: SceneEncoding,
    scratch: SceneEncoding,
}

impl SceneEncodingState {
    pub(super) fn build_key(inputs: SceneEncodingKeyInputs) -> SceneEncodingCacheKey {
        SceneEncodingCacheKey {
            format: inputs.format,
            viewport_size: inputs.viewport_size,
            scale_factor_bits: inputs.scale_factor_bits,
            scene_fingerprint: inputs.scene_fingerprint,
            scene_ops_len: inputs.scene_ops_len,
            render_targets_generation: inputs.render_targets_generation,
            images_generation: inputs.images_generation,
            text_atlas_revision: inputs.text_atlas_revision,
            text_quality_key: inputs.text_quality_key,
            materials_generation: inputs.materials_generation,
            material_paint_budget_per_frame: inputs.material_paint_budget_per_frame,
            material_distinct_budget_per_frame: inputs.material_distinct_budget_per_frame,
            custom_effects_generation: inputs.custom_effects_generation,
        }
    }

    pub(super) fn begin_frame(
        &mut self,
        key: SceneEncodingCacheKey,
        perf_enabled: bool,
        trace_enabled: bool,
        render_scene_span: &tracing::Span,
        frame_perf: &mut RenderPerfStats,
    ) -> (SceneEncoding, bool) {
        let cache_hit = self.is_hit(key);
        record_scene_encoding_cache_frame_result(
            self.key(),
            key,
            cache_hit,
            perf_enabled,
            trace_enabled,
            render_scene_span,
            frame_perf,
        );

        (self.take_for_frame(cache_hit), cache_hit)
    }

    pub(super) fn note_miss(&mut self, key: SceneEncodingCacheKey) {
        // Preserve the old cache's allocations for reuse.
        self.scratch = std::mem::take(&mut self.cache);
        self.key = Some(key);
    }

    pub(super) fn store_after_frame(
        &mut self,
        key: SceneEncodingCacheKey,
        cache_hit: bool,
        encoding: SceneEncoding,
    ) {
        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.key = Some(key);
        }
        self.cache = encoding;
    }

    fn is_hit(&self, key: SceneEncodingCacheKey) -> bool {
        self.key == Some(key)
    }

    fn key(&self) -> Option<SceneEncodingCacheKey> {
        self.key
    }

    fn take_for_frame(&mut self, cache_hit: bool) -> SceneEncoding {
        if cache_hit {
            std::mem::take(&mut self.cache)
        } else {
            std::mem::take(&mut self.scratch)
        }
    }
}

#[cfg(test)]
impl SceneEncodingState {
    pub(super) fn cache_key(&self) -> Option<SceneEncodingCacheKey> {
        self.key()
    }

    pub(super) fn cache(&self) -> &SceneEncoding {
        &self.cache
    }
}
