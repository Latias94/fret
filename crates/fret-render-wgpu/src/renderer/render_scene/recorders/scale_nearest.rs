use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};

pub(in super::super) fn record_scale_nearest_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ScaleNearestPass,
) {
    let device = exec.device;
    let queue = exec.queue;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let encoding = exec.encoding;
    let scale_param_size = exec.scale_param_size;
    let perf_enabled = exec.perf_enabled;

    let scale = pass.scale.max(1);
    let Some(src_view) = exec.require_color_src_view(pass.src, pass.src_size, "ScaleNearest")
    else {
        return;
    };
    let dst_view_owned = exec.ensure_color_dst_view_owned(pass.dst, pass.dst_size, "ScaleNearest");
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let renderer = &*exec.renderer;
    let scale_param_cursor = &mut *exec.scale_param_cursor;
    let scale_param_offset =
        u64::from(*scale_param_cursor) * renderer.effect_params.scale_param_stride;
    let scale_param_offset_u32 = scale_param_offset as u32;
    *scale_param_cursor = scale_param_cursor.saturating_add(1);
    let params = ScaleParamsUniform {
        scale,
        _pad0: 0,
        src_origin: [pass.src_origin.0, pass.src_origin.1],
        dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
        _pad1: 0,
        _pad2: 0,
    };
    queue.write_buffer(
        &renderer.effect_params.scale_param_buffer,
        scale_param_offset,
        bytemuck::bytes_of(&params),
    );
    if perf_enabled {
        exec.frame_perf.uniform_bytes = exec
            .frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<ScaleParamsUniform>() as u64);
    }
    let scale_param_size_nz =
        std::num::NonZeroU64::new(scale_param_size).expect("scale params size");
    let scale_param_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: &renderer.effect_params.scale_param_buffer,
        offset: 0,
        size: Some(scale_param_size_nz),
    });

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(pass.mode, ScaleMode::Upscale));
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based scale-nearest expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;

        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, "ScaleNearest") else {
            return;
        };
        let mask_layout = renderer.scale_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret scale-nearest mask bind group"),
            layout: mask_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scale_param_binding,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = renderer.upscale_mask_pipeline_ref();
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret upscale-nearest mask pass",
            pipeline,
            dst_view,
            pass.load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[scale_param_offset_u32],
            pass.dst_scissor,
            pass.dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
        debug_assert!(matches!(pass.mode, ScaleMode::Upscale));
        let pipeline = renderer.upscale_masked_pipeline_ref();
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;

        let layout = renderer.scale_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret scale-nearest bind group",
            layout,
            &src_view,
            scale_param_binding,
        );

        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret upscale-nearest masked pass",
            pipeline,
            dst_view,
            pass.load,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[scale_param_offset_u32],
            pass.dst_scissor,
            pass.dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else {
        let layout = renderer.scale_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret scale-nearest bind group",
            layout,
            &src_view,
            scale_param_binding,
        );
        let pipeline = renderer.scale_nearest_pipeline_ref(pass.mode);
        let label = match pass.mode {
            ScaleMode::Downsample => "fret downsample-nearest pass",
            ScaleMode::Upscale => "fret upscale-nearest pass",
        };
        run_fullscreen_triangle_pass(
            &mut *exec.encoder,
            label,
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[scale_param_offset_u32],
            pass.dst_scissor,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    }
}
