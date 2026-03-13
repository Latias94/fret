use super::super::frame_targets::FrameTargets;
use super::super::*;
use super::executor::RenderSceneExecutor;

pub(super) struct RenderSceneExecutorLifecycle<'a> {
    custom_effect_v3_pyramid: &'a mut v3_pyramid::CustomEffectV3PyramidState,
    frame_targets: &'a mut FrameTargets,
    intermediate_pool: &'a mut intermediate_pool::IntermediatePool,
}

impl<'a> RenderSceneExecutorLifecycle<'a> {
    pub(super) fn note_plan_target_written(&mut self, target: PlanTarget) {
        self.custom_effect_v3_pyramid
            .bump_plan_target_write_epoch(target);
    }

    pub(super) fn release_plan_target(&mut self, target: PlanTarget) {
        self.note_plan_target_written(target);
        self.frame_targets
            .release_target(self.intermediate_pool, target);
    }
}

pub(super) fn render_plan_pass_written_target(pass: &RenderPlanPass) -> Option<PlanTarget> {
    match pass {
        RenderPlanPass::PathClipMask(pass) => Some(pass.dst),
        RenderPlanPass::SceneDrawRange(pass) => Some(pass.target),
        RenderPlanPass::PathMsaaBatch(pass) => Some(pass.target),
        RenderPlanPass::ScaleNearest(pass) => Some(pass.dst),
        RenderPlanPass::Blur(pass) => Some(pass.dst),
        RenderPlanPass::FullscreenBlit(pass) => Some(pass.dst),
        RenderPlanPass::BackdropWarp(pass) => Some(pass.dst),
        RenderPlanPass::ColorAdjust(pass) => Some(pass.dst),
        RenderPlanPass::ColorMatrix(pass) => Some(pass.dst),
        RenderPlanPass::AlphaThreshold(pass) => Some(pass.dst),
        RenderPlanPass::Dither(pass) => Some(pass.dst),
        RenderPlanPass::Noise(pass) => Some(pass.dst),
        RenderPlanPass::DropShadow(pass) => Some(pass.dst),
        RenderPlanPass::CustomEffect(pass) => Some(pass.common.dst),
        RenderPlanPass::CustomEffectV2(pass) => Some(pass.common.dst),
        RenderPlanPass::CustomEffectV3(pass) => Some(pass.common.dst),
        RenderPlanPass::CompositePremul(pass) => Some(pass.dst),
        RenderPlanPass::ClipMask(pass) => Some(pass.dst),
        RenderPlanPass::ReleaseTarget(_) => None,
    }
}

impl<'a> RenderSceneExecutor<'a> {
    pub(super) fn lifecycle(&mut self) -> RenderSceneExecutorLifecycle<'_> {
        RenderSceneExecutorLifecycle {
            custom_effect_v3_pyramid: &mut self.renderer.custom_effect_v3_pyramid,
            frame_targets: &mut *self.frame_targets,
            intermediate_pool: &mut self.renderer.intermediate_state.pool,
        }
    }
}
