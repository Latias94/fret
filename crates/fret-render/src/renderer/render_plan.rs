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
    Pixelate {
        scale: u32,
    },
    Blur {
        radius: u32,
        downsample_scale: u32,
        scissor: Option<ScissorRect>,
    },
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
    Blur(BlurPass),
    ReleaseTarget(PlanTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlurAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BlurPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) axis: BlurAxis,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FullscreenBlitPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
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
    pub(super) dst_scissor: Option<ScissorRect>,
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
            DebugPostprocess::OffscreenBlit
            | DebugPostprocess::Pixelate { .. }
            | DebugPostprocess::Blur { .. } => PlanTarget::Intermediate0,
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

fn map_scissor_to_size(
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor_in_full?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let full_w = full_size.0.max(1) as u64;
    let full_h = full_size.1.max(1) as u64;
    let dst_w = dst_size.0.max(1) as u64;
    let dst_h = dst_size.1.max(1) as u64;

    let x0 = scissor.x as u64;
    let y0 = scissor.y as u64;
    let x1 = x0.saturating_add(scissor.w as u64);
    let y1 = y0.saturating_add(scissor.h as u64);

    let sx0 = (x0 * dst_w) / full_w;
    let sy0 = (y0 * dst_h) / full_h;
    let sx1 = (x1 * dst_w + full_w - 1) / full_w;
    let sy1 = (y1 * dst_h + full_h - 1) / full_h;

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0 as u32,
        y: sy0 as u32,
        w: (sx1 - sx0) as u32,
        h: (sy1 - sy0) as u32,
    })
}

fn push_scale_nearest(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
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
            dst_scissor,
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
    dst_scissor: Option<ScissorRect>,
    clear: wgpu::Color,
) {
    plan.passes
        .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src,
            dst,
            src_size,
            dst_size,
            dst_scissor,
            load: wgpu::LoadOp::Clear(clear),
        }));
}

