use super::util::union_scissor;
use super::{OrderedDraw, SceneEncoding, ScissorRect};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneSegmentId(pub(super) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanTarget {
    Output,
    Intermediate0,
    Intermediate1,
    Intermediate2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DebugPostprocess {
    None,
    OffscreenBlit,
    Pixelate { scale: u32 },
}

#[derive(Debug)]
pub(super) struct SceneDrawRangePass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
    pub(super) draw_range: Range<usize>,
}

#[derive(Debug)]
pub(super) enum RenderPlanPass {
    SceneDrawRange(SceneDrawRangePass),
    PathMsaaBatch(PathMsaaBatchPass),
    FullscreenBlit(FullscreenBlitPass),
    ScaleNearest(ScaleNearestPass),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FullscreenBlitPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
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
    pub(super) mode: ScaleMode,
    pub(super) scale: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone)]
pub(super) struct PathMsaaBatchPass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) draw_range: Range<usize>,
    pub(super) union_scissor: ScissorRect,
    pub(super) batch_uniform_index: u32,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub(super) passes: Vec<RenderPlanPass>,
}

impl RenderPlan {
    pub(super) fn compile_for_scene(
        encoding: &SceneEncoding,
        clear: wgpu::Color,
        path_samples: u32,
        postprocess: DebugPostprocess,
    ) -> Self {
        let scene_target = match postprocess {
            DebugPostprocess::None => PlanTarget::Output,
            DebugPostprocess::OffscreenBlit | DebugPostprocess::Pixelate { .. } => {
                PlanTarget::Intermediate0
            }
        };
        let draws = &encoding.ordered_draws;

        if path_samples <= 1 {
            let mut plan = Self {
                passes: vec![RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: scene_target,
                    load: wgpu::LoadOp::Clear(clear),
                    draw_range: 0..draws.len(),
                })],
            };
            append_postprocess(&mut plan, postprocess, clear);
            return plan;
        }

        let mut passes: Vec<RenderPlanPass> = Vec::new();
        let mut is_first_pass = true;
        let mut scene_range_start: usize = 0;
        let mut cursor: usize = 0;

        while cursor < draws.len() {
            if let OrderedDraw::Path(first) = &draws[cursor] {
                if is_first_pass {
                    passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                        segment: SceneSegmentId(0),
                        target: scene_target,
                        load: wgpu::LoadOp::Clear(clear),
                        draw_range: scene_range_start..cursor,
                    }));
                    is_first_pass = false;
                } else if scene_range_start < cursor {
                    passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                        segment: SceneSegmentId(0),
                        target: scene_target,
                        load: wgpu::LoadOp::Load,
                        draw_range: scene_range_start..cursor,
                    }));
                }

                let batch_uniform_index = first.uniform_index;
                let mut union = first.scissor;
                let mut end = cursor + 1;
                while end < draws.len() {
                    match &draws[end] {
                        OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                            union = union_scissor(union, d.scissor);
                            end += 1;
                        }
                        _ => break,
                    }
                }

                passes.push(RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                    segment: SceneSegmentId(0),
                    target: scene_target,
                    draw_range: cursor..end,
                    union_scissor: union,
                    batch_uniform_index,
                }));

                cursor = end;
                scene_range_start = cursor;
                continue;
            }

            cursor += 1;
        }

        if is_first_pass {
            passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment: SceneSegmentId(0),
                target: scene_target,
                load: wgpu::LoadOp::Clear(clear),
                draw_range: 0..draws.len(),
            }));
        } else if scene_range_start < draws.len() {
            passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment: SceneSegmentId(0),
                target: scene_target,
                load: wgpu::LoadOp::Load,
                draw_range: scene_range_start..draws.len(),
            }));
        }

        let mut plan = Self { passes };
        append_postprocess(&mut plan, postprocess, clear);
        plan
    }
}

fn append_postprocess(plan: &mut RenderPlan, postprocess: DebugPostprocess, clear: wgpu::Color) {
    match postprocess {
        DebugPostprocess::None => {}
        DebugPostprocess::OffscreenBlit => {
            plan.passes
                .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Output,
                    load: wgpu::LoadOp::Clear(clear),
                }));
        }
        DebugPostprocess::Pixelate { scale } => {
            #[derive(Clone, Copy)]
            struct PingPong {
                read: PlanTarget,
                write: PlanTarget,
            }

            impl PingPong {
                fn new() -> Self {
                    Self {
                        read: PlanTarget::Intermediate0,
                        write: PlanTarget::Intermediate1,
                    }
                }

                fn swap(&mut self) {
                    std::mem::swap(&mut self.read, &mut self.write);
                }
            }

            let scale = scale.max(1);
            let mut ping_pong = PingPong::new();
            plan.passes
                .push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
                    src: ping_pong.read,
                    dst: PlanTarget::Intermediate2,
                    mode: ScaleMode::Downsample,
                    scale,
                    load: wgpu::LoadOp::Clear(clear),
                }));
            ping_pong.swap();
            plan.passes
                .push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
                    src: PlanTarget::Intermediate2,
                    dst: ping_pong.read,
                    mode: ScaleMode::Upscale,
                    scale,
                    load: wgpu::LoadOp::Clear(clear),
                }));
            plan.passes
                .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: ping_pong.read,
                    dst: PlanTarget::Output,
                    load: wgpu::LoadOp::Clear(clear),
                }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_for_scene_none_targets_output() {
        let encoding = SceneEncoding::default();
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::None,
        );

        assert_eq!(plan.passes.len(), 1);
        let RenderPlanPass::SceneDrawRange(pass) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(pass.target, PlanTarget::Output);
    }

    #[test]
    fn compile_for_scene_offscreen_blit_adds_fullscreen_blit() {
        let encoding = SceneEncoding::default();
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::OffscreenBlit,
        );

        assert_eq!(plan.passes.len(), 2);
        let RenderPlanPass::SceneDrawRange(scene) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);
        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[1] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate0);
        assert_eq!(blit.dst, PlanTarget::Output);
    }

    #[test]
    fn compile_for_scene_pixelate_adds_scale_chain_then_blit() {
        let encoding = SceneEncoding::default();
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Pixelate { scale: 4 },
        );

        assert_eq!(plan.passes.len(), 4);
        let RenderPlanPass::SceneDrawRange(scene) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(down) = &plan.passes[1] else {
            panic!("expected ScaleNearest downsample pass");
        };
        assert_eq!(down.src, PlanTarget::Intermediate0);
        assert_eq!(down.dst, PlanTarget::Intermediate2);
        assert_eq!(down.mode, ScaleMode::Downsample);
        assert_eq!(down.scale, 4);

        let RenderPlanPass::ScaleNearest(up) = &plan.passes[2] else {
            panic!("expected ScaleNearest upscale pass");
        };
        assert_eq!(up.src, PlanTarget::Intermediate2);
        assert_eq!(up.dst, PlanTarget::Intermediate1);
        assert_eq!(up.mode, ScaleMode::Upscale);
        assert_eq!(up.scale, 4);

        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[3] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate1);
        assert_eq!(blit.dst, PlanTarget::Output);
    }
}
