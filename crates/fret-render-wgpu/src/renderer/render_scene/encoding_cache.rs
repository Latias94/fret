use super::super::*;
use fret_core::time::Instant;

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
        SceneEncodingCacheKey {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            scene_fingerprint: scene.fingerprint(),
            scene_ops_len: scene.ops_len(),
            render_targets_generation,
            images_generation,
            text_atlas_revision,
            text_quality_key: self.text_system.text_quality_key(),
            materials_generation: self.materials_generation,
            material_paint_budget_per_frame: self.material_paint_budget_per_frame,
            material_distinct_budget_per_frame: self.material_distinct_budget_per_frame,
            custom_effects_generation: self.custom_effects_generation,
        }
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
        let cache_hit = self.scene_encoding_cache.is_hit(key);
        render_scene_span.record("encoding_cache_hit", cache_hit);
        let miss_reasons = if !cache_hit && (perf_enabled || trace_enabled) {
            miss_reasons_for_key_change(self.scene_encoding_cache.key(), key)
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

        let mut encoding = self.scene_encoding_cache.take_for_frame(cache_hit);
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

        self.scene_encoding_cache.note_miss(key);
        (encoding, false)
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
