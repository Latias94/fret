use super::*;

pub(super) const SCENE_ENCODING_CACHE_MISS_COLD_START: u64 = 1 << 0;
pub(super) const SCENE_ENCODING_CACHE_MISS_FORMAT_CHANGED: u64 = 1 << 1;
pub(super) const SCENE_ENCODING_CACHE_MISS_VIEWPORT_SIZE_CHANGED: u64 = 1 << 2;
pub(super) const SCENE_ENCODING_CACHE_MISS_SCALE_FACTOR_CHANGED: u64 = 1 << 3;
pub(super) const SCENE_ENCODING_CACHE_MISS_SCENE_FINGERPRINT_CHANGED: u64 = 1 << 4;
pub(super) const SCENE_ENCODING_CACHE_MISS_SCENE_OPS_LEN_CHANGED: u64 = 1 << 5;
pub(super) const SCENE_ENCODING_CACHE_MISS_RENDER_TARGETS_GENERATION_CHANGED: u64 = 1 << 6;
pub(super) const SCENE_ENCODING_CACHE_MISS_IMAGES_GENERATION_CHANGED: u64 = 1 << 7;
pub(super) const SCENE_ENCODING_CACHE_MISS_TEXT_ATLAS_REVISION_CHANGED: u64 = 1 << 8;
pub(super) const SCENE_ENCODING_CACHE_MISS_TEXT_QUALITY_KEY_CHANGED: u64 = 1 << 9;
pub(super) const SCENE_ENCODING_CACHE_MISS_MATERIALS_GENERATION_CHANGED: u64 = 1 << 10;
pub(super) const SCENE_ENCODING_CACHE_MISS_MATERIAL_PAINT_BUDGET_CHANGED: u64 = 1 << 11;
pub(super) const SCENE_ENCODING_CACHE_MISS_MATERIAL_DISTINCT_BUDGET_CHANGED: u64 = 1 << 12;
pub(super) const SCENE_ENCODING_CACHE_MISS_CUSTOM_EFFECTS_GENERATION_CHANGED: u64 = 1 << 13;

