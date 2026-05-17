use super::context::RenderPlanCompilerCtx;
use super::draw_scope::{DrawScope, DrawScopeStack};
use super::target_selection;
use crate::renderer::{
    CompositePremulPass, MaskRef, PathClipMaskPass, PlanTarget, RenderPlanDegradation,
    RenderPlanDegradationKind, RenderPlanDegradationReason, RenderPlanPass, SceneEncoding,
    ScissorRect, estimate_clip_mask_bytes, estimate_texture_bytes,
};

#[derive(Clone, Copy, Debug)]
struct ClipPathScope {
    scissor: ScissorRect,
    uniform_index: u32,
    mask_draw_index: u32,
    parent_target: PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
    mask_target: Option<PlanTarget>,
    mask_size: (u32, u32),
}

pub(super) struct ClipPathPushCtx<'a> {
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) mask_draw_index: u32,
    pub(super) viewport_size: (u32, u32),
    pub(super) scissor_sized_intermediates: bool,
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) backdrop_source_group_reserved_targets: &'a [PlanTarget],
    pub(super) backdrop_source_group_in_use_bytes: u64,
}

fn active_mask_targets(scopes: &[ClipPathScope]) -> impl Iterator<Item = PlanTarget> + '_ {
    scopes.iter().filter_map(|scope| scope.mask_target)
}

#[derive(Clone, Debug)]
pub(super) struct ClipPathDispatchState {
    scopes: Vec<ClipPathScope>,
    mask_in_use_bytes: u64,
}

impl ClipPathDispatchState {
    pub(super) fn new() -> Self {
        Self {
            scopes: Vec::new(),
            mask_in_use_bytes: 0,
        }
    }

    pub(super) fn mask_in_use_bytes(&self) -> u64 {
        self.mask_in_use_bytes
    }

    pub(super) fn active_mask_targets(&self) -> ActiveMaskTargets {
        ActiveMaskTargets::from_clip_path_scopes(&self.scopes)
    }

