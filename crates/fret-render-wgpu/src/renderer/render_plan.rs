use super::{SceneEncoding, ScissorRect};
use std::ops::Range;

mod analysis;
mod debug;
mod postprocess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneSegmentId(pub(super) usize);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct RenderPlanSegmentFlags {
    pub(super) has_quad: bool,
    pub(super) has_vertex_color: bool,
    pub(super) has_viewport: bool,
    pub(super) has_image: bool,
    pub(super) has_mask: bool,
    pub(super) has_text: bool,
    pub(super) has_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RenderPlanSegment {
    pub(super) id: SceneSegmentId,
    pub(super) draw_range: Range<usize>,
    pub(super) start_uniform_index: Option<u32>,
    pub(super) start_uniform_fingerprint: u64,
    pub(super) flags: RenderPlanSegmentFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct RenderPlanCompileStats {
    pub(super) estimated_peak_intermediate_bytes: u64,
    pub(super) degradation_count: u64,
    pub(super) effect_degradations: super::EffectDegradationSnapshot,
    pub(super) effect_blur_quality: super::BlurQualitySnapshot,
    /// Counts how many effect chains were compiled through `apply_chain_in_place` for the frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) effect_chain_budget_samples: u64,
    /// Minimum effective intermediate budget observed across effect chain compilation for the
    /// frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) effect_chain_effective_budget_min_bytes: u64,
    /// Maximum effective intermediate budget observed across effect chain compilation for the
    /// frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) effect_chain_effective_budget_max_bytes: u64,
    /// Maximum "other live bytes" observed while compiling effect chains for the frame.
    ///
    /// This represents intermediate bytes that were effectively unavailable due to other live
    /// targets and reserved bytes (clip path masks, backdrop source groups), excluding `srcdst`.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) effect_chain_other_live_max_bytes: u64,

    /// Counts how many effect chains containing at least one CustomEffect step (v1/v2/v3) were
    /// compiled through `apply_chain_in_place` for the frame.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_budget_samples: u64,
    /// Minimum effective intermediate budget observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_effective_budget_min_bytes: u64,
    /// Maximum effective intermediate budget observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_effective_budget_max_bytes: u64,
    /// Maximum "other live bytes" observed across CustomEffect chain compilation.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_other_live_max_bytes: u64,
    /// Maximum "base required bytes" observed across CustomEffect chain compilation.
    ///
    /// Base required bytes are expressed as full-size intermediate targets for the chain
    /// (`srcdst` + required scratch/work/raw targets), excluding optional resources (mask/pyramid).
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_base_required_max_bytes: u64,
    /// Maximum "optional required bytes" observed across CustomEffect chain compilation.
    ///
    /// Optional required bytes cover non-full intermediate allocations like clip masks and v3
    /// pyramids.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_optional_required_max_bytes: u64,
    /// Maximum full-size target count implied by "base required bytes" across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_base_required_full_targets_max: u32,
    /// Maximum clip-mask bytes observed across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_optional_mask_max_bytes: u64,
    /// Maximum pyramid bytes observed across CustomEffect chains.
    ///
    /// This is a best-effort diagnostics signal (not a stable API).
    pub(super) custom_effect_chain_optional_pyramid_max_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderPlanDegradationReason {
    BudgetZero,
    BudgetInsufficient,
    TargetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderPlanDegradationKind {
    BackdropEffectNoOp,
    FilterContentDisabled,
    ClipPathDisabled,
    CompositeGroupBlendDegradedToOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RenderPlanDegradation {
    pub(super) draw_ix: usize,
    pub(super) kind: RenderPlanDegradationKind,
    pub(super) reason: RenderPlanDegradationReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanTarget {
    Output,
    Intermediate0,
    Intermediate1,
    Intermediate2,
    Intermediate3,
    Mask0,
    Mask1,
    Mask2,
}

pub(super) fn output_requires_explicit_srgb_encode(format: wgpu::TextureFormat) -> bool {
    if format.is_srgb() {
        return false;
    }
    matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MaskRef {
    pub(super) target: PlanTarget,
    pub(super) size: (u32, u32),
    pub(super) viewport_rect: ScissorRect,
}

/// A scissor rect in render-space, relative to the output viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AbsoluteScissorRect(pub(super) ScissorRect);

/// A scissor rect in the destination texture's local space (`0..dst_size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LocalScissorRect(pub(super) ScissorRect);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DebugPostprocess {
    None,
    OffscreenBlit {
        src: PlanTarget,
    },
    Pixelate {
        scale: u32,
    },
    Blur {
        radius: u32,
        downsample_scale: u32,
        scissor: Option<ScissorRect>,
    },
}

#[derive(Debug)]
pub(super) struct SceneDrawRangePass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) target_origin: (u32, u32),
    pub(super) target_size: (u32, u32),
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
    pub(super) draw_range: Range<usize>,
}

#[derive(Debug)]
pub(super) enum RenderPlanPass {
    SceneDrawRange(SceneDrawRangePass),
    PathMsaaBatch(PathMsaaBatchPass),
    PathClipMask(PathClipMaskPass),
    FullscreenBlit(FullscreenBlitPass),
    CompositePremul(CompositePremulPass),
    ScaleNearest(ScaleNearestPass),
    Blur(BlurPass),
    BackdropWarp(BackdropWarpPass),
    ColorAdjust(ColorAdjustPass),
    ColorMatrix(ColorMatrixPass),
    AlphaThreshold(AlphaThresholdPass),
    Dither(DitherPass),
    Noise(NoisePass),
    DropShadow(DropShadowPass),
    CustomEffect(CustomEffectPass),
    CustomEffectV2(CustomEffectV2Pass),
    CustomEffectV3(CustomEffectV3Pass),
    ClipMask(ClipMaskPass),
    ReleaseTarget(PlanTarget),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PathClipMaskPass {
    pub(super) dst: PlanTarget,
    pub(super) dst_origin: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) scissor: AbsoluteScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) cache_key: u64,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ClipMaskPass {
    pub(super) dst: PlanTarget,
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) uniform_index: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlurAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BlurPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) axis: BlurAxis,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BackdropWarpPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) origin_px: (u32, u32),
    pub(super) bounds_size_px: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) strength_px: f32,
    pub(super) scale_px: f32,
    pub(super) phase: f32,
    pub(super) chromatic_aberration_px: f32,
    pub(super) kind: fret_core::scene::BackdropWarpKindV1,
    pub(super) warp_image: Option<fret_core::ImageId>,
    pub(super) warp_uv: fret_core::scene::UvRect,
    pub(super) warp_sampling: fret_core::scene::ImageSamplingHint,
    pub(super) warp_encoding: fret_core::scene::WarpMapEncodingV1,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ColorAdjustPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) saturation: f32,
    pub(super) brightness: f32,
    pub(super) contrast: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ColorMatrixPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) matrix: [f32; 20],
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AlphaThresholdPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) cutoff: f32,
    pub(super) soft: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DitherPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) mode: fret_core::DitherMode,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NoisePass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) strength: f32,
    pub(super) scale_px: f32,
    pub(super) phase: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DropShadowPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) offset_px: (f32, f32),
    pub(super) color: fret_core::scene::Color,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CustomEffectPassCommon {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) effect: fret_core::EffectId,
    pub(super) params: fret_core::EffectParamsV1,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CustomEffectPass {
    pub(super) common: CustomEffectPassCommon,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CustomEffectV2Pass {
    pub(super) common: CustomEffectPassCommon,
    pub(super) input_image: Option<fret_core::ImageId>,
    pub(super) input_uv: fret_core::scene::UvRect,
    pub(super) input_sampling: fret_core::scene::ImageSamplingHint,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CustomEffectV3Pass {
    pub(super) src_raw: PlanTarget,
    pub(super) src_pyramid: PlanTarget,
    pub(super) pyramid_levels: u32,
    /// Optional ROI scissor for building the `src_pyramid` scratch (level 0 is in `src_size`
    /// space). When present, the renderer may restrict pyramid generation work to the scissor (and
    /// its downsampled projections) instead of building a full-viewport pyramid.
    pub(super) pyramid_build_scissor: Option<LocalScissorRect>,
    pub(super) raw_wanted: bool,
    pub(super) pyramid_wanted: bool,
    pub(super) common: CustomEffectPassCommon,
    pub(super) user0_image: Option<fret_core::ImageId>,
    pub(super) user0_uv: fret_core::scene::UvRect,
    pub(super) user0_sampling: fret_core::scene::ImageSamplingHint,
    pub(super) user1_image: Option<fret_core::ImageId>,
    pub(super) user1_uv: fret_core::scene::UvRect,
    pub(super) user1_sampling: fret_core::scene::ImageSamplingHint,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FullscreenBlitPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    /// Apply an explicit linear->sRGB transfer to the output (premul-aware).
    ///
    /// This must only be used for the final write to a non-sRGB display surface format (see ADR
    /// 0040 / ADR 0117).
    pub(super) encode_output_srgb: bool,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CompositePremulPass {
    pub(super) src: PlanTarget,
    pub(super) src_origin: (u32, u32),
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_origin: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<AbsoluteScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) blend_mode: fret_core::BlendMode,
    pub(super) opacity: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScaleMode {
    Downsample,
    Upscale,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ScaleNearestPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) src_origin: (u32, u32),
    pub(super) dst_scissor: Option<LocalScissorRect>,
    pub(super) dst_origin: (u32, u32),
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) mode: ScaleMode,
    pub(super) scale: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone)]
