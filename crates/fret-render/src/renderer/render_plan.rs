use super::frame_targets::downsampled_size;
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
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
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
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
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
        viewport_size: (u32, u32),
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
            append_postprocess(&mut plan, viewport_size, postprocess, clear);
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
        append_postprocess(&mut plan, viewport_size, postprocess, clear);
        plan
    }
}

fn decompose_pixelate_scale(scale: u32) -> Vec<u32> {
    let mut scale = scale.max(1);
    let mut steps = Vec::new();
    while scale >= 4 && scale % 2 == 0 {
        steps.push(2);
        scale /= 2;
    }
    steps.push(scale.max(1));
    steps
}

fn push_scale_nearest(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    mode: ScaleMode,
    scale: u32,
    clear: wgpu::Color,
) {
    plan.passes
        .push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
            src,
            dst,
            src_size,
            dst_size,
            mode,
            scale,
            load: wgpu::LoadOp::Clear(clear),
        }));
}

fn push_fullscreen_blit(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    clear: wgpu::Color,
) {
    plan.passes
        .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src,
            dst,
            src_size,
            dst_size,
            load: wgpu::LoadOp::Clear(clear),
        }));
}

fn append_downsample_chain(
    plan: &mut RenderPlan,
    mut current_target: PlanTarget,
    mut current_size: (u32, u32),
    steps: &[u32],
    mut dst_a: PlanTarget,
    mut dst_b: PlanTarget,
    clear: wgpu::Color,
) -> (PlanTarget, (u32, u32), Vec<((u32, u32), u32)>) {
    let mut stack: Vec<((u32, u32), u32)> = Vec::with_capacity(steps.len());
    for step in steps.iter().copied() {
        let dst_size = downsampled_size(current_size, step);
        push_scale_nearest(
            plan,
            current_target,
            dst_a,
            current_size,
            dst_size,
            ScaleMode::Downsample,
            step,
            clear,
        );
        stack.push((current_size, step));
        current_target = dst_a;
        current_size = dst_size;
        std::mem::swap(&mut dst_a, &mut dst_b);
    }
    (current_target, current_size, stack)
}

fn append_upsample_chain(
    plan: &mut RenderPlan,
    mut current_target: PlanTarget,
    mut current_size: (u32, u32),
    mut stack: Vec<((u32, u32), u32)>,
    clear: wgpu::Color,
) -> (PlanTarget, (u32, u32)) {
    while let Some((dst_size, step)) = stack.pop() {
        let dst_target = match current_target {
            PlanTarget::Intermediate1 => PlanTarget::Intermediate2,
            PlanTarget::Intermediate2 => PlanTarget::Intermediate1,
            PlanTarget::Intermediate0 | PlanTarget::Output => {
                unreachable!("upsample chain must read from Intermediate1/2")
            }
        };
        push_scale_nearest(
            plan,
            current_target,
            dst_target,
            current_size,
            dst_size,
            ScaleMode::Upscale,
            step,
            clear,
        );
        current_target = dst_target;
        current_size = dst_size;
    }
    (current_target, current_size)
}

fn append_postprocess(
    plan: &mut RenderPlan,
    viewport_size: (u32, u32),
    postprocess: DebugPostprocess,
    clear: wgpu::Color,
) {
    match postprocess {
        DebugPostprocess::None => {}
        DebugPostprocess::OffscreenBlit => {
            push_fullscreen_blit(
                plan,
                PlanTarget::Intermediate0,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                clear,
            );
        }
        DebugPostprocess::Pixelate { scale } => {
            let steps = decompose_pixelate_scale(scale);
            let (current_target, current_size, stack) = append_downsample_chain(
                plan,
                PlanTarget::Intermediate0,
                viewport_size,
                &steps,
                PlanTarget::Intermediate2,
                PlanTarget::Intermediate1,
                clear,
            );
            let (current_target, _current_size) =
                append_upsample_chain(plan, current_target, current_size, stack, clear);
            push_fullscreen_blit(
                plan,
                current_target,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                clear,
            );
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
            (100, 100),
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
            (100, 100),
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
        assert_eq!(blit.src_size, (100, 100));
        assert_eq!(blit.dst_size, (100, 100));
    }

    #[test]
    fn compile_for_scene_pixelate_adds_scale_chain_then_blit() {
        let encoding = SceneEncoding::default();
        let viewport_size = (128, 64);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Pixelate { scale: 4 },
        );

        assert_eq!(plan.passes.len(), 6);
        let RenderPlanPass::SceneDrawRange(scene) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(down0) = &plan.passes[1] else {
            panic!("expected ScaleNearest downsample pass 0");
        };
        assert_eq!(down0.src, PlanTarget::Intermediate0);
        assert_eq!(down0.dst, PlanTarget::Intermediate2);
        assert_eq!(down0.mode, ScaleMode::Downsample);
        assert_eq!(down0.scale, 2);
        assert_eq!(down0.src_size, viewport_size);
        assert_eq!(down0.dst_size, downsampled_size(viewport_size, 2));

        let RenderPlanPass::ScaleNearest(down1) = &plan.passes[2] else {
            panic!("expected ScaleNearest downsample pass 1");
        };
        assert_eq!(down1.src, PlanTarget::Intermediate2);
        assert_eq!(down1.dst, PlanTarget::Intermediate1);
        assert_eq!(down1.mode, ScaleMode::Downsample);
        assert_eq!(down1.scale, 2);
        assert_eq!(down1.src_size, down0.dst_size);
        assert_eq!(down1.dst_size, downsampled_size(down0.dst_size, 2));

        let RenderPlanPass::ScaleNearest(up0) = &plan.passes[3] else {
            panic!("expected ScaleNearest upscale pass 0");
        };
        assert_eq!(up0.src, PlanTarget::Intermediate1);
        assert_eq!(up0.dst, PlanTarget::Intermediate2);
        assert_eq!(up0.mode, ScaleMode::Upscale);
        assert_eq!(up0.scale, 2);
        assert_eq!(up0.src_size, down1.dst_size);
        assert_eq!(up0.dst_size, down1.src_size);

        let RenderPlanPass::ScaleNearest(up1) = &plan.passes[4] else {
            panic!("expected ScaleNearest upscale pass 1");
        };
        assert_eq!(up1.src, PlanTarget::Intermediate2);
        assert_eq!(up1.dst, PlanTarget::Intermediate1);
        assert_eq!(up1.mode, ScaleMode::Upscale);
        assert_eq!(up1.scale, 2);
        assert_eq!(up1.src_size, up0.dst_size);
        assert_eq!(up1.dst_size, viewport_size);

        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[5] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate1);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, viewport_size);
        assert_eq!(blit.dst_size, viewport_size);
    }
}