    pub(super) fn compile_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        encoding: &SceneEncoding,
        draw_ix: usize,
        args: ClipPathPushCtx<'_>,
    ) {
        self.compile_push_inner(plan, draw_scopes, encoding, draw_ix, args);
    }

    pub(super) fn compile_pop(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
    ) {
        self.compile_pop_inner(plan, draw_scopes);
    }

    fn compile_push_inner(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        encoding: &SceneEncoding,
        draw_ix: usize,
        args: ClipPathPushCtx<'_>,
    ) {
        let parent_scope = draw_scopes.current();
        let parent_target = parent_scope.target;
        let parent_origin = parent_scope.origin;
        let parent_size = parent_scope.size;

        let mut content_selection = target_selection::TargetSelection::none();
        let mut mask_target: Option<PlanTarget> = None;
        let mut had_free_mask_target = false;

        let (content_origin, content_size) = if args.scissor_sized_intermediates {
            (
                (args.scissor.x, args.scissor.y),
                (args.scissor.w, args.scissor.h),
            )
        } else {
            ((0, 0), args.viewport_size)
        };
        let mask_size = (args.scissor.w, args.scissor.h);

        if content_size.0 != 0 && content_size.1 != 0 && mask_size.0 != 0 && mask_size.1 != 0 {
            content_selection = target_selection::choose_free_intermediate_target(
                draw_scopes,
                args.backdrop_source_group_reserved_targets,
            );

            let mask_selection = target_selection::choose_free_clip_path_mask_target(|target| {
                self.scopes
                    .iter()
                    .any(|scope| scope.mask_target == Some(target))
            });
            mask_target = mask_selection.target;
            had_free_mask_target = mask_selection.had_free_target;

            if let (Some(_content_target), Some(_mask_target)) =
                (content_selection.target, mask_target)
            {
                content_selection = target_selection::budget_filter_intermediate_target(
                    content_selection,
                    draw_scopes,
                    target_selection::IntermediateAllocationBudget {
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        required_bytes: estimate_texture_bytes(content_size, args.format, 1)
                            .saturating_add(estimate_clip_mask_bytes(mask_size)),
                        extra_in_use_bytes: self
                            .mask_in_use_bytes
                            .saturating_add(args.backdrop_source_group_in_use_bytes),
                        format: args.format,
                    },
                );
                if content_selection.target.is_none() {
                    content_selection.target = None;
                    mask_target = None;
                }
            }
        }

        let content_target = content_selection.target;
        if (content_target.is_none() || mask_target.is_none())
            && content_size.0 != 0
            && content_size.1 != 0
            && mask_size.0 != 0
            && mask_size.1 != 0
        {
            let reason = if args.intermediate_budget_bytes == 0 {
                RenderPlanDegradationReason::BudgetZero
            } else if !content_selection.had_free_target || !had_free_mask_target {
                RenderPlanDegradationReason::TargetExhausted
            } else {
                RenderPlanDegradationReason::BudgetInsufficient
            };
            plan.push_degradation(RenderPlanDegradation {
                draw_ix,
                kind: RenderPlanDegradationKind::ClipPathDisabled,
                reason,
            });
        }

        if let (Some(content_target), Some(mask_target)) = (content_target, mask_target) {
            let mask_draw = encoding.clip_path_masks[args.mask_draw_index as usize];
            debug_assert_eq!(mask_draw.scissor, args.scissor);
            debug_assert_eq!(mask_draw.uniform_index, args.uniform_index);
            plan.push_pass(RenderPlanPass::PathClipMask(PathClipMaskPass {
                dst: mask_target,
                dst_origin: (args.scissor.x, args.scissor.y),
                dst_size: mask_size,
                scissor: super::super::AbsoluteScissorRect(args.scissor),
                uniform_index: args.uniform_index,
                first_vertex: mask_draw.first_vertex,
                vertex_count: mask_draw.vertex_count,
                cache_key: mix_u64(
                    mask_draw.cache_key,
                    (u64::from(mask_size.0) << 32) | u64::from(mask_size.1),
                ),
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            }));

            draw_scopes.push(DrawScope {
                target: content_target,
                origin: content_origin,
                size: content_size,
                needs_clear: true,
                clear_color: wgpu::Color::TRANSPARENT,
            });

            self.mask_in_use_bytes = self
                .mask_in_use_bytes
                .saturating_add(estimate_clip_mask_bytes(mask_size));
        }

        self.scopes.push(ClipPathScope {
            scissor: args.scissor,
            uniform_index: args.uniform_index,
            mask_draw_index: args.mask_draw_index,
            parent_target,
            parent_origin,
            parent_size,
            content_target,
            content_origin,
            content_size,
            mask_target,
            mask_size,
        });
    }

    fn compile_pop_inner(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
    ) {
        let Some(scope) = self.scopes.pop() else {
            return;
        };

        if let (Some(content_target), Some(mask_target)) = (scope.content_target, scope.mask_target)
        {
            debug_assert_eq!(draw_scopes.current().target, content_target);

            plan.push_pass(RenderPlanPass::CompositePremul(CompositePremulPass {
                src: content_target,
                src_origin: scope.content_origin,
                dst: scope.parent_target,
                src_size: scope.content_size,
                dst_origin: scope.parent_origin,
                dst_size: scope.parent_size,
                dst_scissor: Some(super::super::AbsoluteScissorRect(scope.scissor)),
                mask_uniform_index: Some(scope.uniform_index),
                mask: Some(MaskRef {
                    target: mask_target,
                    size: scope.mask_size,
                    viewport_rect: scope.scissor,
                }),
                blend_mode: fret_core::BlendMode::Over,
                opacity: 1.0,
                load: draw_scopes.take_load_for_write(scope.parent_target),
            }));

            let _ = draw_scopes.pop();

            self.mask_in_use_bytes = self
                .mask_in_use_bytes
                .saturating_sub(estimate_clip_mask_bytes(scope.mask_size));
        } else {
            let _ = scope.mask_draw_index;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ActiveMaskTargets {
    targets: [PlanTarget; 3],
    len: usize,
}

impl ActiveMaskTargets {
    fn from_clip_path_scopes(scopes: &[ClipPathScope]) -> Self {
        let mut targets = [PlanTarget::Mask0; 3];
        let mut len = 0;
        for target in active_mask_targets(scopes) {
            debug_assert!(len < targets.len());
            if len < targets.len() {
                targets[len] = target;
                len += 1;
            }
        }
        Self { targets, len }
    }

    pub(super) fn as_slice(&self) -> &[PlanTarget] {
        &self.targets[..self.len]
    }
}

fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}