pub(super) struct PathMsaaBatchPass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) target_origin: (u32, u32),
    pub(super) target_size: (u32, u32),
    pub(super) draw_range: Range<usize>,
    pub(super) union_scissor: AbsoluteScissorRect,
    pub(super) batch_uniform_index: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub(super) segments: Vec<RenderPlanSegment>,
    pub(super) passes: Vec<RenderPlanPass>,
    pub(super) compile_stats: RenderPlanCompileStats,
    pub(super) degradations: Vec<RenderPlanDegradation>,
}

impl RenderPlan {
    pub(super) fn finalize(
        segments: Vec<RenderPlanSegment>,
        passes: Vec<RenderPlanPass>,
        viewport_size: (u32, u32),
        postprocess: DebugPostprocess,
        clear: wgpu::Color,
        format: wgpu::TextureFormat,
        degradations: Vec<RenderPlanDegradation>,
        effect_degradations: super::EffectDegradationSnapshot,
        effect_blur_quality: super::BlurQualitySnapshot,
    ) -> Self {
        let mut plan = Self {
            segments,
            passes,
            compile_stats: RenderPlanCompileStats {
                degradation_count: degradations.len() as u64,
                effect_degradations,
                effect_blur_quality,
                ..Default::default()
            },
            degradations,
        };
        postprocess::append_postprocess(&mut plan, viewport_size, postprocess, clear, format);
        analysis::insert_early_releases(&mut plan.passes);
        plan.compile_stats.estimated_peak_intermediate_bytes =
            analysis::estimate_plan_peak_intermediate_bytes(&plan.passes, format);
        plan.compile_stats.degradation_count = plan.degradations.len() as u64;
        plan
    }

    pub(super) fn compile_for_scene(
        encoding: &SceneEncoding,
        scale_factor: f32,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        clear: wgpu::Color,
        path_samples: u32,
        postprocess: DebugPostprocess,
        intermediate_budget_bytes: u64,
    ) -> Self {
        super::render_plan_compiler::compile_for_scene(
            encoding,
            scale_factor,
            viewport_size,
            format,
            clear,
            path_samples,
            postprocess,
            intermediate_budget_bytes,
        )
    }
}

#[cfg(test)]
mod tests;
