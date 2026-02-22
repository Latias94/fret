use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::{
    ensure_color_dst_view_owned, require_color_src_view, require_mask_view,
};

pub(in super::super) fn record_scale_nearest_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ScaleNearestPass,
) {
    let device = exec.device;
    let queue = exec.queue;
    let format = exec.format;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let usage = exec.usage;
    let encoder = &mut *exec.encoder;
    let frame_targets = &mut *exec.frame_targets;
    let encoding = exec.encoding;
    let scale_param_size = exec.scale_param_size;
    let scale_param_cursor = &mut *exec.scale_param_cursor;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    let scale = pass.scale.max(1);
    let scale_param_offset = u64::from(*scale_param_cursor) * renderer.scale_param_stride;
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
        &renderer.scale_param_buffer,
        scale_param_offset,
        bytemuck::bytes_of(&params),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<ScaleParamsUniform>() as u64);
    }
    let scale_param_size_nz =
        std::num::NonZeroU64::new(scale_param_size).expect("scale params size");
    let scale_param_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: &renderer.scale_param_buffer,
        offset: 0,
        size: Some(scale_param_size_nz),
    });

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "ScaleNearest")
    else {
        return;
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        frame_targets,
        &mut renderer.intermediate_pool,
        device,
        pass.dst,
        pass.dst_size,
        format,
        usage,
        "ScaleNearest",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

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
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride) as u32;

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "ScaleNearest")
        else {
            return;
        };
        let mask_layout = renderer
            .scale_mask_bind_group_layout
            .as_ref()
            .expect("scale mask bind group layout must exist");
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

        let pipeline = renderer
            .upscale_mask_pipeline
            .as_ref()
            .expect("upscale mask pipeline must exist");
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fret upscale-nearest mask pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: pass.load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rp.set_pipeline(pipeline);
        if perf_enabled {
            frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
            frame_perf.pipeline_switches_fullscreen =
                frame_perf.pipeline_switches_fullscreen.saturating_add(1);
        }
        rp.set_bind_group(
            0,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
        );
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
        }
        rp.set_bind_group(1, &bind_group, &[scale_param_offset_u32]);
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
        }
        if let Some(scissor) = pass.dst_scissor
            && scissor.w != 0
            && scissor.h != 0
        {
            rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
            if perf_enabled {
                frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
            }
        }
        rp.draw(0..3, 0..1);
        if perf_enabled {
            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
            frame_perf.fullscreen_draw_calls = frame_perf.fullscreen_draw_calls.saturating_add(1);
        }
    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
        debug_assert!(matches!(pass.mode, ScaleMode::Upscale));
        let pipeline = renderer
            .upscale_masked_pipeline
            .as_ref()
            .expect("upscale masked pipeline must exist");
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride) as u32;

        let layout = renderer
            .scale_bind_group_layout
            .as_ref()
            .expect("scale bind group layout must exist");
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret scale-nearest bind group",
            layout,
            &src_view,
            scale_param_binding,
        );

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fret upscale-nearest masked pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: pass.load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rp.set_pipeline(pipeline);
        if perf_enabled {
            frame_perf.pipeline_switches = frame_perf.pipeline_switches.saturating_add(1);
            frame_perf.pipeline_switches_fullscreen =
                frame_perf.pipeline_switches_fullscreen.saturating_add(1);
        }
        rp.set_bind_group(
            0,
            renderer.pick_uniform_bind_group_for_mask_image(
                encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
        );
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.uniform_bind_group_switches =
                frame_perf.uniform_bind_group_switches.saturating_add(1);
        }
        rp.set_bind_group(1, &bind_group, &[scale_param_offset_u32]);
        if perf_enabled {
            frame_perf.bind_group_switches = frame_perf.bind_group_switches.saturating_add(1);
            frame_perf.texture_bind_group_switches =
                frame_perf.texture_bind_group_switches.saturating_add(1);
        }
        if let Some(scissor) = pass.dst_scissor
            && scissor.w != 0
            && scissor.h != 0
        {
            rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
            if perf_enabled {
                frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
            }
        }
        rp.draw(0..3, 0..1);
        if perf_enabled {
            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
            frame_perf.fullscreen_draw_calls = frame_perf.fullscreen_draw_calls.saturating_add(1);
        }
    } else {
        let layout = renderer
            .scale_bind_group_layout
            .as_ref()
            .expect("scale bind group layout must exist");
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret scale-nearest bind group",
            layout,
            &src_view,
            scale_param_binding,
        );
        let (pipeline, label) = match pass.mode {
            ScaleMode::Downsample => (
                renderer
                    .downsample_pipeline
                    .as_ref()
                    .expect("downsample pipeline must exist"),
                "fret downsample-nearest pass",
            ),
            ScaleMode::Upscale => (
                renderer
                    .upscale_pipeline
                    .as_ref()
                    .expect("upscale pipeline must exist"),
                "fret upscale-nearest pass",
            ),
        };
        run_fullscreen_triangle_pass(
            encoder,
            label,
            pipeline,
            dst_view,
            pass.load,
            &bind_group,
            &[scale_param_offset_u32],
            pass.dst_scissor,
            if perf_enabled { Some(frame_perf) } else { None },
        );
    }
}
