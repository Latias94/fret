use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::{
    ensure_color_dst_view_owned, ensure_mask_dst_view, require_color_src_view, require_mask_view,
    run_composite_premul_quad_pass,
};

pub(in super::super) fn record_color_adjust_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ColorAdjustPass,
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
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    queue.write_buffer(
        &renderer.effect_params.color_adjust_param_buffer,
        0,
        bytemuck::cast_slice(&[pass.saturation, pass.brightness, pass.contrast, 0.0]),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
    }

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "ColorAdjust")
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
        "ColorAdjust",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based color-adjust expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask color-adjust needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "ColorAdjust")
        else {
            return;
        };
        let layout = renderer.color_adjust_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret color-adjust mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer
                        .effect_params
                        .color_adjust_param_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = renderer.color_adjust_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret color-adjust mask pass",
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
        let layout = renderer.color_adjust_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret color-adjust bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .color_adjust_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.color_adjust_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret color-adjust masked pass",
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
        let layout = renderer.color_adjust_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret color-adjust bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .color_adjust_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.color_adjust_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret color-adjust pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_alpha_threshold_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &AlphaThresholdPass,
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
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    queue.write_buffer(
        &renderer.effect_params.alpha_threshold_param_buffer,
        0,
        bytemuck::cast_slice(&[pass.cutoff, pass.soft, 0.0, 0.0]),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
    }

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "AlphaThreshold")
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
        "AlphaThreshold",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based alpha-threshold expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask alpha-threshold needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "AlphaThreshold")
        else {
            return;
        };
        let layout = renderer.alpha_threshold_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret alpha-threshold mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer
                        .effect_params
                        .alpha_threshold_param_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });
        let pipeline = renderer.alpha_threshold_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret alpha-threshold mask pass",
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
        let layout = renderer.alpha_threshold_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret alpha-threshold bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .alpha_threshold_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.alpha_threshold_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret alpha-threshold masked pass",
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
        let layout = renderer.alpha_threshold_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret alpha-threshold bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .alpha_threshold_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.alpha_threshold_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret alpha-threshold pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_color_matrix_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ColorMatrixPass,
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
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    let m = pass.matrix;
    let packed: [f32; 20] = [
        // row0 (col0..3)
        m[0], m[1], m[2], m[3], // row1 (col0..3)
        m[5], m[6], m[7], m[8], // row2 (col0..3)
        m[10], m[11], m[12], m[13], // row3 (col0..3)
        m[15], m[16], m[17], m[18], // bias (col4)
        m[4], m[9], m[14], m[19],
    ];
    queue.write_buffer(
        &renderer.effect_params.color_matrix_param_buffer,
        0,
        bytemuck::cast_slice(&packed),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 20]>() as u64);
    }

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "ColorMatrix")
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
        "ColorMatrix",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based color-matrix expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask color-matrix needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "ColorMatrix")
        else {
            return;
        };
        let layout = renderer.color_matrix_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret color-matrix mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer
                        .effect_params
                        .color_matrix_param_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });
        let pipeline = renderer.color_matrix_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret color-matrix mask pass",
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
        let layout = renderer.color_matrix_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret color-matrix bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .color_matrix_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.color_matrix_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret color-matrix masked pass",
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
        let layout = renderer.color_matrix_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret color-matrix bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .color_matrix_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.color_matrix_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret color-matrix pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_dither_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &DitherPass,
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

    debug_assert!(matches!(pass.mode, fret_core::DitherMode::Bayer4x4));

    let Some(src_view) = require_color_src_view(frame_targets, pass.src, pass.src_size, "Dither")
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
        "Dither",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based dither expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask dither needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) = require_mask_view(frame_targets, mask.target, mask.size, "Dither")
        else {
            return;
        };
        let layout = renderer.dither_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret dither mask bind group"),
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

        let pipeline = renderer.dither_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret dither mask pass",
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
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "clip-based dither expects full-size destination"
        );

        let layout = renderer.dither_bind_group_layout_ref();
        let bind_group =
            create_texture_bind_group(device, "fret dither bind group", layout, &src_view);
        let pipeline = renderer.dither_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret dither masked pass",
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
        let layout = renderer.dither_bind_group_layout_ref();
        let bind_group =
            create_texture_bind_group(device, "fret dither bind group", layout, &src_view);
        let pipeline = renderer.dither_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret dither pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_noise_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &NoisePass,
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
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    queue.write_buffer(
        &renderer.effect_params.noise_param_buffer,
        0,
        bytemuck::cast_slice(&[pass.strength, pass.scale_px, pass.phase, 0.0]),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
    }

    let Some(src_view) = require_color_src_view(frame_targets, pass.src, pass.src_size, "Noise")
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
        "Noise",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based noise expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask noise needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) = require_mask_view(frame_targets, mask.target, mask.size, "Noise")
        else {
            return;
        };
        let layout = renderer.noise_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret noise mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer
                        .effect_params
                        .noise_param_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = renderer.noise_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret noise mask pass",
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
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "clip-based noise expects full-size destination"
        );

        let layout = renderer.noise_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret noise bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .noise_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.noise_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret noise masked pass",
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
        let layout = renderer.noise_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret noise bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .noise_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.noise_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret noise pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_drop_shadow_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &DropShadowPass,
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
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    queue.write_buffer(
        &renderer.effect_params.drop_shadow_param_buffer,
        0,
        bytemuck::cast_slice(&[
            pass.offset_px.0,
            pass.offset_px.1,
            0.0,
            0.0,
            pass.color.r,
            pass.color.g,
            pass.color.b,
            pass.color.a,
        ]),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 8]>() as u64);
    }

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "DropShadow")
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
        "DropShadow",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based drop-shadow expects full-size destination"
        );

        let mask_uniform_index = pass
            .mask_uniform_index
            .expect("mask drop-shadow needs uniform index");
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "DropShadow")
        else {
            return;
        };
        let layout = renderer.drop_shadow_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret drop-shadow mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: renderer
                        .effect_params
                        .drop_shadow_param_buffer
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = renderer.drop_shadow_mask_pipeline_ref();

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret drop-shadow mask pass",
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
        let layout = renderer.drop_shadow_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret drop-shadow bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .drop_shadow_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.drop_shadow_masked_pipeline_ref();
        let uniform_offset =
            (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;

        run_fullscreen_triangle_pass_uniform_texture(
            encoder,
            "fret drop-shadow masked pass",
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
        let layout = renderer.drop_shadow_bind_group_layout_ref();
        let bind_group = create_texture_uniform_bind_group(
            device,
            "fret drop-shadow bind group",
            layout,
            &src_view,
            renderer
                .effect_params
                .drop_shadow_param_buffer
                .as_entire_binding(),
        );
        let pipeline = renderer.drop_shadow_pipeline_ref();
        run_fullscreen_triangle_pass(
            encoder,
            "fret drop-shadow pass",
            pipeline,
            dst_view,
            pass.dst_size,
            pass.load,
            &bind_group,
            &[],
            pass.dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_composite_premul_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CompositePremulPass,
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
    let quad_vertex_bases = exec.quad_vertex_bases;
    let quad_vertex_size = exec.quad_vertex_size;

    let renderer = &mut *exec.renderer;

    let Some(src_view) =
        require_color_src_view(frame_targets, pass.src, pass.src_size, "CompositePremul")
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
        "CompositePremul",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let (composite_pipeline, bind_group) = if let Some(mask) = pass.mask {
        debug_assert!(matches!(
            mask.target,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2
        ));
        debug_assert_eq!(
            pass.dst_size, viewport_size,
            "mask-based composite expects full-size destination"
        );

        let Some(mask_view) =
            require_mask_view(frame_targets, mask.target, mask.size, "CompositePremul")
        else {
            return;
        };
        let layout = renderer.composite_mask_bind_group_layout_ref();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret composite premul mask bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&renderer.globals.viewport_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        (
            renderer.composite_mask_pipeline_ref(pass.blend_mode),
            bind_group,
        )
    } else {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret composite premul bind group"),
            layout: &renderer.globals.viewport_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&renderer.globals.viewport_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
            ],
        });
        (renderer.composite_pipeline_ref(pass.blend_mode), bind_group)
    };
    let Some(base) = quad_vertex_bases.get(ctx.pass_index).and_then(|v| *v) else {
        return;
    };

    let (uniform_bind_group, uniform_offsets) =
        if let Some(mask_uniform_index) = pass.mask_uniform_index {
            let uniform_offset =
                (u64::from(mask_uniform_index) * renderer.uniforms.uniform_stride) as u32;
            let mask_image = encoding
                .uniform_mask_images
                .get(mask_uniform_index as usize)
                .copied()
                .flatten();
            (
                renderer.pick_uniform_bind_group_for_mask_image(mask_image),
                [uniform_offset, ctx.render_space_offset_u32],
            )
        } else {
            (
                &renderer.uniform_bind_group,
                [0, ctx.render_space_offset_u32],
            )
        };

    let base = u64::from(base) * quad_vertex_size;
    let len = 6 * quad_vertex_size;
    run_composite_premul_quad_pass(
        encoder,
        "fret composite premul pass",
        composite_pipeline,
        dst_view,
        pass.load,
        uniform_bind_group,
        &uniform_offsets,
        &bind_group,
        &[],
        &renderer.path_composite_vertices,
        base,
        len,
        pass.dst_scissor,
        pass.dst_origin,
        pass.dst_size,
        perf_enabled.then_some(frame_perf),
    );
}