fn push_blur(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    axis: BlurAxis,
    clear: wgpu::Color,
) {
    plan.passes.push(RenderPlanPass::Blur(BlurPass {
        src,
        dst,
        src_size,
        dst_size,
        dst_scissor,
        axis,
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
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
) -> (PlanTarget, (u32, u32), Vec<((u32, u32), u32)>) {
    let mut stack: Vec<((u32, u32), u32)> = Vec::with_capacity(steps.len());
    for step in steps.iter().copied() {
        let dst_size = downsampled_size(current_size, step);
        let dst_scissor = map_scissor_to_size(scissor_in_full, full_size, dst_size);
        push_scale_nearest(
            plan,
            current_target,
            dst_a,
            current_size,
            dst_size,
            dst_scissor,
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

#[derive(Debug, Clone)]
struct DownsampleHalfQuarter {
    half_target: PlanTarget,
    #[allow(dead_code)]
    half_size: (u32, u32),
    quarter_target: PlanTarget,
    quarter_size: (u32, u32),
    stack: Vec<((u32, u32), u32)>,
}

fn append_downsample_half_quarter(
    plan: &mut RenderPlan,
    src_target: PlanTarget,
    src_size: (u32, u32),
    half_target: PlanTarget,
    quarter_target: PlanTarget,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
    release_src_after_first: bool,
) -> DownsampleHalfQuarter {
    debug_assert_ne!(src_target, PlanTarget::Output);
    debug_assert_ne!(half_target, PlanTarget::Output);
    debug_assert_ne!(quarter_target, PlanTarget::Output);
    debug_assert_ne!(half_target, quarter_target);

    let half_size = downsampled_size(src_size, 2);
    let half_scissor = map_scissor_to_size(scissor_in_full, full_size, half_size);
    push_scale_nearest(
        plan,
        src_target,
        half_target,
        src_size,
        half_size,
        half_scissor,
        ScaleMode::Downsample,
        2,
        clear,
    );
    if release_src_after_first {
        plan.passes.push(RenderPlanPass::ReleaseTarget(src_target));
    }

    let quarter_size = downsampled_size(half_size, 2);
    let quarter_scissor = map_scissor_to_size(scissor_in_full, full_size, quarter_size);
    push_scale_nearest(
        plan,
        half_target,
        quarter_target,
        half_size,
        quarter_size,
        quarter_scissor,
        ScaleMode::Downsample,
        2,
        clear,
    );

    DownsampleHalfQuarter {
        half_target,
        half_size,
        quarter_target,
        quarter_size,
        stack: vec![(src_size, 2), (half_size, 2)],
    }
}

fn append_upsample_chain(
    plan: &mut RenderPlan,
    mut current_target: PlanTarget,
    mut current_size: (u32, u32),
    mut stack: Vec<((u32, u32), u32)>,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
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
        let dst_scissor = map_scissor_to_size(scissor_in_full, full_size, dst_size);
        push_scale_nearest(
            plan,
            current_target,
            dst_target,
            current_size,
            dst_size,
            dst_scissor,
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
                None,
                clear,
            );
        }
        DebugPostprocess::Pixelate { scale } => {
            let steps = decompose_pixelate_scale(scale);
            let (current_target, current_size, stack) =
                if steps.len() >= 2 && steps[0] == 2 && steps[1] == 2 {
                    let half_quarter = append_downsample_half_quarter(
                        plan,
                        PlanTarget::Intermediate0,
                        viewport_size,
                        PlanTarget::Intermediate2,
                        PlanTarget::Intermediate1,
                        None,
                        viewport_size,
                        clear,
                        true,
                    );

                    let mut stack = half_quarter.stack;
                    let (current_target, current_size, rest_stack) = append_downsample_chain(
                        plan,
                        half_quarter.quarter_target,
                        half_quarter.quarter_size,
                        &steps[2..],
                        half_quarter.half_target,
                        half_quarter.quarter_target,
                        None,
                        viewport_size,
                        clear,
                    );
                    stack.extend(rest_stack);
                    (current_target, current_size, stack)
                } else {
                    let first_step = steps[0];
                    let dst_size = downsampled_size(viewport_size, first_step);
                    push_scale_nearest(
                        plan,
                        PlanTarget::Intermediate0,
                        PlanTarget::Intermediate2,
                        viewport_size,
                        dst_size,
                        None,
                        ScaleMode::Downsample,
                        first_step,
                        clear,
                    );
                    plan.passes
                        .push(RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0));
                    let mut stack = vec![(viewport_size, first_step)];

                    let (current_target, current_size, rest_stack) = append_downsample_chain(
                        plan,
                        PlanTarget::Intermediate2,
                        dst_size,
                        &steps[1..],
                        PlanTarget::Intermediate1,
                        PlanTarget::Intermediate2,
                        None,
                        viewport_size,
                        clear,
                    );
                    stack.extend(rest_stack);
                    (current_target, current_size, stack)
                };
            let (current_target, _current_size) = append_upsample_chain(
                plan,
                current_target,
                current_size,
                stack,
                None,
                viewport_size,
                clear,
            );
            push_fullscreen_blit(
                plan,
                current_target,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                None,
                clear,
            );
        }
        DebugPostprocess::Blur {
            radius,
            downsample_scale,
            scissor,
        } => {
            let _radius = radius.max(1);
            let downsample_scale = if downsample_scale >= 4 { 4 } else { 2 };
            let use_quarter = downsample_scale == 4;

            let (blur_src, blur_size, scratch) = if use_quarter {
                (
                    PlanTarget::Intermediate1,
                    downsampled_size(viewport_size, 4),
                    PlanTarget::Intermediate2,
                )
            } else {
                (
                    PlanTarget::Intermediate2,
                    downsampled_size(viewport_size, 2),
                    PlanTarget::Intermediate1,
                )
            };

            let down_scissor = map_scissor_to_size(scissor, viewport_size, blur_size);
            push_scale_nearest(
                plan,
                PlanTarget::Intermediate0,
                blur_src,
                viewport_size,
                blur_size,
                down_scissor,
                ScaleMode::Downsample,
                downsample_scale,
                clear,
            );
            plan.passes
                .push(RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0));

            let blur_scissor = map_scissor_to_size(scissor, viewport_size, blur_size);
            push_blur(
                plan,
                blur_src,
                scratch,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Horizontal,
                clear,
            );
            push_blur(
                plan,
                scratch,
                blur_src,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Vertical,
                clear,
            );

            let final_scissor = map_scissor_to_size(scissor, viewport_size, viewport_size);
            push_scale_nearest(
                plan,
                blur_src,
                PlanTarget::Intermediate0,
                blur_size,
                viewport_size,
                final_scissor,
                ScaleMode::Upscale,
                downsample_scale,
                clear,
            );
            push_fullscreen_blit(
                plan,
                PlanTarget::Intermediate0,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                final_scissor,
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
        assert_eq!(blit.dst_scissor, None);
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

        assert_eq!(plan.passes.len(), 7);
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
        assert_eq!(down0.dst_scissor, None);

        let RenderPlanPass::ReleaseTarget(release) = &plan.passes[2] else {
            panic!("expected ReleaseTarget pass");
        };
        assert_eq!(*release, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(down1) = &plan.passes[3] else {
            panic!("expected ScaleNearest downsample pass 1");
        };
        assert_eq!(down1.src, PlanTarget::Intermediate2);
        assert_eq!(down1.dst, PlanTarget::Intermediate1);
        assert_eq!(down1.mode, ScaleMode::Downsample);
        assert_eq!(down1.scale, 2);
        assert_eq!(down1.src_size, down0.dst_size);
        assert_eq!(down1.dst_size, downsampled_size(down0.dst_size, 2));
        assert_eq!(down1.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(up0) = &plan.passes[4] else {
            panic!("expected ScaleNearest upscale pass 0");
        };
        assert_eq!(up0.src, PlanTarget::Intermediate1);
        assert_eq!(up0.dst, PlanTarget::Intermediate2);
        assert_eq!(up0.mode, ScaleMode::Upscale);
        assert_eq!(up0.scale, 2);
        assert_eq!(up0.src_size, down1.dst_size);
        assert_eq!(up0.dst_size, down1.src_size);
        assert_eq!(up0.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(up1) = &plan.passes[5] else {
            panic!("expected ScaleNearest upscale pass 1");
        };
        assert_eq!(up1.src, PlanTarget::Intermediate2);
        assert_eq!(up1.dst, PlanTarget::Intermediate1);
        assert_eq!(up1.mode, ScaleMode::Upscale);
        assert_eq!(up1.scale, 2);
        assert_eq!(up1.src_size, up0.dst_size);
        assert_eq!(up1.dst_size, viewport_size);
        assert_eq!(up1.dst_scissor, None);

        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[6] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate1);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, viewport_size);
        assert_eq!(blit.dst_size, viewport_size);
        assert_eq!(blit.dst_scissor, None);
    }

    #[test]
    fn downsample_half_quarter_helper_emits_two_passes() {
        let mut plan = RenderPlan { passes: Vec::new() };
        let info = append_downsample_half_quarter(
            &mut plan,
            PlanTarget::Intermediate0,
            (128, 64),
            PlanTarget::Intermediate2,
            PlanTarget::Intermediate1,
            None,
            (128, 64),
            wgpu::Color::TRANSPARENT,
            false,
        );

        assert_eq!(info.half_size, (64, 32));
        assert_eq!(info.quarter_size, (32, 16));
        assert_eq!(info.stack, vec![((128, 64), 2), ((64, 32), 2)]);

        assert_eq!(plan.passes.len(), 2);
        let RenderPlanPass::ScaleNearest(pass0) = &plan.passes[0] else {
            panic!("expected ScaleNearest pass 0");
        };
        assert_eq!(pass0.src, PlanTarget::Intermediate0);
        assert_eq!(pass0.dst, PlanTarget::Intermediate2);
        assert_eq!(pass0.src_size, (128, 64));
        assert_eq!(pass0.dst_size, (64, 32));
        assert_eq!(pass0.mode, ScaleMode::Downsample);
        assert_eq!(pass0.scale, 2);
        assert_eq!(pass0.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(pass1) = &plan.passes[1] else {
            panic!("expected ScaleNearest pass 1");
        };
        assert_eq!(pass1.src, PlanTarget::Intermediate2);
        assert_eq!(pass1.dst, PlanTarget::Intermediate1);
        assert_eq!(pass1.src_size, (64, 32));
        assert_eq!(pass1.dst_size, (32, 16));
        assert_eq!(pass1.mode, ScaleMode::Downsample);
        assert_eq!(pass1.scale, 2);
        assert_eq!(pass1.dst_scissor, None);
    }

    #[test]
    fn compile_for_scene_blur_emits_separable_passes() {
        let encoding = SceneEncoding::default();
        let viewport_size = (128, 64);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Blur {
                radius: 2,
                downsample_scale: 2,
                scissor: None,
            },
        );

        assert_eq!(plan.passes.len(), 7);
        let RenderPlanPass::SceneDrawRange(scene) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(half) = &plan.passes[1] else {
            panic!("expected half downsample pass");
        };
        assert_eq!(half.src, PlanTarget::Intermediate0);
        assert_eq!(half.dst, PlanTarget::Intermediate2);
        assert_eq!(half.src_size, viewport_size);
        assert_eq!(half.dst_size, (64, 32));
        assert_eq!(half.mode, ScaleMode::Downsample);
        assert_eq!(half.scale, 2);
        assert_eq!(half.dst_scissor, None);

        let RenderPlanPass::ReleaseTarget(release) = &plan.passes[2] else {
            panic!("expected ReleaseTarget pass");
        };
        assert_eq!(*release, PlanTarget::Intermediate0);

        let RenderPlanPass::Blur(blur_h) = &plan.passes[3] else {
            panic!("expected blur-h pass");
        };
        assert_eq!(blur_h.axis, BlurAxis::Horizontal);
        assert_eq!(blur_h.src, PlanTarget::Intermediate2);
        assert_eq!(blur_h.dst, PlanTarget::Intermediate1);
        assert_eq!(blur_h.src_size, (64, 32));
        assert_eq!(blur_h.dst_size, (64, 32));
        assert_eq!(blur_h.dst_scissor, None);

        let RenderPlanPass::Blur(blur_v) = &plan.passes[4] else {
            panic!("expected blur-v pass");
        };
        assert_eq!(blur_v.axis, BlurAxis::Vertical);
        assert_eq!(blur_v.src, PlanTarget::Intermediate1);
        assert_eq!(blur_v.dst, PlanTarget::Intermediate2);
        assert_eq!(blur_v.src_size, (64, 32));
        assert_eq!(blur_v.dst_size, (64, 32));
        assert_eq!(blur_v.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(upscale) = &plan.passes[5] else {
            panic!("expected upscale pass");
        };
        assert_eq!(upscale.src, PlanTarget::Intermediate2);
        assert_eq!(upscale.dst, PlanTarget::Intermediate0);
        assert_eq!(upscale.src_size, (64, 32));
        assert_eq!(upscale.dst_size, viewport_size);
        assert_eq!(upscale.mode, ScaleMode::Upscale);
        assert_eq!(upscale.scale, 2);
        assert_eq!(upscale.dst_scissor, None);

        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[6] else {
            panic!("expected blit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate0);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, viewport_size);
        assert_eq!(blit.dst_size, viewport_size);
        assert_eq!(blit.dst_scissor, None);
    }

    #[test]
    fn blur_scissor_is_mapped_per_pass_dst_size() {
        let encoding = SceneEncoding::default();
        let viewport_size = (100, 100);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Blur {
                radius: 2,
                downsample_scale: 2,
                scissor: Some(ScissorRect {
                    x: 10,
                    y: 10,
                    w: 50,
                    h: 50,
                }),
            },
        );

        // Half target is (50, 50) for 100x100.
        let RenderPlanPass::ScaleNearest(half) = &plan.passes[1] else {
            panic!("expected half downsample pass");
        };
        assert_eq!(
            half.dst_scissor,
            Some(ScissorRect {
                x: 5,
                y: 5,
                w: 25,
                h: 25
            })
        );
        let RenderPlanPass::Blur(blur_h) = &plan.passes[3] else {
            panic!("expected blur-h pass");
        };
        assert_eq!(
            blur_h.dst_scissor,
            Some(ScissorRect {
                x: 5,
                y: 5,
                w: 25,
                h: 25
            })
        );
        let RenderPlanPass::FullscreenBlit(blit) = &plan.passes[6] else {
            panic!("expected blit pass");
        };
        assert_eq!(
            blit.dst_scissor,
            Some(ScissorRect {
                x: 10,
                y: 10,
                w: 50,
                h: 50
            })
        );
    }
}