pub(super) fn record_scene_encoding_cache_frame_result(
    prev: Option<SceneEncodingCacheKey>,
    next: SceneEncodingCacheKey,
    cache_hit: bool,
    perf_enabled: bool,
    trace_enabled: bool,
    render_scene_span: &tracing::Span,
    frame_perf: &mut RenderPerfStats,
) {
    render_scene_span.record("encoding_cache_hit", cache_hit);

    let miss_reasons = if !cache_hit && (perf_enabled || trace_enabled) {
        miss_reasons_for_key_change(prev, next)
    } else {
        0
    };

    if !cache_hit && miss_reasons != 0 {
        render_scene_span.record("encoding_cache_miss_reasons", miss_reasons);
        if trace_enabled {
            render_scene_span.record(
                "encoding_cache_miss_reason",
                tracing::field::display(SceneEncodingCacheMissReasonDisplay(miss_reasons)),
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
}

fn miss_reasons_for_key_change(
    prev: Option<SceneEncodingCacheKey>,
    next: SceneEncodingCacheKey,
) -> u64 {
    let Some(prev) = prev else {
        return SCENE_ENCODING_CACHE_MISS_COLD_START;
    };

    let mut reasons = 0u64;
    if prev.format != next.format {
        reasons |= SCENE_ENCODING_CACHE_MISS_FORMAT_CHANGED;
    }
    if prev.viewport_size != next.viewport_size {
        reasons |= SCENE_ENCODING_CACHE_MISS_VIEWPORT_SIZE_CHANGED;
    }
    if prev.scale_factor_bits != next.scale_factor_bits {
        reasons |= SCENE_ENCODING_CACHE_MISS_SCALE_FACTOR_CHANGED;
    }
    if prev.scene_fingerprint != next.scene_fingerprint {
        reasons |= SCENE_ENCODING_CACHE_MISS_SCENE_FINGERPRINT_CHANGED;
    }
    if prev.scene_ops_len != next.scene_ops_len {
        reasons |= SCENE_ENCODING_CACHE_MISS_SCENE_OPS_LEN_CHANGED;
    }
    if prev.render_targets_generation != next.render_targets_generation {
        reasons |= SCENE_ENCODING_CACHE_MISS_RENDER_TARGETS_GENERATION_CHANGED;
    }
    if prev.images_generation != next.images_generation {
        reasons |= SCENE_ENCODING_CACHE_MISS_IMAGES_GENERATION_CHANGED;
    }
    if prev.text_atlas_revision != next.text_atlas_revision {
        reasons |= SCENE_ENCODING_CACHE_MISS_TEXT_ATLAS_REVISION_CHANGED;
    }
    if prev.text_quality_key != next.text_quality_key {
        reasons |= SCENE_ENCODING_CACHE_MISS_TEXT_QUALITY_KEY_CHANGED;
    }
    if prev.materials_generation != next.materials_generation {
        reasons |= SCENE_ENCODING_CACHE_MISS_MATERIALS_GENERATION_CHANGED;
    }
    if prev.material_paint_budget_per_frame != next.material_paint_budget_per_frame {
        reasons |= SCENE_ENCODING_CACHE_MISS_MATERIAL_PAINT_BUDGET_CHANGED;
    }
    if prev.material_distinct_budget_per_frame != next.material_distinct_budget_per_frame {
        reasons |= SCENE_ENCODING_CACHE_MISS_MATERIAL_DISTINCT_BUDGET_CHANGED;
    }
    if prev.custom_effects_generation != next.custom_effects_generation {
        reasons |= SCENE_ENCODING_CACHE_MISS_CUSTOM_EFFECTS_GENERATION_CHANGED;
    }
    reasons
}

struct SceneEncodingCacheMissReasonDisplay(u64);

impl std::fmt::Display for SceneEncodingCacheMissReasonDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        let push = |s: &str, f: &mut std::fmt::Formatter<'_>, first: &mut bool| {
            if !*first {
                let _ = f.write_str("|");
            }
            *first = false;
            f.write_str(s)
        };

        let reasons = self.0;
        if reasons == 0 {
            return f.write_str("unknown");
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_COLD_START) != 0 {
            push("cold_start", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_FORMAT_CHANGED) != 0 {
            push("format", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_VIEWPORT_SIZE_CHANGED) != 0 {
            push("viewport", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_SCALE_FACTOR_CHANGED) != 0 {
            push("scale_factor", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_SCENE_FINGERPRINT_CHANGED) != 0 {
            push("scene_fingerprint", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_SCENE_OPS_LEN_CHANGED) != 0 {
            push("scene_ops_len", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_RENDER_TARGETS_GENERATION_CHANGED) != 0 {
            push("render_targets_generation", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_IMAGES_GENERATION_CHANGED) != 0 {
            push("images_generation", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_TEXT_ATLAS_REVISION_CHANGED) != 0 {
            push("text_atlas_revision", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_TEXT_QUALITY_KEY_CHANGED) != 0 {
            push("text_quality_key", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_MATERIALS_GENERATION_CHANGED) != 0 {
            push("materials_generation", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_MATERIAL_PAINT_BUDGET_CHANGED) != 0 {
            push("material_paint_budget", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_MATERIAL_DISTINCT_BUDGET_CHANGED) != 0 {
            push("material_distinct_budget", f, &mut first)?;
        }
        if (reasons & SCENE_ENCODING_CACHE_MISS_CUSTOM_EFFECTS_GENERATION_CHANGED) != 0 {
            push("custom_effects_generation", f, &mut first)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_key() -> SceneEncodingCacheKey {
        SceneEncodingCacheKey {
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
        }
    }

    #[test]
    fn miss_reasons_include_material_registry_and_budgets() {
        let base = base_key();

        let mut next = base;
        next.materials_generation = 1;
        let reasons = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(
            reasons & SCENE_ENCODING_CACHE_MISS_MATERIALS_GENERATION_CHANGED,
            0
        );

        let mut next = base;
        next.material_paint_budget_per_frame = 123;
        let reasons = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(
            reasons & SCENE_ENCODING_CACHE_MISS_MATERIAL_PAINT_BUDGET_CHANGED,
            0
        );

        let mut next = base;
        next.material_distinct_budget_per_frame = 99;
        let reasons = miss_reasons_for_key_change(Some(base), next);
        assert_ne!(
            reasons & SCENE_ENCODING_CACHE_MISS_MATERIAL_DISTINCT_BUDGET_CHANGED,
            0
        );
    }

    #[test]
    fn record_scene_encoding_cache_frame_result_updates_perf_counters() {
        let base = base_key();
        let mut perf = RenderPerfStats::default();

        record_scene_encoding_cache_frame_result(
            Some(base),
            base,
            true,
            true,
            false,
            &tracing::Span::none(),
            &mut perf,
        );
        assert_eq!(perf.scene_encoding_cache_hits, 1);
        assert_eq!(perf.scene_encoding_cache_misses, 0);
        assert_eq!(perf.scene_encoding_cache_last_miss_reasons, 0);

        let mut next = base;
        next.text_quality_key = 42;
        record_scene_encoding_cache_frame_result(
            Some(base),
            next,
            false,
            true,
            false,
            &tracing::Span::none(),
            &mut perf,
        );
        assert_eq!(perf.scene_encoding_cache_hits, 1);
        assert_eq!(perf.scene_encoding_cache_misses, 1);
        assert_eq!(
            perf.scene_encoding_cache_last_miss_reasons,
            SCENE_ENCODING_CACHE_MISS_TEXT_QUALITY_KEY_CHANGED
        );
    }
}
