use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::{
    ensure_color_dst_view_owned, require_color_src_view, require_mask_view,
};

pub(in super::super) fn record_blur_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &BlurPass,
) {
    let device = exec.device;
    let format = exec.format;
    let target_view = exec.target_view;
    let viewport_size = exec.viewport_size;
    let usage = exec.usage;
    let encoder = &mut *exec.encoder;
    let frame_targets = &mut *exec.frame_targets;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    let Some(src_view) = require_color_src_view(frame_targets, pass.src, pass.src_size, "Blur")
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
        "Blur",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

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
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) = require_mask_view(frame_targets, mask.target, mask.size, "Blur")
        else {
            return;
        };
        let layout = renderer
            .blit_mask_bind_group_layout
            .as_ref()
            .expect("blit mask bind group layout must exist");
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

        let (pipeline, label) = match pass.axis {
            BlurAxis::Horizontal => (
                renderer
                    .blur_h_mask_pipeline
                    .as_ref()
                    .expect("blur-h mask pipeline must exist"),
                "fret blur-h mask pass",
            ),
            BlurAxis::Vertical => (
                renderer
                    .blur_v_mask_pipeline
                    .as_ref()
                    .expect("blur-v mask pipeline must exist"),
                "fret blur-v mask pass",
            ),
        };

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
        );
    } else if let Some(mask_uniform_index) = pass.mask_uniform_index {
        let layout = renderer
            .blit_bind_group_layout
            .as_ref()
            .expect("blit bind group layout must exist");
        let bind_group =
            create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
        let (pipeline, label) = match pass.axis {
            BlurAxis::Horizontal => (
                renderer
                    .blur_h_masked_pipeline
                    .as_ref()
                    .expect("blur-h masked pipeline must exist"),
                "fret blur-h masked pass",
            ),
            BlurAxis::Vertical => (
                renderer
                    .blur_v_masked_pipeline
                    .as_ref()
                    .expect("blur-v masked pipeline must exist"),
                "fret blur-v masked pass",
            ),
        };
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
        );
    } else {
        let layout = renderer
            .blit_bind_group_layout
            .as_ref()
            .expect("blit bind group layout must exist");
        let bind_group =
            create_texture_bind_group(device, "fret blur bind group", layout, &src_view);
        let blur_pipeline = match pass.axis {
            BlurAxis::Horizontal => renderer
                .blur_h_pipeline
                .as_ref()
                .expect("blur-h pipeline must exist"),
            BlurAxis::Vertical => renderer
                .blur_v_pipeline
                .as_ref()
                .expect("blur-v pipeline must exist"),
        };
        let label = match pass.axis {
            BlurAxis::Horizontal => "fret blur-h pass",
            BlurAxis::Vertical => "fret blur-v pass",
        };
        run_fullscreen_triangle_pass(
            encoder,
            label,
            blur_pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            if perf_enabled { Some(frame_perf) } else { None },
        );
    }
}
