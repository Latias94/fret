use super::context::RenderPlanCompilerCtx;
use super::draw_scope::DrawScope;
use super::effects;
use super::target_budget::{
    can_allocate_intermediate_bytes, choose_backdrop_source_group_pyramid_choice,
    estimate_in_use_intermediate_bytes,
};
use super::target_selection;
use crate::renderer::{
    BackdropSourceGroupDegradationCounters, FullscreenBlitPass, LocalScissorRect, PlanTarget,
    RenderPlanPass, ScissorRect, estimate_texture_bytes,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct BackdropSourceGroupScope {
    scissor: ScissorRect,
    pyramid_choice: Option<effects::CustomV3PyramidChoice>,
    pyramid_pad_px: u32,
    raw_target: Option<PlanTarget>,
    reserved_bytes: u64,
}

impl BackdropSourceGroupScope {
    pub(super) fn effect_ctx(&self) -> Option<effects::BackdropSourceGroupCtx> {
        self.raw_target
            .map(|raw_target| effects::BackdropSourceGroupCtx {
                raw_target,
                pyramid: self.pyramid_choice,
                scissor: self.scissor,
                pyramid_pad_px: self.pyramid_pad_px,
            })
    }
}

pub(super) struct BackdropSourceGroupDispatchState {
    scopes: Vec<BackdropSourceGroupScope>,
    reserved_targets: Vec<PlanTarget>,
    in_use_bytes: u64,
}

impl BackdropSourceGroupDispatchState {
    pub(super) fn new() -> Self {
        Self {
            scopes: Vec::new(),
            reserved_targets: Vec::new(),
            in_use_bytes: 0,
        }
    }

    pub(super) fn reserved_targets(&self) -> &[PlanTarget] {
        &self.reserved_targets
    }

    pub(super) fn in_use_bytes(&self) -> u64 {
        self.in_use_bytes
    }

    pub(super) fn effect_ctx(&self) -> Option<effects::BackdropSourceGroupCtx> {
        self.scopes.last().and_then(|scope| scope.effect_ctx())
    }

    pub(super) fn compile_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &[DrawScope],
        degradations: &mut BackdropSourceGroupDegradationCounters,
        args: BackdropSourceGroupPushCtx,
    ) {
        compile_backdrop_source_group_push(
            plan,
            draw_scopes,
            &mut self.scopes,
            &mut self.reserved_targets,
            &mut self.in_use_bytes,
            degradations,
            args,
        );
    }

    pub(super) fn compile_pop(&mut self) {
        compile_backdrop_source_group_pop(
            &mut self.scopes,
            &mut self.reserved_targets,
            &mut self.in_use_bytes,
        );
    }
}

pub(super) struct BackdropSourceGroupPushCtx {
    pub(super) scissor: ScissorRect,
    pub(super) pyramid: Option<fret_core::scene::CustomEffectPyramidRequestV1>,
    pub(super) quality: fret_core::EffectQuality,
    pub(super) scale_factor: f32,
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clip_path_mask_in_use_bytes: u64,
}

