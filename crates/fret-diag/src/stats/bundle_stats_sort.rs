#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum BundleStatsSort {
    #[default]
    Invalidation,
    Time,
    UiThreadCpuTime,
    UiThreadCpuCycles,
    Dispatch,
    HitTest,
    RendererEncodeScene,
    RendererEnsurePipelines,
    RendererPlanCompile,
    RendererUpload,
    RendererRecordPasses,
    RendererEncoderFinish,
    RendererPrepareText,
    RendererDrawCalls,
    RendererPipelineSwitches,
    RendererBindGroupSwitches,
    RendererTextAtlasUploadBytes,
    RendererTextAtlasEvictedPages,
    RendererSvgUploadBytes,
    RendererImageUploadBytes,
    RendererSvgRasterCacheMisses,
    RendererSvgRasterBudgetEvictions,
    RendererIntermediateBudgetBytes,
    RendererIntermediateInUseBytes,
    RendererIntermediatePeakInUseBytes,
    RendererIntermediateReleaseTargets,
    RendererIntermediatePoolAllocations,
    RendererIntermediatePoolReuses,
    RendererIntermediatePoolReleases,
    RendererIntermediatePoolEvictions,
    RendererIntermediatePoolFreeBytes,
    RendererIntermediatePoolFreeTextures,
}

