use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::{
    ensure_color_dst_view_owned, ensure_mask_dst_view, require_color_src_view, require_mask_view,
    run_composite_premul_quad_pass,
};

struct FullscreenEffectLabels {
    bind_group: &'static str,
    bind_group_mask: &'static str,
    pass_unmasked: &'static str,
    pass_masked: &'static str,
    pass_mask: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn record_fullscreen_param_effect_pass<
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

    {
        let buffer = param_buffer(renderer);
        queue.write_buffer(buffer, 0, param_bytes);
    }
    if perf_enabled {
        frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(param_uniform_bytes);
    }

    let Some(src_view) = require_color_src_view(frame_targets, src, src_size, pass_name) else {
        return;
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        frame_targets,
        &mut renderer.intermediate_pool,
        device,
        dst,
        dst_size,
        format,
        usage,
        pass_name,
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let uniform_stride = renderer.uniforms.uniform_stride;

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

        let Some(mask_view) = require_mask_view(frame_targets, mask.target, mask.size, pass_name)
        else {
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
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
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
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
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
        let pipeline = pipeline_unmasked(renderer);
        run_fullscreen_triangle_pass(
            encoder,
            labels.pass_unmasked,
            pipeline,
            dst_view,
            dst_size,
            load,
            &bind_group,
            &[],
            dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn record_fullscreen_texture_effect_pass<
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

    let Some(src_view) = require_color_src_view(frame_targets, src, src_size, pass_name) else {
        return;
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        frame_targets,
        &mut renderer.intermediate_pool,
        device,
        dst,
        dst_size,
        format,
        usage,
        pass_name,
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

    let uniform_stride = renderer.uniforms.uniform_stride;

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

        let Some(mask_view) = require_mask_view(frame_targets, mask.target, mask.size, pass_name)
        else {
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
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
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
            encoder,
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
            if perf_enabled { Some(frame_perf) } else { None },
        );
    } else {
        let layout = layout_unmasked(renderer);
        let bind_group = create_texture_bind_group(device, labels.bind_group, layout, &src_view);
        let pipeline = pipeline_unmasked(renderer);
        run_fullscreen_triangle_pass(
            encoder,
            labels.pass_unmasked,
            pipeline,
            dst_view,
            dst_size,
            load,
            &bind_group,
            &[],
            dst_scissor,
            perf_enabled.then_some(frame_perf),
        );
    }
}

pub(in super::super) fn record_color_adjust_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ColorAdjustPass,
) {
    let packed = [pass.saturation, pass.brightness, pass.contrast, 0.0];
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "ColorAdjust",
        FullscreenEffectLabels {
            bind_group: "fret color-adjust bind group",
            bind_group_mask: "fret color-adjust mask bind group",
            pass_unmasked: "fret color-adjust pass",
            pass_masked: "fret color-adjust masked pass",
            pass_mask: "fret color-adjust mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        bytemuck::cast_slice(&packed),
        std::mem::size_of_val(&packed) as u64,
        |r| &r.effect_params.color_adjust_param_buffer,
        |r| r.color_adjust_bind_group_layout_ref(),
        |r| r.color_adjust_mask_bind_group_layout_ref(),
        |r| r.color_adjust_pipeline_ref(),
        |r| r.color_adjust_masked_pipeline_ref(),
        |r| r.color_adjust_mask_pipeline_ref(),
        None,
        Some("mask-based color-adjust expects full-size destination"),
    );
}

pub(in super::super) fn record_alpha_threshold_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &AlphaThresholdPass,
) {
    let packed = [pass.cutoff, pass.soft, 0.0, 0.0];
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "AlphaThreshold",
        FullscreenEffectLabels {
            bind_group: "fret alpha-threshold bind group",
            bind_group_mask: "fret alpha-threshold mask bind group",
            pass_unmasked: "fret alpha-threshold pass",
            pass_masked: "fret alpha-threshold masked pass",
            pass_mask: "fret alpha-threshold mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        bytemuck::cast_slice(&packed),
        std::mem::size_of_val(&packed) as u64,
        |r| &r.effect_params.alpha_threshold_param_buffer,
        |r| r.alpha_threshold_bind_group_layout_ref(),
        |r| r.alpha_threshold_mask_bind_group_layout_ref(),
        |r| r.alpha_threshold_pipeline_ref(),
        |r| r.alpha_threshold_masked_pipeline_ref(),
        |r| r.alpha_threshold_mask_pipeline_ref(),
        None,
        Some("mask-based alpha-threshold expects full-size destination"),
    );
}

pub(in super::super) fn record_color_matrix_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &ColorMatrixPass,
) {
    let m = pass.matrix;
    let packed: [f32; 20] = [
        // row0 (col0..3)
        m[0], m[1], m[2], m[3], // row1 (col0..3)
        m[5], m[6], m[7], m[8], // row2 (col0..3)
        m[10], m[11], m[12], m[13], // row3 (col0..3)
        m[15], m[16], m[17], m[18], // bias (col4)
        m[4], m[9], m[14], m[19],
    ];
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "ColorMatrix",
        FullscreenEffectLabels {
            bind_group: "fret color-matrix bind group",
            bind_group_mask: "fret color-matrix mask bind group",
            pass_unmasked: "fret color-matrix pass",
            pass_masked: "fret color-matrix masked pass",
            pass_mask: "fret color-matrix mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        bytemuck::cast_slice(&packed),
        std::mem::size_of_val(&packed) as u64,
        |r| &r.effect_params.color_matrix_param_buffer,
        |r| r.color_matrix_bind_group_layout_ref(),
        |r| r.color_matrix_mask_bind_group_layout_ref(),
        |r| r.color_matrix_pipeline_ref(),
        |r| r.color_matrix_masked_pipeline_ref(),
        |r| r.color_matrix_mask_pipeline_ref(),
        None,
        Some("mask-based color-matrix expects full-size destination"),
    );
}

