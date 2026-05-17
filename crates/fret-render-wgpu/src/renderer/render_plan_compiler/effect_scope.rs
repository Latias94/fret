use super::backdrop_source_group;
use super::clip_path;
use super::context::RenderPlanCompilerCtx;
use super::draw_scope::{DrawScope, take_scope_load_for_write};
use super::effects;
use super::target_budget::{
    can_allocate_intermediate_bytes, intermediate_budget_breakdown_for_chain,
};
use crate::renderer::{
    BlurQualitySnapshot, CompositePremulPass, EffectDegradationSnapshot, PlanTarget,
    RenderPlanDegradation, RenderPlanDegradationKind, RenderPlanDegradationReason, RenderPlanPass,
    ScissorRect, estimate_texture_bytes,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectScope {
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    uniform_index: u32,
    parent_target: PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
}

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
struct EffectChainApplyCtx<'a> {
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    clear: wgpu::Color,
    scale_factor: f32,
    intermediate_budget_bytes: u64,
    extra_in_use_bytes: u64,
    unavailable_mask_targets: &'a [PlanTarget],
    reserved_targets: &'a [PlanTarget],
    backdrop_source_group: Option<effects::BackdropSourceGroupCtx>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectScopePushCtx<'a> {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) mode: fret_core::EffectMode,
    pub(super) chain: fret_core::EffectChain,
    pub(super) quality: fret_core::EffectQuality,
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clip_path_mask_in_use_bytes: u64,
    pub(super) clip_path_scopes: &'a [clip_path::ClipPathScope],
    pub(super) backdrop_source_group_scopes:
        &'a [backdrop_source_group::BackdropSourceGroupScope],
    pub(super) backdrop_source_group_reserved_targets: &'a [PlanTarget],
    pub(super) backdrop_source_group_in_use_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectScopePopCtx<'a> {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clip_path_mask_in_use_bytes: u64,
    pub(super) backdrop_source_group_reserved_targets: &'a [PlanTarget],
    pub(super) backdrop_source_group_in_use_bytes: u64,
}