impl BundleStatsSort {
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "invalidation" => Ok(Self::Invalidation),
            "time" => Ok(Self::Time),
            "cpu_time" | "cpu_us" | "ui_thread_cpu_time" => Ok(Self::UiThreadCpuTime),
            "cpu_cycles" | "cycles" | "ui_thread_cpu_cycles" => Ok(Self::UiThreadCpuCycles),
            "dispatch" => Ok(Self::Dispatch),
            "hit_test" => Ok(Self::HitTest),
            "encode_scene" | "encode" | "renderer_encode_scene" => Ok(Self::RendererEncodeScene),
            "ensure_pipelines" | "ensure" | "renderer_ensure_pipelines" => {
                Ok(Self::RendererEnsurePipelines)
            }
            "plan_compile" | "plan" | "renderer_plan_compile" => Ok(Self::RendererPlanCompile),
            "upload" | "uploads" | "renderer_upload" => Ok(Self::RendererUpload),
            "record_passes" | "record" | "renderer_record_passes" => Ok(Self::RendererRecordPasses),
            "encoder_finish" | "finish" | "renderer_encoder_finish" => {
                Ok(Self::RendererEncoderFinish)
            }
            "prepare_text" | "renderer_prepare_text" => Ok(Self::RendererPrepareText),
            "draw_calls" | "draws" | "renderer_draw_calls" => Ok(Self::RendererDrawCalls),
            "pipeline_switches" | "pipelines" | "renderer_pipeline_switches" => {
                Ok(Self::RendererPipelineSwitches)
            }
            "bind_group_switches" | "binds" | "renderer_bind_group_switches" => {
                Ok(Self::RendererBindGroupSwitches)
            }
            "atlas_upload_bytes" | "text_atlas_upload_bytes" | "renderer_text_atlas_upload_bytes" => {
                Ok(Self::RendererTextAtlasUploadBytes)
            }
            "atlas_evicted_pages"
            | "text_atlas_evicted_pages"
            | "renderer_text_atlas_evicted_pages" => Ok(Self::RendererTextAtlasEvictedPages),
            "svg_upload_bytes" | "renderer_svg_upload_bytes" => Ok(Self::RendererSvgUploadBytes),
            "image_upload_bytes" | "renderer_image_upload_bytes" => Ok(Self::RendererImageUploadBytes),
            "svg_cache_misses"
            | "svg_raster_cache_misses"
            | "renderer_svg_raster_cache_misses" => Ok(Self::RendererSvgRasterCacheMisses),
            "svg_evictions"
            | "svg_raster_budget_evictions"
            | "renderer_svg_raster_budget_evictions" => Ok(Self::RendererSvgRasterBudgetEvictions),
            "intermediate_budget_bytes" | "intermediate_budget" | "renderer_intermediate_budget_bytes" => {
                Ok(Self::RendererIntermediateBudgetBytes)
            }
            "intermediate_in_use_bytes" | "intermediate_in_use" | "renderer_intermediate_in_use_bytes" => {
                Ok(Self::RendererIntermediateInUseBytes)
            }
            "intermediate_peak_bytes" | "intermediate_peak" | "renderer_intermediate_peak_in_use_bytes" => {
                Ok(Self::RendererIntermediatePeakInUseBytes)
            }
            "intermediate_release_targets" | "renderer_intermediate_release_targets" => {
                Ok(Self::RendererIntermediateReleaseTargets)
            }
            "intermediate_allocations"
            | "intermediate_pool_allocations"
            | "renderer_intermediate_pool_allocations" => Ok(Self::RendererIntermediatePoolAllocations),
            "intermediate_reuses" | "intermediate_pool_reuses" | "renderer_intermediate_pool_reuses" => {
                Ok(Self::RendererIntermediatePoolReuses)
            }
            "intermediate_releases" | "intermediate_pool_releases" | "renderer_intermediate_pool_releases" => {
                Ok(Self::RendererIntermediatePoolReleases)
            }
            "pool_evictions" | "intermediate_pool_evictions" | "renderer_intermediate_pool_evictions" => {
                Ok(Self::RendererIntermediatePoolEvictions)
            }
            "intermediate_free_bytes"
            | "intermediate_pool_free_bytes"
            | "renderer_intermediate_pool_free_bytes" => Ok(Self::RendererIntermediatePoolFreeBytes),
            "intermediate_free_textures"
            | "intermediate_pool_free_textures"
            | "renderer_intermediate_pool_free_textures" => {
                Ok(Self::RendererIntermediatePoolFreeTextures)
            }
            other => Err(format!(
                "invalid --sort value: {other} (expected: invalidation|time|cpu_time|cpu_cycles|dispatch|hit_test|encode_scene|ensure_pipelines|plan_compile|upload|record_passes|encoder_finish|prepare_text|draw_calls|pipeline_switches|bind_group_switches|atlas_upload_bytes|atlas_evicted_pages|svg_upload_bytes|image_upload_bytes|svg_cache_misses|svg_evictions|intermediate_budget_bytes|intermediate_in_use_bytes|intermediate_peak_bytes|intermediate_release_targets|intermediate_allocations|intermediate_reuses|intermediate_releases|pool_evictions|intermediate_free_bytes|intermediate_free_textures)"
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Invalidation => "invalidation",
            Self::Time => "time",
            Self::UiThreadCpuTime => "cpu_time",
            Self::UiThreadCpuCycles => "cpu_cycles",
            Self::Dispatch => "dispatch",
            Self::HitTest => "hit_test",
            Self::RendererEncodeScene => "encode_scene",
            Self::RendererEnsurePipelines => "ensure_pipelines",
            Self::RendererPlanCompile => "plan_compile",
            Self::RendererUpload => "upload",
            Self::RendererRecordPasses => "record_passes",
            Self::RendererEncoderFinish => "encoder_finish",
            Self::RendererPrepareText => "prepare_text",
            Self::RendererDrawCalls => "draw_calls",
            Self::RendererPipelineSwitches => "pipeline_switches",
            Self::RendererBindGroupSwitches => "bind_group_switches",
            Self::RendererTextAtlasUploadBytes => "atlas_upload_bytes",
            Self::RendererTextAtlasEvictedPages => "atlas_evicted_pages",
            Self::RendererSvgUploadBytes => "svg_upload_bytes",
            Self::RendererImageUploadBytes => "image_upload_bytes",
            Self::RendererSvgRasterCacheMisses => "svg_cache_misses",
            Self::RendererSvgRasterBudgetEvictions => "svg_evictions",
            Self::RendererIntermediateBudgetBytes => "intermediate_budget_bytes",
            Self::RendererIntermediateInUseBytes => "intermediate_in_use_bytes",
            Self::RendererIntermediatePeakInUseBytes => "intermediate_peak_bytes",
            Self::RendererIntermediateReleaseTargets => "intermediate_release_targets",
            Self::RendererIntermediatePoolAllocations => "intermediate_allocations",
            Self::RendererIntermediatePoolReuses => "intermediate_reuses",
            Self::RendererIntermediatePoolReleases => "intermediate_releases",
            Self::RendererIntermediatePoolEvictions => "pool_evictions",
            Self::RendererIntermediatePoolFreeBytes => "intermediate_free_bytes",
            Self::RendererIntermediatePoolFreeTextures => "intermediate_free_textures",
        }
    }
}