pub(in super::super) fn record_dither_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &DitherPass,
) {
    debug_assert!(matches!(pass.mode, fret_core::DitherMode::Bayer4x4));

    record_fullscreen_texture_effect_pass(
        exec,
        ctx,
        "Dither",
        FullscreenEffectLabels {
            bind_group: "fret dither bind group",
            bind_group_mask: "fret dither mask bind group",
            pass_unmasked: "fret dither pass",
            pass_masked: "fret dither masked pass",
            pass_mask: "fret dither mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        |r| r.dither_bind_group_layout_ref(),
        |r| r.dither_mask_bind_group_layout_ref(),
        |r| r.dither_pipeline_ref(),
        |r| r.dither_masked_pipeline_ref(),
        |r| r.dither_mask_pipeline_ref(),
        Some("clip-based dither expects full-size destination"),
        Some("mask-based dither expects full-size destination"),
    );
}

pub(in super::super) fn record_noise_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &NoisePass,
) {
    let packed = [pass.strength, pass.scale_px, pass.phase, 0.0];
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "Noise",
        FullscreenEffectLabels {
            bind_group: "fret noise bind group",
            bind_group_mask: "fret noise mask bind group",
            pass_unmasked: "fret noise pass",
            pass_masked: "fret noise masked pass",
            pass_mask: "fret noise mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        bytemuck::cast_slice(&packed),
        std::mem::size_of_val(&packed) as u64,
        |r| &r.effect_params.noise_param_buffer,
        |r| r.noise_bind_group_layout_ref(),
        |r| r.noise_mask_bind_group_layout_ref(),
        |r| r.noise_pipeline_ref(),
        |r| r.noise_masked_pipeline_ref(),
        |r| r.noise_mask_pipeline_ref(),
        Some("clip-based noise expects full-size destination"),
        Some("mask-based noise expects full-size destination"),
    );
}

pub(in super::super) fn record_drop_shadow_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &DropShadowPass,
) {
    let packed = [
        pass.offset_px.0,
        pass.offset_px.1,
        0.0,
        0.0,
        pass.color.r,
        pass.color.g,
        pass.color.b,
        pass.color.a,
    ];
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "DropShadow",
        FullscreenEffectLabels {
            bind_group: "fret drop-shadow bind group",
            bind_group_mask: "fret drop-shadow mask bind group",
            pass_unmasked: "fret drop-shadow pass",
            pass_masked: "fret drop-shadow masked pass",
            pass_mask: "fret drop-shadow mask pass",
        },
        pass.src,
        pass.dst,
        pass.src_size,
        pass.dst_size,
        pass.dst_scissor,
        pass.mask_uniform_index,
        pass.mask,
        pass.load,
        bytemuck::cast_slice(&packed),
        std::mem::size_of_val(&packed) as u64,
        |r| &r.effect_params.drop_shadow_param_buffer,
        |r| r.drop_shadow_bind_group_layout_ref(),
        |r| r.drop_shadow_mask_bind_group_layout_ref(),
        |r| r.drop_shadow_pipeline_ref(),
        |r| r.drop_shadow_masked_pipeline_ref(),
        |r| r.drop_shadow_mask_pipeline_ref(),
        Some("clip-based drop-shadow expects full-size destination"),
        Some("mask-based drop-shadow expects full-size destination"),
    );
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
