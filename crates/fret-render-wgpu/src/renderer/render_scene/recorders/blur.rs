use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};

pub(in super::super) fn record_blur_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &BlurPass,
) {
    let device = exec.device;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;

    let Some(src_view) = exec.require_color_src_view(pass.src, pass.src_size, "Blur") else {
        return;
    };

    let dst_view_owned = exec.ensure_color_dst_view_owned(pass.dst, pass.dst_size, "Blur");
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);
    let renderer = &*exec.renderer;

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based blur expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask blur needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;

        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, "Blur") else {
            return;
        };
        let layout = renderer.blit_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret blur mask bind group"),
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

        let pipeline = renderer.blur_mask_pipeline_ref(pass.axis);
        let label = match pass.axis {
            BlurAxis::Horizontal => "fret blur-h mask pass",
            BlurAxis::Vertical => "fret blur-v mask pass",
        };

        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            label,
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
            &[],
            pass.dst_scissor,
            pass.dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
        let layout = renderer.blit_bind_group_layout_ref();
        let bind_group =
            create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
        let pipeline = renderer.blur_masked_pipeline_ref(pass.axis);
        let label = match pass.axis {
            BlurAxis::Horizontal => "fret blur-h masked pass",
            BlurAxis::Vertical => "fret blur-v masked pass",
        };
        let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            label,
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
            &[],
            pass.dst_scissor,
            pass.dst_size,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    } else {
        let layout = renderer.blit_bind_group_layout_ref();
        let bind_group =
            create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
        let blur_pipeline = renderer.blur_pipeline_ref(pass.axis);
        let label = match pass.axis {
            BlurAxis::Horizontal => "fret blur-h pass",
            BlurAxis::Vertical => "fret blur-v pass",
        };
        run_fullscreen_triangle_pass(
            &mut *exec.encoder,
            label,
            blur_pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            if perf_enabled {
                Some(&mut *exec.frame_perf)
            } else {
                None
            },
        );
    }
}
