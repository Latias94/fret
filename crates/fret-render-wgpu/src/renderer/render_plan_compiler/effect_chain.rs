use super::context::RenderPlanCompilerCtx;
use super::draw_scope::DrawScope;
use super::effects;
use super::target_budget::intermediate_budget_breakdown_for_chain;
use crate::renderer::{BlurQualitySnapshot, EffectDegradationSnapshot, PlanTarget, ScissorRect};

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectChainBudgetStats {
    effect_chain_budget_samples: u64,
    effect_chain_effective_budget_min_bytes: u64,
    effect_chain_effective_budget_max_bytes: u64,
    effect_chain_other_live_max_bytes: u64,
    custom_effect_chain_budget_samples: u64,
    custom_effect_chain_effective_budget_min_bytes: u64,
    custom_effect_chain_effective_budget_max_bytes: u64,
    custom_effect_chain_other_live_max_bytes: u64,
    custom_effect_chain_base_required_max_bytes: u64,
    custom_effect_chain_optional_required_max_bytes: u64,
    custom_effect_chain_base_required_full_targets_max: u32,
    custom_effect_chain_optional_mask_max_bytes: u64,
    custom_effect_chain_optional_pyramid_max_bytes: u64,
}

impl Default for EffectChainBudgetStats {
    fn default() -> Self {
        Self {
            effect_chain_budget_samples: 0,
            effect_chain_effective_budget_min_bytes: u64::MAX,
            effect_chain_effective_budget_max_bytes: 0,
            effect_chain_other_live_max_bytes: 0,
            custom_effect_chain_budget_samples: 0,
            custom_effect_chain_effective_budget_min_bytes: u64::MAX,
            custom_effect_chain_effective_budget_max_bytes: 0,
            custom_effect_chain_other_live_max_bytes: 0,
            custom_effect_chain_base_required_max_bytes: 0,
            custom_effect_chain_optional_required_max_bytes: 0,
            custom_effect_chain_base_required_full_targets_max: 0,
            custom_effect_chain_optional_mask_max_bytes: 0,
            custom_effect_chain_optional_pyramid_max_bytes: 0,
        }
    }
}

