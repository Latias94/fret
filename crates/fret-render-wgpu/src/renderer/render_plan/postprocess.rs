use super::*;
use crate::renderer::frame_targets::downsampled_size;
use crate::renderer::render_plan_effects::{map_scissor_downsample_nearest, map_scissor_to_size};

fn decompose_pixelate_scale(scale: u32) -> Vec<u32> {
    let mut scale = scale.max(1);
    let mut steps = Vec::new();
    while scale >= 4 && scale.is_multiple_of(2) {
        steps.push(2);
        scale /= 2;
    }
    steps.push(scale.max(1));
    steps
}

type DownsampleChainEntry = ((u32, u32), u32);
type DownsampleChainResult = (PlanTarget, (u32, u32), Vec<DownsampleChainEntry>);

fn push_scale_nearest(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    mode: ScaleMode,
    scale: u32,
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes
        .push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
            src,
            dst,
            src_size,
            dst_size,
            src_origin: (0, 0),
            dst_scissor: dst_scissor.map(LocalScissorRect),
            dst_origin: (0, 0),
            mask_uniform_index: None,
            mask: None,
            mode,
            scale,
            load,
        }));
}

fn push_fullscreen_blit(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    encode_output_srgb: bool,
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes
        .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src,
            dst,
            src_size,
            dst_size,
            dst_scissor: dst_scissor.map(LocalScissorRect),
            encode_output_srgb,
            load,
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
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes.push(RenderPlanPass::Blur(BlurPass {
        src,
        dst,
        src_size,
        dst_size,
        dst_scissor: dst_scissor.map(LocalScissorRect),
        mask_uniform_index: None,
        mask: None,
        axis,
        load,
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
) -> DownsampleChainResult {
    let mut stack: Vec<DownsampleChainEntry> = Vec::with_capacity(steps.len());
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
            wgpu::LoadOp::Clear(clear),
        );
        stack.push((current_size, step));
        current_target = dst_a;
        current_size = dst_size;
        std::mem::swap(&mut dst_a, &mut dst_b);
    }
    (current_target, current_size, stack)
}

#[derive(Debug, Clone)]
pub(super) struct DownsampleHalfQuarter {
    pub(super) half_target: PlanTarget,
    pub(super) quarter_target: PlanTarget,
    pub(super) quarter_size: (u32, u32),
    pub(super) stack: Vec<((u32, u32), u32)>,
}

pub(super) fn append_downsample_half_quarter(
    plan: &mut RenderPlan,
    src_target: PlanTarget,
    src_size: (u32, u32),
    half_target: PlanTarget,
    quarter_target: PlanTarget,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
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
        wgpu::LoadOp::Clear(clear),
    );

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
        wgpu::LoadOp::Clear(clear),
    );

    DownsampleHalfQuarter {
        half_target,
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
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                unreachable!("upsample chain must read from Intermediate1/2")
            }
            PlanTarget::Intermediate0 | PlanTarget::Intermediate3 | PlanTarget::Output => {
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
            wgpu::LoadOp::Clear(clear),
        );
        current_target = dst_target;
        current_size = dst_size;
    }
    (current_target, current_size)
}

pub(super) fn append_postprocess(
    plan: &mut RenderPlan,
    viewport_size: (u32, u32),
    postprocess: DebugPostprocess,
    clear: wgpu::Color,
    format: wgpu::TextureFormat,
) {
    let encode_output_srgb = output_requires_explicit_srgb_encode(format);
    match postprocess {
        DebugPostprocess::None => {}
        DebugPostprocess::OffscreenBlit { src } => {
            push_fullscreen_blit(
                plan,
                src,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                None,
                encode_output_srgb,
                wgpu::LoadOp::Clear(clear),
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
                        wgpu::LoadOp::Clear(clear),
                    );
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
                encode_output_srgb,
                wgpu::LoadOp::Clear(clear),
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

            let down_scissor = map_scissor_downsample_nearest(scissor, downsample_scale, blur_size);
            push_scale_nearest(
                plan,
                PlanTarget::Intermediate0,
                blur_src,
                viewport_size,
                blur_size,
                down_scissor,
                ScaleMode::Downsample,
                downsample_scale,
                wgpu::LoadOp::Clear(clear),
            );

            let blur_scissor = down_scissor;
            push_blur(
                plan,
                blur_src,
                scratch,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Horizontal,
                wgpu::LoadOp::Clear(clear),
            );
            push_blur(
                plan,
                scratch,
                blur_src,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Vertical,
                wgpu::LoadOp::Clear(clear),
            );

            let final_scissor = map_scissor_to_size(scissor, viewport_size, viewport_size);
            if scissor.is_some() {
                push_scale_nearest(
                    plan,
                    blur_src,
                    PlanTarget::Intermediate0,
                    blur_size,
                    viewport_size,
                    final_scissor,
                    ScaleMode::Upscale,
                    downsample_scale,
                    wgpu::LoadOp::Load,
                );
                push_fullscreen_blit(
                    plan,
                    PlanTarget::Intermediate0,
                    PlanTarget::Output,
                    viewport_size,
                    viewport_size,
                    None,
                    encode_output_srgb,
                    wgpu::LoadOp::Clear(clear),
                );
            } else {
                push_scale_nearest(
                    plan,
                    blur_src,
                    PlanTarget::Intermediate0,
                    blur_size,
                    viewport_size,
                    final_scissor,
                    ScaleMode::Upscale,
                    downsample_scale,
                    wgpu::LoadOp::Clear(clear),
                );
                push_fullscreen_blit(
                    plan,
                    PlanTarget::Intermediate0,
                    PlanTarget::Output,
                    viewport_size,
                    viewport_size,
                    final_scissor,
                    encode_output_srgb,
                    wgpu::LoadOp::Clear(clear),
                );
            }
        }
    }
}
