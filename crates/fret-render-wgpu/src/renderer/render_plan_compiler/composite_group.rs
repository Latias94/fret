use super::context::RenderPlanCompilerCtx;
use super::draw_scope::{DrawScope, take_scope_load_for_write};
use super::target_selection;
use crate::renderer::{
    CompositePremulPass, PlanTarget, RenderPlanDegradation, RenderPlanDegradationKind,
    RenderPlanDegradationReason, RenderPlanPass, ScissorRect, estimate_texture_bytes,
};

#[derive(Clone, Copy, Debug)]
struct CompositeGroupScope {
    mode: fret_core::BlendMode,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    uniform_index: u32,
    opacity: f32,
    parent_target: PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
}

pub(super) struct CompositeGroupDispatchState {
    scopes: Vec<CompositeGroupScope>,
}

impl CompositeGroupDispatchState {
    pub(super) fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub(super) fn compile_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut Vec<DrawScope>,
        draw_ix: usize,
        args: CompositeGroupPushCtx<'_>,
    ) {
        self.compile_push_inner(plan, draw_scopes, draw_ix, args);
    }

    pub(super) fn compile_pop(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut Vec<DrawScope>,
    ) {
        self.compile_pop_inner(plan, draw_scopes);
    }

    fn compile_push_inner(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut Vec<DrawScope>,
        draw_ix: usize,
        args: CompositeGroupPushCtx<'_>,
    ) {
        let parent_scope = draw_scopes.last().expect("draw scope");
        let parent_target = parent_scope.target;
        let parent_origin = parent_scope.origin;
        let parent_size = parent_scope.size;

        let (content_origin, content_size) = if args.scissor_sized_intermediates {
            (
                (args.scissor.x, args.scissor.y),
                (args.scissor.w, args.scissor.h),
            )
        } else {
            ((0, 0), args.viewport_size)
        };
        let mut target_selection = target_selection::TargetSelection {
            target: None,
            had_free_target: false,
        };
        if content_size.0 != 0 && content_size.1 != 0 {
            target_selection = target_selection::choose_free_intermediate_target(
                draw_scopes,
                args.backdrop_source_group_reserved_targets,
            );

            if target_selection.target.is_some()
                && !super::target_budget::can_allocate_intermediate_bytes(
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
        } else if args.mode != fret_core::BlendMode::Over
            && content_size.0 != 0
            && content_size.1 != 0
        {
            plan.push_degradation(RenderPlanDegradation {
                draw_ix,
                kind: RenderPlanDegradationKind::CompositeGroupBlendDegradedToOver,
                reason: if !target_selection.had_free_target {
                    RenderPlanDegradationReason::TargetExhausted
                } else if args.intermediate_budget_bytes == 0 {
                    RenderPlanDegradationReason::BudgetZero
                } else {
                    RenderPlanDegradationReason::BudgetInsufficient
                },
            });
        }

        self.scopes.push(CompositeGroupScope {
            mode: args.mode,
            quality: args.quality,
            scissor: args.scissor,
            uniform_index: args.uniform_index,
            opacity: args.opacity,
            parent_target,
            parent_origin,
            parent_size,
            content_target,
            content_origin,
            content_size,
        });
    }

    fn compile_pop_inner(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut Vec<DrawScope>,
    ) {
        let Some(scope) = self.scopes.pop() else {
            return;
        };

        if let Some(content_target) = scope.content_target {
            debug_assert_eq!(
                draw_scopes.last().expect("draw scope").target,
                content_target
            );

            let load = take_scope_load_for_write(draw_scopes, scope.parent_target);
            plan.push_pass(RenderPlanPass::CompositePremul(CompositePremulPass {
                src: content_target,
                src_origin: scope.content_origin,
                dst: scope.parent_target,
                src_size: scope.content_size,
                dst_origin: scope.parent_origin,
                dst_size: scope.parent_size,
                dst_scissor: Some(super::super::AbsoluteScissorRect(scope.scissor)),
                mask_uniform_index: Some(scope.uniform_index),
                mask: None,
                blend_mode: scope.mode,
                opacity: scope.opacity,
                load,
            }));

            let _ = draw_scopes.pop();
        } else if scope.mode != fret_core::BlendMode::Over {
            // Degraded: no free intermediate targets, so behave as if the group was not isolated and
            // the blend mode was `Over` (ADR 0247).
            let _ = scope.quality;
        }
    }
}

pub(super) struct CompositeGroupPushCtx<'a> {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) mode: fret_core::BlendMode,
    pub(super) quality: fret_core::EffectQuality,
    pub(super) opacity: f32,
    pub(super) viewport_size: (u32, u32),
    pub(super) scissor_sized_intermediates: bool,
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clip_path_mask_in_use_bytes: u64,
    pub(super) backdrop_source_group_reserved_targets: &'a [PlanTarget],
    pub(super) backdrop_source_group_in_use_bytes: u64,
}
