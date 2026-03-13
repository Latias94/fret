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

const MISS_COLD_START: u64 = 1 << 0;
const MISS_FORMAT_CHANGED: u64 = 1 << 1;
const MISS_VIEWPORT_SIZE_CHANGED: u64 = 1 << 2;
const MISS_SCALE_FACTOR_CHANGED: u64 = 1 << 3;
const MISS_SCENE_FINGERPRINT_CHANGED: u64 = 1 << 4;
const MISS_SCENE_OPS_LEN_CHANGED: u64 = 1 << 5;
const MISS_RENDER_TARGETS_GENERATION_CHANGED: u64 = 1 << 6;
const MISS_IMAGES_GENERATION_CHANGED: u64 = 1 << 7;
const MISS_TEXT_ATLAS_REVISION_CHANGED: u64 = 1 << 8;
const MISS_TEXT_QUALITY_KEY_CHANGED: u64 = 1 << 9;
const MISS_MATERIALS_GENERATION_CHANGED: u64 = 1 << 10;
const MISS_MATERIAL_PAINT_BUDGET_CHANGED: u64 = 1 << 11;
const MISS_MATERIAL_DISTINCT_BUDGET_CHANGED: u64 = 1 << 12;
const MISS_CUSTOM_EFFECTS_GENERATION_CHANGED: u64 = 1 << 13;

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
        render_scene_span.record("encoding_cache_hit", cache_hit);
        let miss_reasons = if !cache_hit && (perf_enabled || trace_enabled) {
            miss_reasons_for_key_change(self.key(), key)
        } else {
            0
        };
        if !cache_hit && miss_reasons != 0 {
            render_scene_span.record("encoding_cache_miss_reasons", miss_reasons);
            if trace_enabled {
                render_scene_span.record(
                    "encoding_cache_miss_reason",
                    tracing::field::display(MissReasonDisplay(miss_reasons)),
                );
            }
        }
        if perf_enabled {
            if cache_hit {
                frame_perf.scene_encoding_cache_hits =
                    frame_perf.scene_encoding_cache_hits.saturating_add(1);
            } else {
                frame_perf.scene_encoding_cache_misses =
                    frame_perf.scene_encoding_cache_misses.saturating_add(1);
                frame_perf.scene_encoding_cache_last_miss_reasons = miss_reasons;
            }
        }

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

fn miss_reasons_for_key_change(
    prev: Option<SceneEncodingCacheKey>,
    next: SceneEncodingCacheKey,
) -> u64 {
    let Some(prev) = prev else {
        return MISS_COLD_START;
    };

    let mut reasons = 0u64;
    if prev.format != next.format {
        reasons |= MISS_FORMAT_CHANGED;
    }
    if prev.viewport_size != next.viewport_size {
        reasons |= MISS_VIEWPORT_SIZE_CHANGED;
    }
    if prev.scale_factor_bits != next.scale_factor_bits {
        reasons |= MISS_SCALE_FACTOR_CHANGED;
    }
    if prev.scene_fingerprint != next.scene_fingerprint {
        reasons |= MISS_SCENE_FINGERPRINT_CHANGED;
    }
    if prev.scene_ops_len != next.scene_ops_len {
        reasons |= MISS_SCENE_OPS_LEN_CHANGED;
    }
    if prev.render_targets_generation != next.render_targets_generation {
        reasons |= MISS_RENDER_TARGETS_GENERATION_CHANGED;
    }
    if prev.images_generation != next.images_generation {
        reasons |= MISS_IMAGES_GENERATION_CHANGED;
    }
    if prev.text_atlas_revision != next.text_atlas_revision {
        reasons |= MISS_TEXT_ATLAS_REVISION_CHANGED;
    }
    if prev.text_quality_key != next.text_quality_key {
        reasons |= MISS_TEXT_QUALITY_KEY_CHANGED;
    }
    if prev.materials_generation != next.materials_generation {
        reasons |= MISS_MATERIALS_GENERATION_CHANGED;
    }
    if prev.material_paint_budget_per_frame != next.material_paint_budget_per_frame {
        reasons |= MISS_MATERIAL_PAINT_BUDGET_CHANGED;
    }
    if prev.material_distinct_budget_per_frame != next.material_distinct_budget_per_frame {
        reasons |= MISS_MATERIAL_DISTINCT_BUDGET_CHANGED;
    }
    if prev.custom_effects_generation != next.custom_effects_generation {
        reasons |= MISS_CUSTOM_EFFECTS_GENERATION_CHANGED;
    }
    reasons
}

