use super::blur_primitive;
use super::frame_targets::downsampled_size;
use super::intermediate_pool::{
    estimate_clip_mask_bytes, estimate_mipped_texture_bytes, estimate_texture_bytes,
};
use super::{
    AlphaThresholdPass, BackdropWarpPass, BlurAxis, BlurPass, ClipMaskPass, ColorAdjustPass,
    ColorMatrixPass, CustomEffectPass, CustomEffectV2Pass, CustomEffectV3Pass, DitherPass,
    DropShadowPass, FullscreenBlitPass, LocalScissorRect, MaskRef, NoisePass, PlanTarget,
    RenderPlanPass, ScaleMode, ScaleNearestPass, ScissorRect,
};

fn required_bytes_for_full_size_targets(full_target_bytes: u64, target_count: u64) -> u64 {
    // Budget model convention: intermediate budgets are reasoned about as the sum of concurrent
    // full-viewport intermediate targets (plus optional mips / downsampled targets).
    full_target_bytes.saturating_mul(target_count)
}

fn base_required_bytes_for_srcdst_and_single_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 2)
}

fn base_required_bytes_for_srcdst_and_two_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 3)
}

fn base_required_bytes_for_srcdst_and_three_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 4)
}

fn budget_excluding_full_size_targets(
    budget_bytes: u64,
    full_target_bytes: u64,
    excluded_targets: u64,
) -> u64 {
    budget_bytes.saturating_sub(required_bytes_for_full_size_targets(
        full_target_bytes,
        excluded_targets,
    ))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectCompileCtx {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CustomEffectChainBudgetEvidence {
    pub(super) effective_budget_bytes: u64,
    pub(super) base_required_bytes: u64,
    pub(super) base_required_full_targets: u32,
    pub(super) optional_mask_bytes: u64,
    pub(super) optional_pyramid_bytes: u64,
}

impl CustomEffectChainBudgetEvidence {
    pub(super) fn optional_required_bytes(&self) -> u64 {
        self.optional_mask_bytes
            .saturating_add(self.optional_pyramid_bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CustomV3PyramidDegradeReason {
    BudgetZero,
    BudgetInsufficient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CustomV3PyramidChoice {
    pub(super) levels: u32,
    pub(super) degraded_to_one: Option<CustomV3PyramidDegradeReason>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BackdropSourceGroupCtx {
    pub(super) raw_target: PlanTarget,
    pub(super) pyramid: Option<CustomV3PyramidChoice>,
    pub(super) scissor: ScissorRect,
    pub(super) pyramid_pad_px: u32,
}

mod blur;
mod builtin;
mod chain;
mod custom;
mod scissor;

use self::chain::{
    apply_unpadded_chain_in_place, backdrop_source_group_parts, prepare_chain_start,
    try_apply_padded_chain_in_place,
};

#[cfg(test)]
use self::blur::inflate_scissor_to_viewport;
#[cfg(test)]
use self::custom::plan_custom_v3_sources_and_charge_budget;

pub(super) fn choose_effect_blur_downsample_scale(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> Option<u32> {
    builtin::choose_effect_blur_downsample_scale(
        viewport_size,
        format,
        budget_bytes,
        requested_downsample,
        quality,
    )
}

pub(super) fn color_adjust_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::color_adjust_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn dither_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::dither_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn noise_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::noise_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn backdrop_warp_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::backdrop_warp_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn pixelate_enabled(
    viewport_size: (u32, u32),
    scissor: Option<ScissorRect>,
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    scale: u32,
) -> bool {
    builtin::pixelate_enabled(viewport_size, scissor, format, budget_bytes, scale)
}

pub(super) fn choose_custom_v3_pyramid_choice_for_request(
    req: fret_core::scene::CustomEffectPyramidRequestV1,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    base_required_bytes: u64,
) -> CustomV3PyramidChoice {
    custom::choose_custom_v3_pyramid_choice_for_request(
        req,
        size,
        format,
        budget_bytes,
        base_required_bytes,
    )
}

pub(super) fn estimate_custom_v3_pyramid_bytes(
    size: (u32, u32),
    format: wgpu::TextureFormat,
    levels: u32,
) -> u64 {
    custom::estimate_custom_v3_pyramid_bytes(size, format, levels)
}

pub(super) fn map_scissor_to_size(
    scissor: Option<ScissorRect>,
    src_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    scissor::map_scissor_to_size(scissor, src_size, dst_size)
}

pub(super) fn map_scissor_downsample_nearest(
    scissor: Option<ScissorRect>,
    scale: u32,
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    scissor::map_scissor_downsample_nearest(scissor, scale, dst_size)
}

pub(super) fn apply_chain_in_place(
    passes: &mut Vec<RenderPlanPass>,
    in_use_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    unavailable_mask_targets: &[PlanTarget],
    effect_degradations: &mut super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
) -> Option<CustomEffectChainBudgetEvidence> {
    if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let steps: Vec<fret_core::EffectStep> = chain.iter().collect();
    if steps.is_empty() {
        return None;
    }

    let (group_raw, _, _) = backdrop_source_group_parts(backdrop_source_group);

    let mut chain_start = prepare_chain_start(
        passes,
        in_use_targets,
        srcdst,
        &steps,
        scissor,
        mask_uniform_index,
        unavailable_mask_targets,
        quality,
        ctx,
    );

    // Padded effect chains:
    //
    // Some effects sample their input outside the destination pixel (blur radius, refraction
    // displacement, chromatic offsets). When such an effect appears later in a chain, earlier
    // steps must produce output for an expanded region, otherwise the chain will see edge artifacts
    // (e.g. "dark corners" in blur -> refraction lenses).
    //
    // If the chain declares any non-zero sampling padding and we have enough scratch targets +
    // budget, we evaluate the chain into a work target using a per-step expanded scissor plan,
    // then composite the final result back into `srcdst` while applying clip/mask coverage exactly
    // once.
    if try_apply_padded_chain_in_place(
        passes,
        &steps,
        chain_start.scratch_targets.as_slice(),
        srcdst,
        mode,
        quality,
        scissor,
        chain_start.budget_bytes,
        chain_start.coverage,
        effect_degradations,
        effect_blur_quality,
        ctx,
        backdrop_source_group,
        &mut chain_start.custom_chain_budget,
    ) {
        return chain_start.custom_chain_budget;
    }

    // Clip/shape masks are coverage (alpha) multipliers. If we apply them at every effect step in a
    // chain, coverage compounds (e.g. clip^2) and produces visible edge darkening (especially
    // around rounded corners) for common chains like blur -> custom refraction.
    //
    // To avoid this, we apply clip/mask only on the final step of the chain and keep intermediate
    // steps unmasked.
    apply_unpadded_chain_in_place(
        passes,
        &steps,
        srcdst,
        mode,
        quality,
        scissor,
        group_raw,
        effect_degradations,
        effect_blur_quality,
        ctx,
        backdrop_source_group,
        &mut chain_start,
    )
}

#[cfg(test)]
mod tests;