pub(in super::super) fn record_clip_mask_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ClipMaskPass,
) {
    let device = exec.device;
    let queue = exec.queue;
    let usage = exec.usage;
    let encoder = &mut *exec.encoder;
    let frame_targets = &mut *exec.frame_targets;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    queue.write_buffer(
        &renderer.effect_params.clip_mask_param_buffer,
        0,
        bytemuck::cast_slice(&[pass.dst_size.0 as f32, pass.dst_size.1 as f32, 0.0, 0.0]),
    );
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf
            .uniform_bytes
            .saturating_add(std::mem::size_of::<[f32; 4]>() as u64);
    }
    let Some(dst_view) = ensure_mask_dst_view(
        frame_targets,
        &mut renderer.intermediate_pool,
        device,
        pass.dst,
        pass.dst_size,
        usage,
        "ClipMask",
    ) else {
        return;
    };

    let pipeline = renderer.clip_mask_pipeline_ref();
    let uniform_offset = (u64::from(pass.uniform_index) * renderer.uniforms.uniform_stride) as u32;

    run_clip_mask_triangle_pass(
        encoder,
        "fret clip mask pass",
        pipeline,
        &dst_view,
        pass.load,
        renderer.pick_uniform_bind_group_for_mask_image(
            encoding
                .uniform_mask_images
                .get(pass.uniform_index as usize)
                .copied()
                .flatten(),
        ),
        &[uniform_offset, ctx.render_space_offset_u32],
        &renderer.effect_params.clip_mask_param_bind_group,
        &[],
        pass.dst_scissor,
        pass.dst_size,
        perf_enabled.then_some(frame_perf),
    );
}
