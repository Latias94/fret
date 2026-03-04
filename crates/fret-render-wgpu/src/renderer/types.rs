use bytemuck::{Pod, Zeroable};
use fret_core::geometry::Transform2D;
use fret_core::scene::MAX_STOPS;
use fret_core::scene::UvRect;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ClipRRectUniform {
    pub(super) rect: [f32; 4],
    pub(super) corner_radii: [f32; 4],
    pub(super) inv0: [f32; 4],
    pub(super) inv1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct MaskGradientUniform {
    /// Bounds in local pixel coordinates (x, y, w, h). Outside bounds, the mask is treated as 1.0.
    pub(super) bounds: [f32; 4],
    /// 1 = LinearGradient, 2 = RadialGradient, 3 = Image (coverage sampled from a renderer-bound texture).
    pub(super) kind: u32,
    /// For gradients: tile mode encoding. For image masks (kind=3): channel selector (0 = red, 1 = alpha).
    pub(super) tile_mode: u32,
    pub(super) stop_count: u32,
    pub(super) _pad0: u32,
    /// Linear: start.xy end.xy. Radial: center.xy radius.xy. Image: uv0.xy uv1.xy.
    pub(super) params0: [f32; 4],
    pub(super) inv0: [f32; 4],
    pub(super) inv1: [f32; 4],
    pub(super) stop_alphas0: [f32; 4],
    pub(super) stop_alphas1: [f32; 4],
    pub(super) stop_offsets0: [f32; 4],
    pub(super) stop_offsets1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ViewportUniform {
    pub(super) viewport_size: [f32; 2],
    pub(super) clip_head: u32,
    pub(super) clip_count: u32,
    pub(super) mask_head: u32,
    pub(super) mask_count: u32,
    /// Masks active at this scope boundary are excluded from draw shaders (applied later by
    /// composites). See ADR 0239.
    pub(super) mask_scope_head: u32,
    pub(super) mask_scope_count: u32,
    pub(super) output_is_srgb: u32,
    pub(super) _pad: u32,
    /// The viewport-space rect that mask textures are scoped to (top-left origin in pixels).
    ///
    /// For non-effect draws this is the full viewport. For effect scopes this is the effect
    /// scissor rect so viewport-scoped mask targets can be generated and sampled correctly.
    pub(super) mask_viewport_origin: [f32; 2],
    /// The viewport-space size of the rect that mask textures are scoped to (in pixels).
    pub(super) mask_viewport_size: [f32; 2],
    /// Padding to match WGSL uniform layout rules: `vec4<f32>` requires 16-byte alignment.
    pub(super) _pad_text_gamma: [u32; 2],

    /// Text gamma correction ratios (GPUI-aligned). Applied to grayscale coverage masks and
    /// subpixel RGB coverage in the text sampling shaders.
    pub(super) text_gamma_ratios: [f32; 4],
    /// Enhanced contrast factor for grayscale text (mask glyphs).
    pub(super) text_grayscale_enhanced_contrast: f32,
    /// Enhanced contrast factor for subpixel text (RGB coverage glyphs).
    pub(super) text_subpixel_enhanced_contrast: f32,
    pub(super) _pad_text_quality: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct RenderSpaceUniform {
    /// Absolute (root-scene) pixel origin that maps to framebuffer (0,0) for the current pass.
    pub(super) origin_px: [f32; 2],
    /// Framebuffer size in pixels for the current pass.
    pub(super) size_px: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ScaleParamsUniform {
    pub(super) scale: u32,
    pub(super) _pad0: u32,
    pub(super) src_origin: [u32; 2],
    pub(super) dst_origin: [u32; 2],
    pub(super) _pad1: u32,
    pub(super) _pad2: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PaintGpu {
    pub(super) kind: u32,
    pub(super) tile_mode: u32,
    pub(super) color_space: u32,
    pub(super) stop_count: u32,
    pub(super) eval_space: u32,
    pub(super) _pad_eval_space: [u32; 3],
    pub(super) params0: [f32; 4],
    pub(super) params1: [f32; 4],
    pub(super) params2: [f32; 4],
    pub(super) params3: [f32; 4],
    pub(super) stop_colors: [[f32; 4]; MAX_STOPS],
    pub(super) stop_offsets0: [f32; 4],
    pub(super) stop_offsets1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct QuadInstance {
    pub(super) rect: [f32; 4],
    pub(super) transform0: [f32; 4],
    pub(super) transform1: [f32; 4],
    pub(super) fill_paint: PaintGpu,
    pub(super) border_paint: PaintGpu,
    pub(super) corner_radii: [f32; 4],
    pub(super) border: [f32; 4],
    pub(super) dash_params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ViewportVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) uv: [f32; 2],
    pub(super) opacity: f32,
    pub(super) _pad: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct TextVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) local_pos_px: [f32; 2],
    pub(super) uv: [f32; 2],
    pub(super) color: [f32; 4],
    pub(super) outline_params: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PathVertex {
    pub(super) pos_px: [f32; 2],
    pub(super) local_pos_px: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ScissorRect {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) w: u32,
    pub(super) h: u32,
}

impl ScissorRect {
    pub(super) fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SvgRasterKind {
    AlphaMask,
    Rgba,
}

pub(super) const SVG_MASK_ATLAS_PAGE_SIZE_PX: u32 = 1024;
pub(super) const SVG_MASK_ATLAS_PADDING_PX: u32 = 1;

#[derive(Debug, Default, Clone, Copy)]
pub struct EffectDegradationCounters {
    pub requested: u64,
    pub applied: u64,
    pub degraded_budget_zero: u64,
    pub degraded_budget_insufficient: u64,
    pub degraded_target_exhausted: u64,
}

impl EffectDegradationCounters {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.requested = self.requested.saturating_add(other.requested);
        self.applied = self.applied.saturating_add(other.applied);
        self.degraded_budget_zero = self
            .degraded_budget_zero
            .saturating_add(other.degraded_budget_zero);
        self.degraded_budget_insufficient = self
            .degraded_budget_insufficient
            .saturating_add(other.degraded_budget_insufficient);
        self.degraded_target_exhausted = self
            .degraded_target_exhausted
            .saturating_add(other.degraded_target_exhausted);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CustomEffectV3SourceDegradationCounters {
    pub raw_requested: u64,
    pub raw_distinct: u64,
    pub raw_aliased_to_src: u64,

    pub pyramid_requested: u64,
    pub pyramid_applied_levels_ge2: u64,
    pub pyramid_degraded_to_one_budget_zero: u64,
    pub pyramid_degraded_to_one_budget_insufficient: u64,
}

impl CustomEffectV3SourceDegradationCounters {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.raw_requested = self.raw_requested.saturating_add(other.raw_requested);
        self.raw_distinct = self.raw_distinct.saturating_add(other.raw_distinct);
        self.raw_aliased_to_src = self
            .raw_aliased_to_src
            .saturating_add(other.raw_aliased_to_src);

        self.pyramid_requested = self
            .pyramid_requested
            .saturating_add(other.pyramid_requested);
        self.pyramid_applied_levels_ge2 = self
            .pyramid_applied_levels_ge2
            .saturating_add(other.pyramid_applied_levels_ge2);
        self.pyramid_degraded_to_one_budget_zero = self
            .pyramid_degraded_to_one_budget_zero
            .saturating_add(other.pyramid_degraded_to_one_budget_zero);
        self.pyramid_degraded_to_one_budget_insufficient = self
            .pyramid_degraded_to_one_budget_insufficient
            .saturating_add(other.pyramid_degraded_to_one_budget_insufficient);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BackdropSourceGroupDegradationCounters {
    pub requested: u64,
    pub applied_raw: u64,
    pub raw_degraded_budget_zero: u64,
    pub raw_degraded_budget_insufficient: u64,
    pub raw_degraded_target_exhausted: u64,

    pub pyramid_requested: u64,
    pub pyramid_applied_levels_ge2: u64,
    pub pyramid_degraded_to_one_budget_zero: u64,
    pub pyramid_degraded_to_one_budget_insufficient: u64,
    pub pyramid_skipped_raw_unavailable: u64,
}

impl BackdropSourceGroupDegradationCounters {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.requested = self.requested.saturating_add(other.requested);
        self.applied_raw = self.applied_raw.saturating_add(other.applied_raw);
        self.raw_degraded_budget_zero = self
            .raw_degraded_budget_zero
            .saturating_add(other.raw_degraded_budget_zero);
        self.raw_degraded_budget_insufficient = self
            .raw_degraded_budget_insufficient
            .saturating_add(other.raw_degraded_budget_insufficient);
        self.raw_degraded_target_exhausted = self
            .raw_degraded_target_exhausted
            .saturating_add(other.raw_degraded_target_exhausted);

        self.pyramid_requested = self
            .pyramid_requested
            .saturating_add(other.pyramid_requested);
        self.pyramid_applied_levels_ge2 = self
            .pyramid_applied_levels_ge2
            .saturating_add(other.pyramid_applied_levels_ge2);
        self.pyramid_degraded_to_one_budget_zero = self
            .pyramid_degraded_to_one_budget_zero
            .saturating_add(other.pyramid_degraded_to_one_budget_zero);
        self.pyramid_degraded_to_one_budget_insufficient = self
            .pyramid_degraded_to_one_budget_insufficient
            .saturating_add(other.pyramid_degraded_to_one_budget_insufficient);
        self.pyramid_skipped_raw_unavailable = self
            .pyramid_skipped_raw_unavailable
            .saturating_add(other.pyramid_skipped_raw_unavailable);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EffectDegradationSnapshot {
    pub gaussian_blur: EffectDegradationCounters,
    pub drop_shadow: EffectDegradationCounters,
    pub backdrop_warp: EffectDegradationCounters,
    pub color_adjust: EffectDegradationCounters,
    pub color_matrix: EffectDegradationCounters,
    pub alpha_threshold: EffectDegradationCounters,
    pub pixelate: EffectDegradationCounters,
    pub dither: EffectDegradationCounters,
    pub noise: EffectDegradationCounters,
    pub custom_effect: EffectDegradationCounters,
    pub custom_effect_v3_sources: CustomEffectV3SourceDegradationCounters,
    pub backdrop_source_groups: BackdropSourceGroupDegradationCounters,
}

impl EffectDegradationSnapshot {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.gaussian_blur
            .saturating_add_assign(other.gaussian_blur);
        self.drop_shadow.saturating_add_assign(other.drop_shadow);
        self.backdrop_warp
            .saturating_add_assign(other.backdrop_warp);
        self.color_adjust.saturating_add_assign(other.color_adjust);
        self.color_matrix.saturating_add_assign(other.color_matrix);
        self.alpha_threshold
            .saturating_add_assign(other.alpha_threshold);
        self.pixelate.saturating_add_assign(other.pixelate);
        self.dither.saturating_add_assign(other.dither);
        self.noise.saturating_add_assign(other.noise);
        self.custom_effect
            .saturating_add_assign(other.custom_effect);
        self.custom_effect_v3_sources
            .saturating_add_assign(other.custom_effect_v3_sources);
        self.backdrop_source_groups
            .saturating_add_assign(other.backdrop_source_groups);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BlurQualityCounters {
    pub applied: u64,
    pub applied_downsample_1: u64,
    pub applied_downsample_2: u64,
    pub applied_downsample_4: u64,
    pub applied_iterations_zero: u64,
    pub applied_iterations_sum: u64,
    pub applied_iterations_max: u64,
    /// Counts cases where the applied downsample scale differs from the desired scale.
    pub quality_degraded_downsample: u64,
    /// Counts cases where the requested blur was removed (hard shadow fallback).
    pub quality_degraded_blur_removed: u64,
}

impl BlurQualityCounters {
    pub(crate) fn record_applied(&mut self, downsample_scale: u32, iterations: u32, desired: u32) {
        self.applied = self.applied.saturating_add(1);
        match downsample_scale {
            1 => self.applied_downsample_1 = self.applied_downsample_1.saturating_add(1),
            2 => self.applied_downsample_2 = self.applied_downsample_2.saturating_add(1),
            4 => self.applied_downsample_4 = self.applied_downsample_4.saturating_add(1),
            _ => {}
        }
        self.applied_iterations_sum = self
            .applied_iterations_sum
            .saturating_add(u64::from(iterations));
        self.applied_iterations_max = self.applied_iterations_max.max(u64::from(iterations));
        if iterations == 0 {
            self.applied_iterations_zero = self.applied_iterations_zero.saturating_add(1);
        }
        if iterations != 0 && downsample_scale != desired {
            self.quality_degraded_downsample = self.quality_degraded_downsample.saturating_add(1);
        }
    }

    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.applied = self.applied.saturating_add(other.applied);
        self.applied_downsample_1 = self
            .applied_downsample_1
            .saturating_add(other.applied_downsample_1);
        self.applied_downsample_2 = self
            .applied_downsample_2
            .saturating_add(other.applied_downsample_2);
        self.applied_downsample_4 = self
            .applied_downsample_4
            .saturating_add(other.applied_downsample_4);
        self.applied_iterations_zero = self
            .applied_iterations_zero
            .saturating_add(other.applied_iterations_zero);
        self.applied_iterations_sum = self
            .applied_iterations_sum
            .saturating_add(other.applied_iterations_sum);
        self.applied_iterations_max = self
            .applied_iterations_max
            .max(other.applied_iterations_max);
        self.quality_degraded_downsample = self
            .quality_degraded_downsample
            .saturating_add(other.quality_degraded_downsample);
        self.quality_degraded_blur_removed = self
            .quality_degraded_blur_removed
            .saturating_add(other.quality_degraded_blur_removed);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BlurQualitySnapshot {
    pub gaussian_blur: BlurQualityCounters,
    pub drop_shadow: BlurQualityCounters,
}

impl BlurQualitySnapshot {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.gaussian_blur
            .saturating_add_assign(other.gaussian_blur);
        self.drop_shadow.saturating_add_assign(other.drop_shadow);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RenderPerfSnapshot {
    pub frames: u64,

    pub encode_scene_us: u64,
    pub ensure_pipelines_us: u64,
    pub plan_compile_us: u64,
    pub upload_us: u64,
    pub record_passes_us: u64,
    pub encoder_finish_us: u64,
    pub prepare_svg_us: u64,
    pub prepare_text_us: u64,

    // Non-text upload churn (best-effort). These counters attempt to make CPU->GPU texture uploads
    // visible in diagnostics, beyond text atlas updates.
    pub svg_uploads: u64,
    pub svg_upload_bytes: u64,
    pub image_uploads: u64,
    pub image_upload_bytes: u64,

    // Imported render targets (best-effort). These counters expose the declared ingestion strategy
    // for `RenderTargetId` updates applied before the UI render pass.
    pub render_target_updates_ingest_unknown: u64,
    pub render_target_updates_ingest_owned: u64,
    pub render_target_updates_ingest_external_zero_copy: u64,
    pub render_target_updates_ingest_gpu_copy: u64,
    pub render_target_updates_ingest_cpu_upload: u64,
    // Imported render targets (best-effort). Requested vs effective ingestion strategy is tracked
    // so capability-gated fallbacks become visible in perf/diagnostics snapshots.
    pub render_target_updates_requested_ingest_unknown: u64,
    pub render_target_updates_requested_ingest_owned: u64,
    pub render_target_updates_requested_ingest_external_zero_copy: u64,
    pub render_target_updates_requested_ingest_gpu_copy: u64,
    pub render_target_updates_requested_ingest_cpu_upload: u64,
    pub render_target_updates_ingest_fallbacks: u64,

    // Imported render targets (best-effort). Metadata degradations for strategies that cannot
    // preserve declared semantics.
    pub render_target_metadata_degradations_color_encoding_dropped: u64,

    // SVG raster cache (best-effort). These are intended to distinguish one-time warmup from
    // steady-state thrash (e.g. budget-driven eviction + repeated re-upload).
    pub svg_raster_budget_bytes: u64,
    pub svg_rasters_live: u64,
    pub svg_standalone_bytes_live: u64,
    pub svg_mask_atlas_pages_live: u64,
    pub svg_mask_atlas_bytes_live: u64,
    pub svg_mask_atlas_used_px: u64,
    pub svg_mask_atlas_capacity_px: u64,
    pub svg_raster_cache_hits: u64,
    pub svg_raster_cache_misses: u64,
    pub svg_raster_budget_evictions: u64,
    pub svg_mask_atlas_page_evictions: u64,
    pub svg_mask_atlas_entries_evicted: u64,

    // Text atlas churn (best-effort). These numbers are per-frame signals and should be treated as
    // diagnostic hints rather than strict correctness metrics.
    pub text_atlas_revision: u64,
    pub text_atlas_uploads: u64,
    pub text_atlas_upload_bytes: u64,
    pub text_atlas_evicted_glyphs: u64,
    pub text_atlas_evicted_pages: u64,
    pub text_atlas_evicted_page_glyphs: u64,
    pub text_atlas_resets: u64,

    // Intermediate pool churn (best-effort; used for blur/effect pipelines).
    pub intermediate_budget_bytes: u64,
    /// Estimated bytes required for a single full-viewport intermediate target at the current
    /// viewport size and format.
    ///
    /// This is intended for diagnostics / triage evidence (to interpret budget thresholds like
    /// “needs >= 2 full targets”) and should not be treated as a stable API surface.
    pub intermediate_full_target_bytes: u64,
    pub intermediate_in_use_bytes: u64,
    pub intermediate_peak_in_use_bytes: u64,
    pub intermediate_release_targets: u64,
    pub intermediate_pool_allocations: u64,
    pub intermediate_pool_reuses: u64,
    pub intermediate_pool_releases: u64,
    pub intermediate_pool_evictions: u64,
    pub intermediate_pool_free_bytes: u64,
    pub intermediate_pool_free_textures: u64,
    // GPU registry live resource estimates (best-effort).
    //
    // These are diagnostics-only estimates derived from descriptors the runner provides for
    // imported render targets and images. They may not match backend allocations exactly but are
    // useful for explaining large vmmap/driver footprint deltas.
    pub gpu_images_live: u64,
    pub gpu_images_bytes_estimate: u64,
    pub gpu_images_max_bytes_estimate: u64,
    pub gpu_render_targets_live: u64,
    pub gpu_render_targets_bytes_estimate: u64,
    pub gpu_render_targets_max_bytes_estimate: u64,
    pub render_plan_estimated_peak_intermediate_bytes: u64,
    pub render_plan_segments: u64,
    pub render_plan_segments_changed: u64,
    pub render_plan_segments_passes_increased: u64,
    pub render_plan_degradations: u64,
    /// Number of effect chain budget samples recorded during render plan compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_effect_chain_budget_samples: u64,
    /// Minimum effective intermediate budget observed across effect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_effect_chain_effective_budget_min_bytes: u64,
    /// Maximum effective intermediate budget observed across effect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_effect_chain_effective_budget_max_bytes: u64,
    /// Maximum "other live bytes" observed across effect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_effect_chain_other_live_max_bytes: u64,

    /// Number of CustomEffect chain budget samples recorded during render plan compilation.
    ///
    /// A "CustomEffect chain" is an effect chain that contains at least one CustomEffect step
    /// (v1/v2/v3). These are best-effort diagnostics signals (not a stable API).
    pub render_plan_custom_effect_chain_budget_samples: u64,
    /// Minimum effective intermediate budget observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_effective_budget_min_bytes: u64,
    /// Maximum effective intermediate budget observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_effective_budget_max_bytes: u64,
    /// Maximum "other live bytes" observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_other_live_max_bytes: u64,
    /// Maximum "base required bytes" observed across CustomEffect chains.
    ///
    /// Base required bytes are expressed as full-size intermediate targets for the chain
    /// (`srcdst` + required scratch/work/raw targets), excluding optional resources
    /// (mask/pyramid).
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_base_required_max_bytes: u64,
    /// Maximum "optional required bytes" observed across CustomEffect chains.
    ///
    /// Optional required bytes cover non-full intermediate allocations like clip masks and v3
    /// pyramids.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_optional_required_max_bytes: u64,
    /// Maximum full-size target count implied by "base required bytes" across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_base_required_full_targets_max: u32,
    /// Maximum clip-mask bytes observed across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_optional_mask_max_bytes: u64,
    /// Maximum pyramid bytes observed across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub render_plan_custom_effect_chain_optional_pyramid_max_bytes: u64,
    pub render_plan_degradations_budget_zero: u64,
    pub render_plan_degradations_budget_insufficient: u64,
    pub render_plan_degradations_target_exhausted: u64,
    pub render_plan_degradations_backdrop_noop: u64,
    pub render_plan_degradations_filter_content_disabled: u64,
    pub render_plan_degradations_clip_path_disabled: u64,
    pub render_plan_degradations_composite_group_blend_to_over: u64,
    pub effect_degradations: EffectDegradationSnapshot,
    pub effect_blur_quality: BlurQualitySnapshot,
    /// Counts `EffectStep::CustomV1` occurrences in requested effect chains for the frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v1_steps_requested: u64,
    /// Counts `RenderPlanPass::CustomEffect` emitted by the render plan compiler for the frame.
    ///
    /// When `custom_effect_v1_steps_requested > 0` but this remains `0`, CustomEffectV1 was
    /// requested but skipped (typically due to intermediate budget / target constraints).
    pub custom_effect_v1_passes_emitted: u64,
    /// Counts `EffectStep::CustomV2` occurrences in requested effect chains for the frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v2_steps_requested: u64,
    /// Counts `RenderPlanPass::CustomEffectV2` emitted by the render plan compiler for the frame.
    ///
    /// When `custom_effect_v2_steps_requested > 0` but this remains `0`, CustomEffectV2 was
    /// requested but skipped (typically due to intermediate budget / target constraints).
    pub custom_effect_v2_passes_emitted: u64,
    /// Counts CustomEffectV2 passes where an incompatible user image was provided and the backend
    /// bound the deterministic fallback instead (1x1 transparent).
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v2_user_image_incompatible_fallbacks: u64,
    /// Counts `EffectStep::CustomV3` occurrences in requested effect chains for the frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v3_steps_requested: u64,
    /// Counts `RenderPlanPass::CustomEffectV3` emitted by the render plan compiler for the frame.
    ///
    /// When `custom_effect_v3_steps_requested > 0` but this remains `0`, CustomEffectV3 was
    /// requested but skipped (typically due to intermediate budget / target constraints).
    pub custom_effect_v3_passes_emitted: u64,
    /// Counts CustomEffectV3 passes where an incompatible `user0` image was provided and the
    /// backend bound the deterministic fallback instead (1x1 transparent).
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v3_user0_image_incompatible_fallbacks: u64,
    /// Counts CustomEffectV3 passes where an incompatible `user1` image was provided and the
    /// backend bound the deterministic fallback instead (1x1 transparent).
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub custom_effect_v3_user1_image_incompatible_fallbacks: u64,
    pub custom_effect_v3_pyramid_cache_hits: u64,
    pub custom_effect_v3_pyramid_cache_misses: u64,

    pub clip_path_mask_cache_bytes_live: u64,
    pub clip_path_mask_cache_entries_live: u64,
    pub clip_path_mask_cache_hits: u64,
    pub clip_path_mask_cache_misses: u64,

    pub draw_calls: u64,
    pub quad_draw_calls: u64,
    pub viewport_draw_calls: u64,
    pub viewport_draw_calls_ingest_unknown: u64,
    pub viewport_draw_calls_ingest_owned: u64,
    pub viewport_draw_calls_ingest_external_zero_copy: u64,
    pub viewport_draw_calls_ingest_gpu_copy: u64,
    pub viewport_draw_calls_ingest_cpu_upload: u64,
    pub image_draw_calls: u64,
    pub text_draw_calls: u64,
    pub path_draw_calls: u64,
    pub mask_draw_calls: u64,
    pub fullscreen_draw_calls: u64,
    pub clip_mask_draw_calls: u64,

    pub pipeline_switches: u64,
    pub pipeline_switches_quad: u64,
    pub pipeline_switches_viewport: u64,
    pub pipeline_switches_mask: u64,
    pub pipeline_switches_text_mask: u64,
    pub pipeline_switches_text_color: u64,
    pub pipeline_switches_text_subpixel: u64,
    pub pipeline_switches_path: u64,
    pub pipeline_switches_path_msaa: u64,
    pub pipeline_switches_composite: u64,
    pub pipeline_switches_fullscreen: u64,
    pub pipeline_switches_clip_mask: u64,
    pub bind_group_switches: u64,
    pub uniform_bind_group_switches: u64,
    pub texture_bind_group_switches: u64,
    pub scissor_sets: u64,

    // Path MSAA observability (best-effort).
    pub path_msaa_samples_requested: u32,
    pub path_msaa_samples_effective: u32,
    // Counts frames where Vulkan path MSAA was requested but degraded to non-MSAA (e.g. opt-out
    // via env var). This is intended for diagnostics and may evolve over time.
    pub path_msaa_vulkan_safety_valve_degradations: u64,

    pub uniform_bytes: u64,
    pub instance_bytes: u64,
    pub vertex_bytes: u64,

    pub scene_encoding_cache_hits: u64,
    pub scene_encoding_cache_misses: u64,
    /// Best-effort miss reason mask for the most recent encoding cache miss.
    ///
    /// This is intended for diagnostics bundles and trace logs. It should not be treated as a
    /// stable API surface.
    pub scene_encoding_cache_last_miss_reasons: u64,

    // Tier B materials (ADR 0235) observability (best-effort).
    pub material_quad_ops: u64,
    pub material_sampled_quad_ops: u64,
    pub material_distinct: u64,
    pub material_unknown_ids: u64,
    pub material_degraded_due_to_budget: u64,

    // Path material paint degradations (best-effort).
    //
    // The wgpu path pipeline supports `Paint::Material`, but encoding may still deterministically
    // degrade a requested material paint (typically to the base solid color) if it cannot be
    // represented (unknown id, per-frame material budgets, or future capability gates).
    pub path_material_paints_degraded_to_solid_base: u64,
}

#[derive(Debug, Default)]
pub(super) struct RenderPerfStats {
    pub(super) frames: u64,

    pub(super) encode_scene: Duration,
    pub(super) ensure_pipelines: Duration,
    pub(super) plan_compile: Duration,
    pub(super) upload: Duration,
    pub(super) record_passes: Duration,
    pub(super) encoder_finish: Duration,
    pub(super) prepare_svg: Duration,
    pub(super) prepare_text: Duration,

    pub(super) svg_uploads: u64,
    pub(super) svg_upload_bytes: u64,
    pub(super) image_uploads: u64,
    pub(super) image_upload_bytes: u64,

    pub(super) render_target_updates_ingest_unknown: u64,
    pub(super) render_target_updates_ingest_owned: u64,
    pub(super) render_target_updates_ingest_external_zero_copy: u64,
    pub(super) render_target_updates_ingest_gpu_copy: u64,
    pub(super) render_target_updates_ingest_cpu_upload: u64,
    pub(super) render_target_updates_requested_ingest_unknown: u64,
    pub(super) render_target_updates_requested_ingest_owned: u64,
    pub(super) render_target_updates_requested_ingest_external_zero_copy: u64,
    pub(super) render_target_updates_requested_ingest_gpu_copy: u64,
    pub(super) render_target_updates_requested_ingest_cpu_upload: u64,
    pub(super) render_target_updates_ingest_fallbacks: u64,
    pub(super) render_target_metadata_degradations_color_encoding_dropped: u64,

    pub(super) svg_raster_budget_bytes: u64,
    pub(super) svg_rasters_live: u64,
    pub(super) svg_standalone_bytes_live: u64,
    pub(super) svg_mask_atlas_pages_live: u64,
    pub(super) svg_mask_atlas_bytes_live: u64,
    pub(super) svg_mask_atlas_used_px: u64,
    pub(super) svg_mask_atlas_capacity_px: u64,
    pub(super) svg_raster_cache_hits: u64,
    pub(super) svg_raster_cache_misses: u64,
    pub(super) svg_raster_budget_evictions: u64,
    pub(super) svg_mask_atlas_page_evictions: u64,
    pub(super) svg_mask_atlas_entries_evicted: u64,

    pub(super) text_atlas_revision: u64,
    pub(super) text_atlas_uploads: u64,
    pub(super) text_atlas_upload_bytes: u64,
    pub(super) text_atlas_evicted_glyphs: u64,
    pub(super) text_atlas_evicted_pages: u64,
    pub(super) text_atlas_evicted_page_glyphs: u64,
    pub(super) text_atlas_resets: u64,

    pub(super) intermediate_budget_bytes: u64,
    pub(super) intermediate_full_target_bytes: u64,
    pub(super) intermediate_in_use_bytes: u64,
    pub(super) intermediate_peak_in_use_bytes: u64,
    pub(super) intermediate_release_targets: u64,
    pub(super) intermediate_pool_allocations: u64,
    pub(super) intermediate_pool_reuses: u64,
    pub(super) intermediate_pool_releases: u64,
    pub(super) intermediate_pool_evictions: u64,
    pub(super) intermediate_pool_free_bytes: u64,
    pub(super) intermediate_pool_free_textures: u64,
    pub(super) render_plan_estimated_peak_intermediate_bytes: u64,
    pub(super) render_plan_segments: u64,
    pub(super) render_plan_segments_changed: u64,
    pub(super) render_plan_segments_passes_increased: u64,
    pub(super) render_plan_degradations: u64,
    pub(super) render_plan_effect_chain_budget_samples: u64,
    pub(super) render_plan_effect_chain_effective_budget_min_bytes: u64,
    pub(super) render_plan_effect_chain_effective_budget_max_bytes: u64,
    pub(super) render_plan_effect_chain_other_live_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_budget_samples: u64,
    pub(super) render_plan_custom_effect_chain_effective_budget_min_bytes: u64,
    pub(super) render_plan_custom_effect_chain_effective_budget_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_other_live_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_base_required_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_optional_required_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_base_required_full_targets_max: u32,
    pub(super) render_plan_custom_effect_chain_optional_mask_max_bytes: u64,
    pub(super) render_plan_custom_effect_chain_optional_pyramid_max_bytes: u64,
    pub(super) render_plan_degradations_budget_zero: u64,
    pub(super) render_plan_degradations_budget_insufficient: u64,
    pub(super) render_plan_degradations_target_exhausted: u64,
    pub(super) render_plan_degradations_backdrop_noop: u64,
    pub(super) render_plan_degradations_filter_content_disabled: u64,
    pub(super) render_plan_degradations_clip_path_disabled: u64,
    pub(super) render_plan_degradations_composite_group_blend_to_over: u64,
    pub(super) effect_degradations: EffectDegradationSnapshot,
    pub(super) effect_blur_quality: BlurQualitySnapshot,
    pub(super) custom_effect_v1_steps_requested: u64,
    pub(super) custom_effect_v1_passes_emitted: u64,
    pub(super) custom_effect_v2_steps_requested: u64,
    pub(super) custom_effect_v2_passes_emitted: u64,
    pub(super) custom_effect_v2_user_image_incompatible_fallbacks: u64,
    pub(super) custom_effect_v3_steps_requested: u64,
    pub(super) custom_effect_v3_passes_emitted: u64,
    pub(super) custom_effect_v3_user0_image_incompatible_fallbacks: u64,
    pub(super) custom_effect_v3_user1_image_incompatible_fallbacks: u64,
    pub(super) custom_effect_v3_pyramid_cache_hits: u64,
    pub(super) custom_effect_v3_pyramid_cache_misses: u64,

    pub(super) clip_path_mask_cache_bytes_live: u64,
    pub(super) clip_path_mask_cache_entries_live: u64,
    pub(super) clip_path_mask_cache_hits: u64,
    pub(super) clip_path_mask_cache_misses: u64,

    pub(super) draw_calls: u64,
    pub(super) quad_draw_calls: u64,
    pub(super) viewport_draw_calls: u64,
    pub(super) viewport_draw_calls_ingest_unknown: u64,
    pub(super) viewport_draw_calls_ingest_owned: u64,
    pub(super) viewport_draw_calls_ingest_external_zero_copy: u64,
    pub(super) viewport_draw_calls_ingest_gpu_copy: u64,
    pub(super) viewport_draw_calls_ingest_cpu_upload: u64,
    pub(super) image_draw_calls: u64,
    pub(super) text_draw_calls: u64,
    pub(super) path_draw_calls: u64,
    pub(super) mask_draw_calls: u64,
    pub(super) fullscreen_draw_calls: u64,
    pub(super) clip_mask_draw_calls: u64,

    pub(super) pipeline_switches: u64,
    pub(super) pipeline_switches_quad: u64,
    pub(super) pipeline_switches_viewport: u64,
    pub(super) pipeline_switches_mask: u64,
    pub(super) pipeline_switches_text_mask: u64,
    pub(super) pipeline_switches_text_color: u64,
    pub(super) pipeline_switches_text_subpixel: u64,
    pub(super) pipeline_switches_path: u64,
    pub(super) pipeline_switches_path_msaa: u64,
    pub(super) pipeline_switches_composite: u64,
    pub(super) pipeline_switches_fullscreen: u64,
    pub(super) pipeline_switches_clip_mask: u64,
    pub(super) bind_group_switches: u64,
    pub(super) uniform_bind_group_switches: u64,
    pub(super) texture_bind_group_switches: u64,
    pub(super) scissor_sets: u64,

    pub(super) path_msaa_samples_requested: u32,
    pub(super) path_msaa_samples_effective: u32,
    pub(super) path_msaa_vulkan_safety_valve_degradations: u64,

    pub(super) uniform_bytes: u64,
    pub(super) instance_bytes: u64,
    pub(super) vertex_bytes: u64,

    pub(super) scene_encoding_cache_hits: u64,
    pub(super) scene_encoding_cache_misses: u64,
    pub(super) scene_encoding_cache_last_miss_reasons: u64,

    pub(super) material_quad_ops: u64,
    pub(super) material_sampled_quad_ops: u64,
    pub(super) material_distinct: u64,
    pub(super) material_unknown_ids: u64,
    pub(super) material_degraded_due_to_budget: u64,

    pub(super) path_material_paints_degraded_to_solid_base: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SvgPerfSnapshot {
    pub frames: u64,
    pub prepare_svg_ops_us: u64,

    pub cache_hits: u64,
    pub cache_misses: u64,

    pub alpha_raster_count: u64,
    pub alpha_raster_us: u64,
    pub rgba_raster_count: u64,
    pub rgba_raster_us: u64,

    pub alpha_atlas_inserts: u64,
    pub alpha_atlas_write_us: u64,
    pub alpha_standalone_uploads: u64,
    pub alpha_standalone_upload_us: u64,
    pub rgba_uploads: u64,
    pub rgba_upload_us: u64,

    pub atlas_pages_live: usize,
    pub svg_rasters_live: usize,
    pub svg_standalone_bytes_live: u64,
    pub svg_mask_atlas_bytes_live: u64,
    pub svg_mask_atlas_used_px: u64,
    pub svg_mask_atlas_capacity_px: u64,
}

#[derive(Debug, Default)]
pub(super) struct SvgPerfStats {
    pub(super) frames: u64,
    pub(super) prepare_svg_ops: Duration,

    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,

    pub(super) alpha_raster_count: u64,
    pub(super) alpha_raster: Duration,
    pub(super) rgba_raster_count: u64,
    pub(super) rgba_raster: Duration,

    pub(super) alpha_atlas_inserts: u64,
    pub(super) alpha_atlas_write: Duration,
    pub(super) alpha_standalone_uploads: u64,
    pub(super) alpha_standalone_upload: Duration,
    pub(super) rgba_uploads: u64,
    pub(super) rgba_upload: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IntermediatePerfSnapshot {
    pub frames: u64,
    pub budget_bytes: u64,

    pub last_frame_in_use_bytes: u64,
    pub last_frame_peak_in_use_bytes: u64,
    pub last_frame_release_targets: u64,
    pub blur_degraded_to_quarter: u64,
    pub blur_disabled_due_to_budget: u64,
    pub pool_free_bytes: u64,
    pub pool_free_textures: u64,

    pub pool_allocations: u64,
    pub pool_reuses: u64,
    pub pool_releases: u64,
    pub pool_evictions: u64,
}

#[derive(Debug, Default)]
pub(super) struct IntermediatePerfStats {
    pub(super) frames: u64,
    pub(super) last_frame_in_use_bytes: u64,
    pub(super) last_frame_peak_in_use_bytes: u64,
    pub(super) last_frame_release_targets: u64,
    pub(super) blur_degraded_to_quarter: u64,
    pub(super) blur_disabled_due_to_budget: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SvgRasterKey {
    pub(super) svg: fret_core::SvgId,
    pub(super) target_w: u32,
    pub(super) target_h: u32,
    pub(super) smooth_scale_bits: u32,
    pub(super) kind: SvgRasterKind,
    pub(super) fit: fret_core::SvgFit,
}

pub(super) enum SvgRasterStorage {
    Standalone {
        _texture: wgpu::Texture,
    },
    MaskAtlas {
        page_index: usize,
        alloc_id: etagere::AllocId,
    },
}

pub(super) struct SvgMaskAtlasPage {
    pub(super) image: fret_core::ImageId,
    pub(super) size_px: (u32, u32),
    pub(super) allocator: etagere::BucketedAtlasAllocator,
    pub(super) entries: usize,
    pub(super) last_used_epoch: u64,
    pub(super) _texture: wgpu::Texture,
}

pub(super) struct SvgRasterEntry {
    pub(super) image: fret_core::ImageId,
    pub(super) uv: UvRect,
    pub(super) size_px: (u32, u32),
    pub(super) approx_bytes: u64,
    pub(super) last_used_epoch: u64,
    pub(super) storage: SvgRasterStorage,
}

#[derive(Debug, Clone)]
pub(super) struct SvgEntry {
    pub(super) bytes: Arc<[u8]>,
    pub(super) refs: u32,
}

impl SvgMaskAtlasPage {
    pub(super) fn bytes(&self) -> u64 {
        u64::from(self.size_px.0).saturating_mul(u64::from(self.size_px.1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct QuadPipelineKey {
    pub(super) fill_kind: u8,
    pub(super) border_kind: u8,
    pub(super) border_present: bool,
    pub(super) dash_enabled: bool,
    pub(super) fill_material_sampled: bool,
    pub(super) border_material_sampled: bool,
}

#[derive(Clone, Copy)]
pub(super) struct QuadDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_instance: u32,
    pub(super) instance_count: u32,
    pub(super) pipeline: QuadPipelineKey,
}

pub(super) struct ViewportDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) target: fret_core::RenderTargetId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UniformMaskImageSelection {
    pub(super) image: fret_core::ImageId,
    pub(super) sampling: fret_core::scene::ImageSamplingHint,
}

#[derive(Clone, Copy)]
pub(super) enum ClipPop {
    NoShader,
    Shader { prev_head: u32 },
    Path,
}

#[derive(Clone, Copy)]
pub(super) enum MaskPop {
    NoShader,
    Shader {
        prev_head: u32,
        prev_mask_image: Option<UniformMaskImageSelection>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct ImageDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) image: fret_core::ImageId,
    pub(super) sampling: fret_core::scene::ImageSamplingHint,
}

#[derive(Clone, Copy)]
pub(super) struct MaskDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) image: fret_core::ImageId,
    pub(super) sampling: fret_core::scene::ImageSamplingHint,
}

pub(super) struct TextDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) kind: TextDrawKind,
    pub(super) atlas_page: u16,
    pub(super) paint_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextDrawKind {
    Mask,
    MaskOutline,
    Color,
    Subpixel,
    SubpixelOutline,
}

#[derive(Clone, Copy)]
pub(super) struct PathDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) paint_index: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ClipPathMaskDraw {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) cache_key: u64,
}

pub(super) struct PathIntermediate {
    pub(super) size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) sample_count: u32,
    pub(super) _msaa_texture: Option<wgpu::Texture>,
    pub(super) msaa_view: Option<wgpu::TextureView>,
    pub(super) _resolved_texture: wgpu::Texture,
    pub(super) resolved_view: wgpu::TextureView,
    pub(super) bind_group: wgpu::BindGroup,
}

pub(super) enum OrderedDraw {
    Quad(QuadDraw),
    Viewport(ViewportDraw),
    Image(ImageDraw),
    Mask(MaskDraw),
    Text(TextDraw),
    Path(PathDraw),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum EffectMarkerKind {
    Push {
        scissor: ScissorRect,
        uniform_index: u32,
        mode: fret_core::EffectMode,
        chain: fret_core::EffectChain,
        quality: fret_core::EffectQuality,
    },
    Pop,
    BackdropSourceGroupPush {
        scissor: ScissorRect,
        pyramid: Option<fret_core::scene::CustomEffectPyramidRequestV1>,
        quality: fret_core::EffectQuality,
    },
    BackdropSourceGroupPop,
    ClipPathPush {
        scissor: ScissorRect,
        uniform_index: u32,
        mask_draw_index: u32,
    },
    ClipPathPop,
    CompositeGroupPush {
        scissor: ScissorRect,
        uniform_index: u32,
        mode: fret_core::BlendMode,
        quality: fret_core::EffectQuality,
        opacity: f32,
    },
    CompositeGroupPop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EffectMarker {
    pub(super) draw_ix: usize,
    pub(super) kind: EffectMarkerKind,
}

#[derive(Default)]
pub(super) struct SceneEncoding {
    pub(super) instances: Vec<QuadInstance>,
    pub(super) path_paints: Vec<PaintGpu>,
    pub(super) text_paints: Vec<PaintGpu>,
    pub(super) viewport_vertices: Vec<ViewportVertex>,
    pub(super) text_vertices: Vec<TextVertex>,
    pub(super) path_vertices: Vec<PathVertex>,
    pub(super) clip_path_masks: Vec<ClipPathMaskDraw>,
    pub(super) clips: Vec<ClipRRectUniform>,
    pub(super) masks: Vec<MaskGradientUniform>,
    pub(super) uniforms: Vec<ViewportUniform>,
    /// Per-uniform CPU-side mask-image selection used to pick the correct bind group for `Mask::Image`.
    pub(super) uniform_mask_images: Vec<Option<UniformMaskImageSelection>>,
    pub(super) ordered_draws: Vec<OrderedDraw>,
    pub(super) effect_markers: Vec<EffectMarker>,

    pub(super) encode_scissor_stack_scratch: Vec<ScissorRect>,
    pub(super) encode_clip_pop_stack_scratch: Vec<ClipPop>,
    pub(super) encode_mask_pop_stack_scratch: Vec<MaskPop>,
    pub(super) encode_mask_scope_stack_scratch: Vec<(u32, u32)>,
    pub(super) encode_transform_stack_scratch: Vec<Transform2D>,
    pub(super) encode_opacity_stack_scratch: Vec<f32>,
    pub(super) encode_material_seen_scratch: HashSet<fret_core::MaterialId>,

    pub(super) material_quad_ops: u64,
    pub(super) material_sampled_quad_ops: u64,
    pub(super) material_distinct: u64,
    pub(super) material_unknown_ids: u64,
    pub(super) material_degraded_due_to_budget: u64,

    pub(super) path_material_paints_degraded_to_solid_base: u64,
}

impl SceneEncoding {
    pub(super) fn clear(&mut self) {
        self.instances.clear();
        self.path_paints.clear();
        self.text_paints.clear();
        self.viewport_vertices.clear();
        self.text_vertices.clear();
        self.path_vertices.clear();
        self.clip_path_masks.clear();
        self.clips.clear();
        self.masks.clear();
        self.uniforms.clear();
        self.uniform_mask_images.clear();
        self.ordered_draws.clear();
        self.effect_markers.clear();
        self.encode_scissor_stack_scratch.clear();
        self.encode_clip_pop_stack_scratch.clear();
        self.encode_mask_pop_stack_scratch.clear();
        self.encode_mask_scope_stack_scratch.clear();
        self.encode_transform_stack_scratch.clear();
        self.encode_opacity_stack_scratch.clear();
        self.encode_material_seen_scratch.clear();
        self.material_quad_ops = 0;
        self.material_sampled_quad_ops = 0;
        self.material_distinct = 0;
        self.material_unknown_ids = 0;
        self.material_degraded_due_to_budget = 0;
        self.path_material_paints_degraded_to_solid_base = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SceneEncodingCacheKey {
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
