use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};

pub(super) struct FullscreenEffectLabels {
    pub(super) bind_group: &'static str,
    pub(super) bind_group_mask: &'static str,
    pub(super) pass_unmasked: &'static str,
    pub(super) pass_masked: &'static str,
    pub(super) pass_mask: &'static str,
}

pub(super) fn pack_effect_params_v1(params: fret_core::EffectParamsV1) -> [u8; 64] {
    let mut out = [0u8; 64];
    let mut cursor = 0usize;
    for v in params.vec4s {
        for x in v {
            out[cursor..cursor + 4].copy_from_slice(&x.to_bits().to_le_bytes());
            cursor += 4;
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_fullscreen_param_effect_pass<
    ParamBuffer,
    BindGroupLayoutUnmasked,
    BindGroupLayoutMask,
    PipelineUnmasked,
    PipelineMasked,
    PipelineMask,
>(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass_name: &'static str,
    labels: FullscreenEffectLabels,
    unmasked_uses_uniforms: bool,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<LocalScissorRect>,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
    load: wgpu::LoadOp<wgpu::Color>,
    param_bytes: &[u8],
    param_uniform_bytes: u64,
    param_buffer: ParamBuffer,
    layout_unmasked: BindGroupLayoutUnmasked,
    layout_mask: BindGroupLayoutMask,
    pipeline_unmasked: PipelineUnmasked,
    pipeline_masked: PipelineMasked,
    pipeline_mask: PipelineMask,
    clip_full_size_msg: Option<&'static str>,
    mask_full_size_msg: Option<&'static str>,
) where
    ParamBuffer: for<'a> Fn(&'a Renderer) -> &'a wgpu::Buffer,
    BindGroupLayoutUnmasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::BindGroupLayout,
    BindGroupLayoutMask: for<'a> Fn(&'a Renderer) -> &'a wgpu::BindGroupLayout,
    PipelineUnmasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
    PipelineMasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
    PipelineMask: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
{
    let device = exec.device;
    let queue = exec.queue;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;

    {
        let buffer = param_buffer(&*exec.renderer);
        queue.write_buffer(buffer, 0, param_bytes);
    }
    if perf_enabled {
        exec.frame_perf.uniform_bytes = exec
            .frame_perf
            .uniform_bytes
            .saturating_add(param_uniform_bytes);
    }

    let Some(src_view) = exec.require_color_src_view(src, src_size, pass_name) else {
        return;
    };

    let dst_view_owned = exec.ensure_color_dst_view_owned(dst, dst_size, pass_name);
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let renderer = &*exec.renderer;
    let uniform_stride = renderer.uniform_stride();

    if let Some(mask) = mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        if let Some(msg) = mask_full_size_msg {
            debug_assert_eq!(dst_size, viewport_size, "{msg}");
        }

        let mask_uniform_index = mask_uniform_index.expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;

        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, pass_name) else {
            return;
        };

        let layout = layout_mask(renderer);
        let buffer = param_buffer(renderer);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(labels.bind_group_mask),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = pipeline_mask(renderer);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            labels.pass_mask,
            pipeline,
            dst_view,
            load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            dst_scissor,
            dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else if let Some(mask_uniform_index) = mask_uniform_index {
        if let Some(msg) = clip_full_size_msg {
            debug_assert_eq!(dst_size, viewport_size, "{msg}");
        }

        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let layout = layout_unmasked(renderer);
        let buffer = param_buffer(renderer);
        let bind_group = create_texture_uniform_bind_group(
            device,
            labels.bind_group,
            layout,
            &src_view,
            buffer.as_entire_binding(),
        );
        let pipeline = pipeline_masked(renderer);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            labels.pass_masked,
            pipeline,
            dst_view,
            load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            dst_scissor,
            dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else {
        let layout = layout_unmasked(renderer);
        let buffer = param_buffer(renderer);
        let bind_group = create_texture_uniform_bind_group(
            device,
            labels.bind_group,
            layout,
            &src_view,
            buffer.as_entire_binding(),
        );

        if unmasked_uses_uniforms {
            let pipeline = pipeline_unmasked(renderer);
            run_fullscreen_triangle_pass_uniform_texture(
                &mut *exec.encoder,
                labels.pass_unmasked,
                pipeline,
                dst_view,
                load,
                renderer.pick_uniform_bind_group_for_mask_image(None),
                &[0, ctx.render_space_offset_u32],
                &bind_group,
                &[],
                dst_scissor,
                dst_size,
                if perf_enabled {
                    Some(&mut *exec.frame_perf)
                } else {
                    None
                },
            );
        } else {
            let pipeline = pipeline_unmasked(renderer);
            run_fullscreen_triangle_pass(
                &mut *exec.encoder,
                labels.pass_unmasked,
                pipeline,
                dst_view,
                dst_size,
                load,
                &bind_group,
                &[],
                dst_scissor,
                if perf_enabled {
                    Some(&mut *exec.frame_perf)
                } else {
                    None
                },
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_fullscreen_texture_effect_pass<
    BindGroupLayoutUnmasked,
    BindGroupLayoutMask,
    PipelineUnmasked,
    PipelineMasked,
    PipelineMask,
>(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass_name: &'static str,
    labels: FullscreenEffectLabels,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<LocalScissorRect>,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
    load: wgpu::LoadOp<wgpu::Color>,
    layout_unmasked: BindGroupLayoutUnmasked,
    layout_mask: BindGroupLayoutMask,
    pipeline_unmasked: PipelineUnmasked,
    pipeline_masked: PipelineMasked,
    pipeline_mask: PipelineMask,
    clip_full_size_msg: Option<&'static str>,
    mask_full_size_msg: Option<&'static str>,
) where
    BindGroupLayoutUnmasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::BindGroupLayout,
    BindGroupLayoutMask: for<'a> Fn(&'a Renderer) -> &'a wgpu::BindGroupLayout,
    PipelineUnmasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
    PipelineMasked: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
    PipelineMask: for<'a> Fn(&'a Renderer) -> &'a wgpu::RenderPipeline,
{
    let device = exec.device;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;

    let Some(src_view) = exec.require_color_src_view(src, src_size, pass_name) else {
        return;
    };

    let dst_view_owned = exec.ensure_color_dst_view_owned(dst, dst_size, pass_name);
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let renderer = &*exec.renderer;
    let uniform_stride = renderer.uniform_stride();

    if let Some(mask) = mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        if let Some(msg) = mask_full_size_msg {
            debug_assert_eq!(dst_size, viewport_size, "{msg}");
        }

        let mask_uniform_index = mask_uniform_index.expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;

        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, pass_name) else {
            return;
        };

        let layout = layout_mask(renderer);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(labels.bind_group_mask),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = pipeline_mask(renderer);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            labels.pass_mask,
            pipeline,
            dst_view,
            load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            dst_scissor,
            dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else if let Some(mask_uniform_index) = mask_uniform_index {
        if let Some(msg) = clip_full_size_msg {
            debug_assert_eq!(dst_size, viewport_size, "{msg}");
        }

        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let layout = layout_unmasked(renderer);
        let bind_group = create_texture_bind_group(device, labels.bind_group, layout, &src_view);
        let pipeline = pipeline_masked(renderer);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            labels.pass_masked,
            pipeline,
            dst_view,
            load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            dst_scissor,
            dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else {
        let layout = layout_unmasked(renderer);
        let bind_group = create_texture_bind_group(device, labels.bind_group, layout, &src_view);
        let pipeline = pipeline_unmasked(renderer);
        run_fullscreen_triangle_pass(
            &mut *exec.encoder,
            labels.pass_unmasked,
            pipeline,
            dst_view,
            dst_size,
            load,
            &bind_group,
            &[],
            dst_scissor,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    }
}