fn compile_backdrop_source_group_push(
    plan: &mut RenderPlanCompilerCtx,
    draw_scopes: &[DrawScope],
    backdrop_source_group_scopes: &mut Vec<BackdropSourceGroupScope>,
    reserved_targets: &mut Vec<PlanTarget>,
    in_use_bytes: &mut u64,
    degradations: &mut BackdropSourceGroupDegradationCounters,
    args: BackdropSourceGroupPushCtx,
) {
    degradations.requested = degradations.requested.saturating_add(1);
    if args.pyramid.is_some() {
        degradations.pyramid_requested = degradations.pyramid_requested.saturating_add(1);
    }

    let pyramid_pad_px = args
        .pyramid
        .map(|req| {
            if !req.max_radius_px.0.is_finite()
                || req.max_radius_px.0 <= 0.0
                || !args.scale_factor.is_finite()
                || args.scale_factor <= 0.0
            {
                return 0u32;
            }
            let pad = (req.max_radius_px.0 * args.scale_factor).ceil().max(0.0) as u32;
            pad.min(args.viewport_size.0.max(args.viewport_size.1))
        })
        .unwrap_or(0);

    let parent_scope = draw_scopes.last().expect("draw scope");
    let parent_target = parent_scope.target;

    let raw_selection =
        target_selection::choose_free_intermediate_target(draw_scopes, reserved_targets);
    let mut raw_target = raw_selection.target;
    let had_free_target = raw_selection.had_free_target;

    let raw_bytes = estimate_texture_bytes(args.viewport_size, args.format, 1);
    let mut reserved_bytes: u64 = 0;
    let mut pyramid_choice: Option<effects::CustomV3PyramidChoice> = None;

    let can_afford_raw = had_free_target
        && can_allocate_intermediate_bytes(
            args.intermediate_budget_bytes,
            draw_scopes,
            raw_bytes,
            args.clip_path_mask_in_use_bytes
                .saturating_add(*in_use_bytes),
            args.format,
        );
    if !can_afford_raw {
        raw_target = None;
        if args.pyramid.is_some() {
            degradations.pyramid_skipped_raw_unavailable = degradations
                .pyramid_skipped_raw_unavailable
                .saturating_add(1);
        }
        if !had_free_target {
            degradations.raw_degraded_target_exhausted =
                degradations.raw_degraded_target_exhausted.saturating_add(1);
        } else if args.intermediate_budget_bytes == 0 {
            degradations.raw_degraded_budget_zero =
                degradations.raw_degraded_budget_zero.saturating_add(1);
        } else {
            degradations.raw_degraded_budget_insufficient = degradations
                .raw_degraded_budget_insufficient
                .saturating_add(1);
        }
    }

    if let Some(raw_target) = raw_target {
        degradations.applied_raw = degradations.applied_raw.saturating_add(1);

        let snapshot_scissor = args.pyramid.map(|_| {
            let max_w = args.viewport_size.0;
            let max_h = args.viewport_size.1;
            let x0 = args.scissor.x.saturating_sub(pyramid_pad_px).min(max_w);
            let y0 = args.scissor.y.saturating_sub(pyramid_pad_px).min(max_h);
            let x1 = args
                .scissor
                .x
                .saturating_add(args.scissor.w)
                .saturating_add(pyramid_pad_px)
                .min(max_w);
            let y1 = args
                .scissor
                .y
                .saturating_add(args.scissor.h)
                .saturating_add(pyramid_pad_px)
                .min(max_h);
            if x1 <= x0 || y1 <= y0 {
                LocalScissorRect(args.scissor)
            } else {
                LocalScissorRect(ScissorRect {
                    x: x0,
                    y: y0,
                    w: x1 - x0,
                    h: y1 - y0,
                })
            }
        });

        plan.push_pass(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: parent_target,
            dst: raw_target,
            src_size: args.viewport_size,
            dst_size: args.viewport_size,
            dst_scissor: snapshot_scissor,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        }));

        reserved_bytes = reserved_bytes.saturating_add(raw_bytes);

        if let Some(req) = args.pyramid {
            let in_use_intermediate_bytes =
                estimate_in_use_intermediate_bytes(draw_scopes, args.format);

            let choice = choose_backdrop_source_group_pyramid_choice(
                req,
                args.viewport_size,
                args.format,
                args.intermediate_budget_bytes,
                in_use_intermediate_bytes,
                args.clip_path_mask_in_use_bytes,
                *in_use_bytes,
                raw_bytes,
            );

            if choice.levels >= 2 {
                degradations.pyramid_applied_levels_ge2 =
                    degradations.pyramid_applied_levels_ge2.saturating_add(1);
                reserved_bytes =
                    reserved_bytes.saturating_add(effects::estimate_custom_v3_pyramid_bytes(
                        args.viewport_size,
                        args.format,
                        choice.levels,
                    ));
            } else if let Some(reason) = choice.degraded_to_one {
                match reason {
                    effects::CustomV3PyramidDegradeReason::BudgetZero => {
                        degradations.pyramid_degraded_to_one_budget_zero = degradations
                            .pyramid_degraded_to_one_budget_zero
                            .saturating_add(1);
                    }
                    effects::CustomV3PyramidDegradeReason::BudgetInsufficient => {
                        degradations.pyramid_degraded_to_one_budget_insufficient = degradations
                            .pyramid_degraded_to_one_budget_insufficient
                            .saturating_add(1);
                    }
                }
            }
            pyramid_choice = Some(choice);
        }

        reserved_targets.push(raw_target);
        let _ = args.quality;
    }

    *in_use_bytes = in_use_bytes.saturating_add(reserved_bytes);
    backdrop_source_group_scopes.push(BackdropSourceGroupScope {
        scissor: args.scissor,
        pyramid_choice,
        pyramid_pad_px,
        raw_target,
        reserved_bytes,
    });
}

fn compile_backdrop_source_group_pop(
    backdrop_source_group_scopes: &mut Vec<BackdropSourceGroupScope>,
    reserved_targets: &mut Vec<PlanTarget>,
    in_use_bytes: &mut u64,
) {
    let Some(scope) = backdrop_source_group_scopes.pop() else {
        return;
    };

    *in_use_bytes = in_use_bytes.saturating_sub(scope.reserved_bytes);

    if let Some(raw_target) = scope.raw_target {
        let popped = reserved_targets.pop();
        debug_assert_eq!(popped, Some(raw_target));
    }

    let _ = scope.scissor;
}
