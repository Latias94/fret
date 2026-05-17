use super::clip_path;
use super::context::RenderPlanCompilerCtx;
use super::draw_scope::{DrawScope, take_scope_load_for_write};
use super::effect_chain::{EffectChainApplyCtx, EffectChainBudgetStats, apply_chain_in_place};
use super::effects;
use super::target_budget::can_allocate_intermediate_bytes;
use super::target_selection;
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
    pub(super) clip_path_active_mask_targets: clip_path::ActiveMaskTargets,
    pub(super) backdrop_source_group: Option<effects::BackdropSourceGroupCtx>,
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
            let had_free_scratch_target = target_selection::has_free_intermediate_target_except(
                draw_scopes,
                args.backdrop_source_group_reserved_targets,
                parent_target,
            );
            let before = plan.passes_len();
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
                    unavailable_mask_targets: args.clip_path_active_mask_targets.as_slice(),
                    reserved_targets: args.backdrop_source_group_reserved_targets,
                    backdrop_source_group: args.backdrop_source_group,
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
            let mut target_selection = target_selection::TargetSelection {
                target: None,
                had_free_target: false,
            };
            if content_size.0 != 0 && content_size.1 != 0 {
                target_selection = target_selection::choose_free_intermediate_target(
                    draw_scopes,
                    args.backdrop_source_group_reserved_targets,
                );
            }

            if target_selection.target.is_some()
                && !can_allocate_intermediate_bytes(
                    args.intermediate_budget_bytes,
                    draw_scopes,
                    estimate_texture_bytes(content_size, args.format, 1),
                    args.clip_path_mask_in_use_bytes
                        .saturating_add(args.backdrop_source_group_in_use_bytes),
                    args.format,
                )
            {
                target_selection.target = None;
            }

            let content_target = target_selection.target;
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
                    reason: if !target_selection.had_free_target {
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

        let had_free_scratch_target = target_selection::has_free_intermediate_target_except(
            draw_scopes,
            args.backdrop_source_group_reserved_targets,
            content_target,
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