#[allow(clippy::too_many_arguments)]
fn apply_chain_in_place(
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

fn has_free_intermediate_target(
    draw_scopes: &[DrawScope],
    excluded: PlanTarget,
    reserved_targets: &[PlanTarget],
) -> bool {
    [
        PlanTarget::Intermediate0,
        PlanTarget::Intermediate1,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate3,
    ]
    .into_iter()
    .any(|target| {
        target != excluded
            && !draw_scopes.iter().any(|scope| scope.target == target)
            && !reserved_targets.contains(&target)
    })
}

fn choose_free_intermediate_target(
    draw_scopes: &[DrawScope],
    reserved_targets: &[PlanTarget],
) -> (Option<PlanTarget>, bool) {
    for target in [
        PlanTarget::Intermediate0,
        PlanTarget::Intermediate1,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate3,
    ] {
        if draw_scopes.iter().any(|scope| scope.target == target)
            || reserved_targets.contains(&target)
        {
            continue;
        }
        return (Some(target), true);
    }

    (None, false)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_effect_scope_push(
    plan: &mut RenderPlanCompilerCtx,
    draw_scopes: &mut Vec<DrawScope>,
    effect_scopes: &mut Vec<EffectScope>,
    stats: &mut EffectChainBudgetStats,
    effect_degradations: &mut EffectDegradationSnapshot,
    effect_blur_quality: &mut BlurQualitySnapshot,
    draw_ix: usize,
    args: EffectScopePushCtx<'_>,
) {
    let parent_scope = draw_scopes.last().expect("draw scope");
    let parent_target = parent_scope.target;
    let parent_origin = parent_scope.origin;
    let parent_size = parent_scope.size;

    match args.mode {
        fret_core::EffectMode::Backdrop => {
            let had_free_scratch_target = has_free_intermediate_target(
                draw_scopes,
                parent_target,
                args.backdrop_source_group_reserved_targets,
            );
            let before = plan.passes_len();
            let unavailable_mask_targets: Vec<PlanTarget> =
                clip_path::active_mask_targets(args.clip_path_scopes).collect();
            let backdrop_source_group = args
                .backdrop_source_group_scopes
                .last()
                .and_then(|scope| scope.effect_ctx());

            apply_chain_in_place(
                plan,
                stats,
                draw_scopes,
                parent_target,
                args.mode,
                args.chain,
                args.quality,
                args.scissor,
                Some(args.uniform_index),
                EffectChainApplyCtx {
                    viewport_size: parent_size,
                    format: args.format,
                    clear: args.clear,
                    scale_factor: args.scale_factor,
                    intermediate_budget_bytes: args.intermediate_budget_bytes,
                    extra_in_use_bytes: args
                        .clip_path_mask_in_use_bytes
                        .saturating_add(args.backdrop_source_group_in_use_bytes),
                    unavailable_mask_targets: &unavailable_mask_targets,
                    reserved_targets: args.backdrop_source_group_reserved_targets,
                    backdrop_source_group,
                },
                effect_degradations,
                effect_blur_quality,
            );

            if before == plan.passes_len()
                && !args.chain.is_empty()
                && parent_target != PlanTarget::Output
                && args.scissor.w != 0
                && args.scissor.h != 0
            {
                let reason = if args.intermediate_budget_bytes == 0 {
                    RenderPlanDegradationReason::BudgetZero
                } else if !had_free_scratch_target {
                    RenderPlanDegradationReason::TargetExhausted
                } else {
                    RenderPlanDegradationReason::BudgetInsufficient
                };
                plan.push_degradation(RenderPlanDegradation {
                    draw_ix,
                    kind: RenderPlanDegradationKind::BackdropEffectNoOp,
                    reason,
                });
            }

            effect_scopes.push(EffectScope {
                mode: args.mode,
                chain: args.chain,
                quality: args.quality,
                scissor: args.scissor,
                uniform_index: args.uniform_index,
                parent_target,
                parent_origin,
                parent_size,
                content_target: None,
                content_origin: (0, 0),
                content_size: (0, 0),
            });
        }
        fret_core::EffectMode::FilterContent => {
            // `bounds` are computation bounds (ADR 0117), not an implicit clip.
            // FilterContent therefore must preserve unfiltered content outside `bounds`, which
            // requires a full-viewport content target (the postprocess passes themselves remain
            // scissored to `bounds`).
            let (content_origin, content_size) = ((0, 0), args.viewport_size);
            let (mut content_target, had_free_target) =
                if content_size.0 != 0 && content_size.1 != 0 {
                    choose_free_intermediate_target(
                        draw_scopes,
                        args.backdrop_source_group_reserved_targets,
                    )
                } else {
                    (None, false)
                };

            if content_target.is_some()
                && !can_allocate_intermediate_bytes(
                    args.intermediate_budget_bytes,
                    draw_scopes,
                    estimate_texture_bytes(content_size, args.format, 1),
                    args.clip_path_mask_in_use_bytes
                        .saturating_add(args.backdrop_source_group_in_use_bytes),
                    args.format,
                )
            {
                content_target = None;
            }

            if let Some(content_target) = content_target {
                draw_scopes.push(DrawScope {
                    target: content_target,
                    origin: content_origin,
                    size: content_size,
                    needs_clear: true,
                    clear_color: wgpu::Color::TRANSPARENT,
                });
            } else if content_size.0 != 0 && content_size.1 != 0 {
                plan.push_degradation(RenderPlanDegradation {
                    draw_ix,
                    kind: RenderPlanDegradationKind::FilterContentDisabled,
                    reason: if !had_free_target {
                        RenderPlanDegradationReason::TargetExhausted
                    } else if args.intermediate_budget_bytes == 0 {
                        RenderPlanDegradationReason::BudgetZero
                    } else {
                        RenderPlanDegradationReason::BudgetInsufficient
                    },
                });
            }

            effect_scopes.push(EffectScope {
                mode: args.mode,
                chain: args.chain,
                quality: args.quality,
                scissor: args.scissor,
                uniform_index: args.uniform_index,
                parent_target,
                parent_origin,
                parent_size,
                content_target,
                content_origin,
                content_size,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_effect_scope_pop(
    plan: &mut RenderPlanCompilerCtx,
    draw_scopes: &mut Vec<DrawScope>,
    effect_scopes: &mut Vec<EffectScope>,
    stats: &mut EffectChainBudgetStats,
    effect_degradations: &mut EffectDegradationSnapshot,
    effect_blur_quality: &mut BlurQualitySnapshot,
    draw_ix: usize,
    args: EffectScopePopCtx<'_>,
) {
    let Some(scope) = effect_scopes.pop() else {
        return;
    };

    if scope.mode == fret_core::EffectMode::FilterContent
        && let Some(content_target) = scope.content_target
    {
        debug_assert_eq!(
            draw_scopes.last().expect("draw scope").target,
            content_target
        );

        let had_free_scratch_target = has_free_intermediate_target(
            draw_scopes,
            content_target,
            args.backdrop_source_group_reserved_targets,
        );
        let chain_scissor = if scope.content_size == args.viewport_size {
            scope.scissor
        } else {
            ScissorRect::full(scope.content_size.0, scope.content_size.1)
        };

        let before = plan.passes_len();
        apply_chain_in_place(
            plan,
            stats,
            draw_scopes,
            content_target,
            scope.mode,
            scope.chain,
            scope.quality,
            chain_scissor,
            None,
            EffectChainApplyCtx {
                viewport_size: scope.content_size,
                format: args.format,
                clear: args.clear,
                scale_factor: args.scale_factor,
                intermediate_budget_bytes: args.intermediate_budget_bytes,
                extra_in_use_bytes: args
                    .clip_path_mask_in_use_bytes
                    .saturating_add(args.backdrop_source_group_in_use_bytes),
                unavailable_mask_targets: &[],
                reserved_targets: args.backdrop_source_group_reserved_targets,
                backdrop_source_group: None,
            },
            effect_degradations,
            effect_blur_quality,
        );

        if before == plan.passes_len()
            && !scope.chain.is_empty()
            && chain_scissor.w != 0
            && chain_scissor.h != 0
        {
            plan.push_degradation(RenderPlanDegradation {
                draw_ix,
                kind: RenderPlanDegradationKind::FilterContentDisabled,
                reason: if args.intermediate_budget_bytes == 0 {
                    RenderPlanDegradationReason::BudgetZero
                } else if !had_free_scratch_target {
                    RenderPlanDegradationReason::TargetExhausted
                } else {
                    RenderPlanDegradationReason::BudgetInsufficient
                },
            });
        }

        let cropped = scope.content_origin != (0, 0) || scope.content_size != args.viewport_size;
        let load = take_scope_load_for_write(draw_scopes, scope.parent_target);
        plan.push_pass(RenderPlanPass::CompositePremul(CompositePremulPass {
            src: content_target,
            src_origin: scope.content_origin,
            dst: scope.parent_target,
            src_size: scope.content_size,
            dst_origin: scope.parent_origin,
            dst_size: scope.parent_size,
            dst_scissor: cropped.then_some(super::super::AbsoluteScissorRect(scope.scissor)),
            mask_uniform_index: Some(scope.uniform_index),
            mask: None,
            blend_mode: fret_core::BlendMode::Over,
            opacity: 1.0,
            load,
        }));

        let _ = draw_scopes.pop();
    }
}