impl EffectChainBudgetStats {
    pub(super) fn apply_to_plan(&self, plan: &mut super::super::RenderPlan) {
        if self.effect_chain_budget_samples > 0 {
            plan.compile_stats.effect_chain_budget_samples = self.effect_chain_budget_samples;
            plan.compile_stats.effect_chain_effective_budget_min_bytes =
                self.effect_chain_effective_budget_min_bytes;
            plan.compile_stats.effect_chain_effective_budget_max_bytes =
                self.effect_chain_effective_budget_max_bytes;
            plan.compile_stats.effect_chain_other_live_max_bytes =
                self.effect_chain_other_live_max_bytes;
        }

        if self.custom_effect_chain_budget_samples > 0 {
            plan.compile_stats.custom_effect_chain_budget_samples =
                self.custom_effect_chain_budget_samples;
            plan.compile_stats
                .custom_effect_chain_effective_budget_min_bytes =
                self.custom_effect_chain_effective_budget_min_bytes;
            plan.compile_stats
                .custom_effect_chain_effective_budget_max_bytes =
                self.custom_effect_chain_effective_budget_max_bytes;
            plan.compile_stats.custom_effect_chain_other_live_max_bytes =
                self.custom_effect_chain_other_live_max_bytes;
            plan.compile_stats
                .custom_effect_chain_base_required_max_bytes =
                self.custom_effect_chain_base_required_max_bytes;
            plan.compile_stats
                .custom_effect_chain_optional_required_max_bytes =
                self.custom_effect_chain_optional_required_max_bytes;
            plan.compile_stats
                .custom_effect_chain_base_required_full_targets_max =
                self.custom_effect_chain_base_required_full_targets_max;
            plan.compile_stats
                .custom_effect_chain_optional_mask_max_bytes =
                self.custom_effect_chain_optional_mask_max_bytes;
            plan.compile_stats
                .custom_effect_chain_optional_pyramid_max_bytes =
                self.custom_effect_chain_optional_pyramid_max_bytes;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectChainApplyCtx<'a> {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) extra_in_use_bytes: u64,
    pub(super) unavailable_mask_targets: &'a [PlanTarget],
    pub(super) reserved_targets: &'a [PlanTarget],
    pub(super) backdrop_source_group: Option<effects::BackdropSourceGroupCtx>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_chain_in_place(
    plan: &mut RenderPlanCompilerCtx,
    stats: &mut EffectChainBudgetStats,
    draw_scopes: &[DrawScope],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    args: EffectChainApplyCtx<'_>,
    effect_degradations: &mut EffectDegradationSnapshot,
    effect_blur_quality: &mut BlurQualitySnapshot,
) {
    if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
        return;
    }

    let breakdown = intermediate_budget_breakdown_for_chain(
        args.intermediate_budget_bytes,
        draw_scopes,
        srcdst,
        args.format,
        args.extra_in_use_bytes,
    );
    let effective_budget_bytes = breakdown.effective_budget_bytes;

    if !chain.is_empty() {
        stats.effect_chain_budget_samples = stats.effect_chain_budget_samples.saturating_add(1);
        stats.effect_chain_effective_budget_min_bytes = stats
            .effect_chain_effective_budget_min_bytes
            .min(effective_budget_bytes);
        stats.effect_chain_effective_budget_max_bytes = stats
            .effect_chain_effective_budget_max_bytes
            .max(effective_budget_bytes);
        stats.effect_chain_other_live_max_bytes = stats
            .effect_chain_other_live_max_bytes
            .max(breakdown.other_live_bytes);
    }

    let mut in_use_targets: Vec<PlanTarget> = Vec::new();
    for scope in draw_scopes {
        if !in_use_targets.contains(&scope.target) {
            in_use_targets.push(scope.target);
        }
    }
    for &target in args.reserved_targets {
        if !in_use_targets.contains(&target) {
            in_use_targets.push(target);
        }
    }

    let custom_evidence = effects::apply_chain_in_place(
        plan.passes_mut(),
        &in_use_targets,
        srcdst,
        mode,
        chain,
        quality,
        scissor,
        mask_uniform_index,
        args.unavailable_mask_targets,
        effect_degradations,
        effect_blur_quality,
        effects::EffectCompileCtx {
            viewport_size: args.viewport_size,
            format: args.format,
            intermediate_budget_bytes: effective_budget_bytes,
            clear: args.clear,
            scale_factor: args.scale_factor,
        },
        args.backdrop_source_group,
    );

    if let Some(evidence) = custom_evidence {
        stats.custom_effect_chain_budget_samples =
            stats.custom_effect_chain_budget_samples.saturating_add(1);
        let effective_budget_bytes = evidence.effective_budget_bytes;
        stats.custom_effect_chain_effective_budget_min_bytes = stats
            .custom_effect_chain_effective_budget_min_bytes
            .min(effective_budget_bytes);
        stats.custom_effect_chain_effective_budget_max_bytes = stats
            .custom_effect_chain_effective_budget_max_bytes
            .max(effective_budget_bytes);
        stats.custom_effect_chain_other_live_max_bytes = stats
            .custom_effect_chain_other_live_max_bytes
            .max(breakdown.other_live_bytes);
        stats.custom_effect_chain_base_required_max_bytes = stats
            .custom_effect_chain_base_required_max_bytes
            .max(evidence.base_required_bytes);
        stats.custom_effect_chain_optional_required_max_bytes = stats
            .custom_effect_chain_optional_required_max_bytes
            .max(evidence.optional_required_bytes());
        stats.custom_effect_chain_base_required_full_targets_max = stats
            .custom_effect_chain_base_required_full_targets_max
            .max(evidence.base_required_full_targets);
        stats.custom_effect_chain_optional_mask_max_bytes = stats
            .custom_effect_chain_optional_mask_max_bytes
            .max(evidence.optional_mask_bytes);
        stats.custom_effect_chain_optional_pyramid_max_bytes = stats
            .custom_effect_chain_optional_pyramid_max_bytes
            .max(evidence.optional_pyramid_bytes);
    }
}
