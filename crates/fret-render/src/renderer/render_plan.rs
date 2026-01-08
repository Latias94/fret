#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneSegmentId(pub(super) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanTarget {
    Output,
}

#[derive(Debug)]
pub(super) struct ScenePass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug)]
pub(super) enum RenderPlanPass {
    Scene(ScenePass),
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub(super) passes: Vec<RenderPlanPass>,
}

impl RenderPlan {
    pub(super) fn single_scene(load: wgpu::LoadOp<wgpu::Color>) -> Self {
        Self {
            passes: vec![RenderPlanPass::Scene(ScenePass {
                segment: SceneSegmentId(0),
                target: PlanTarget::Output,
                load,
            })],
        }
    }
}