struct MissReasonDisplay(u64);

impl std::fmt::Display for MissReasonDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        let push = |s: &str, f: &mut std::fmt::Formatter<'_>, first: &mut bool| {
            if !*first {
                let _ = f.write_str("|");
            }
            *first = false;
            f.write_str(s)
        };

        let r = self.0;
        if r == 0 {
            return f.write_str("unknown");
        }
        if (r & MISS_COLD_START) != 0 {
            push("cold_start", f, &mut first)?;
        }
        if (r & MISS_FORMAT_CHANGED) != 0 {
            push("format", f, &mut first)?;
        }
        if (r & MISS_VIEWPORT_SIZE_CHANGED) != 0 {
            push("viewport", f, &mut first)?;
        }
        if (r & MISS_SCALE_FACTOR_CHANGED) != 0 {
            push("scale_factor", f, &mut first)?;
        }
        if (r & MISS_SCENE_FINGERPRINT_CHANGED) != 0 {
            push("scene_fingerprint", f, &mut first)?;
        }
        if (r & MISS_SCENE_OPS_LEN_CHANGED) != 0 {
            push("scene_ops_len", f, &mut first)?;
        }
        if (r & MISS_RENDER_TARGETS_GENERATION_CHANGED) != 0 {
            push("render_targets_generation", f, &mut first)?;
        }
        if (r & MISS_IMAGES_GENERATION_CHANGED) != 0 {
            push("images_generation", f, &mut first)?;
        }
        if (r & MISS_TEXT_ATLAS_REVISION_CHANGED) != 0 {
            push("text_atlas_revision", f, &mut first)?;
        }
        if (r & MISS_TEXT_QUALITY_KEY_CHANGED) != 0 {
            push("text_quality_key", f, &mut first)?;
        }
        if (r & MISS_MATERIALS_GENERATION_CHANGED) != 0 {
            push("materials_generation", f, &mut first)?;
        }
        if (r & MISS_MATERIAL_PAINT_BUDGET_CHANGED) != 0 {
            push("material_paint_budget", f, &mut first)?;
        }
        if (r & MISS_MATERIAL_DISTINCT_BUDGET_CHANGED) != 0 {
            push("material_distinct_budget", f, &mut first)?;
        }
        if (r & MISS_CUSTOM_EFFECTS_GENERATION_CHANGED) != 0 {
            push("custom_effects_generation", f, &mut first)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miss_reasons_include_material_registry_and_budgets() {
        let base = SceneEncodingCacheKey {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            viewport_size: (100, 100),
            scale_factor_bits: 1.0f32.to_bits(),
            scene_fingerprint: 1,
            scene_ops_len: 1,
            render_targets_generation: 0,
            images_generation: 0,
            text_atlas_revision: 0,
            text_quality_key: 0,
            materials_generation: 0,
            material_paint_budget_per_frame: 50_000,
            material_distinct_budget_per_frame: 256,
            custom_effects_generation: 0,
        };

        let mut next = base;
        next.materials_generation = 1;
        let r = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(r & MISS_MATERIALS_GENERATION_CHANGED, 0);

        let mut next = base;
        next.material_paint_budget_per_frame = 123;
        let r = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(r & MISS_MATERIAL_PAINT_BUDGET_CHANGED, 0);

        let mut next = base;
        next.material_distinct_budget_per_frame = 99;
        let r = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(r & MISS_MATERIAL_DISTINCT_BUDGET_CHANGED, 0);
    }
}
