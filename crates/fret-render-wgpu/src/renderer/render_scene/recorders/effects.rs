use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::helpers::{
    ensure_color_dst_view_owned, ensure_mask_dst_view, require_color_src_view, require_mask_view,
    run_composite_premul_quad_pass,
};
use super::blit::record_fullscreen_blit_pass;

struct FullscreenEffectLabels {
    bind_group: &'static str,
    bind_group_mask: &'static str,
    pass_unmasked: &'static str,
    pass_masked: &'static str,
    pass_mask: &'static str,
}

fn pack_effect_params_v1(params: fret_core::EffectParamsV1) -> [u8; 64] {
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

fn resolve_custom_effect_filterable_user_image_view<'a>(
    renderer: &'a Renderer,
    device_features: wgpu::Features,
    image: Option<fret_core::ImageId>,
    incompatible: &mut bool,
) -> &'a wgpu::TextureView {
    let Some(id) = image else {
        return &renderer.globals.custom_effect_input_fallback_view;
    };
    let Some(view) = renderer.gpu_resources.image_view(id) else {
        return &renderer.globals.custom_effect_input_fallback_view;
    };
    let Some(format) = renderer.gpu_resources.image_format(id) else {
        return &renderer.globals.custom_effect_input_fallback_view;
    };

    let f = renderer.adapter.get_texture_format_features(format);
    let ok_usage = f
        .allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING);
    let ok_sample_type = format.sample_type(None, Some(device_features))
        == Some(wgpu::TextureSampleType::Float { filterable: true });

    if ok_usage && ok_sample_type {
        view
    } else {
        *incompatible = true;
        &renderer.globals.custom_effect_input_fallback_view
    }
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
        &mut renderer.intermediate_state.pool,
        device,
        dst,
        dst_size,
        format,
        usage,
        pass_name,
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

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

        if unmasked_uses_uniforms {
            let pipeline = pipeline_unmasked(renderer);
            run_fullscreen_triangle_pass_uniform_texture(
                encoder,
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
                if perf_enabled { Some(frame_perf) } else { None },
            );
        } else {
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
        &mut renderer.intermediate_state.pool,
        device,
        dst,
        dst_size,
        format,
        usage,
        pass_name,
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(target_view);

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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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

pub(in super::super) fn record_custom_effect_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CustomEffectPass,
) {
    let common = pass.common;
    let effect = common.effect;

    if exec
        .renderer
        .material_effect_state
        .custom_effects
        .get(effect)
        .is_none()
    {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    exec.renderer
        .ensure_custom_effect_pipelines(exec.device, exec.format, effect);
    if !exec
        .renderer
        .pipelines
        .custom_effect_pipelines
        .contains_key(&effect)
    {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    let packed = pack_effect_params_v1(common.params);
    record_fullscreen_param_effect_pass(
        exec,
        ctx,
        "CustomEffectV1",
        FullscreenEffectLabels {
            bind_group: "fret custom-effect bind group",
            bind_group_mask: "fret custom-effect mask bind group",
            pass_unmasked: "fret custom-effect pass",
            pass_masked: "fret custom-effect masked pass",
            pass_mask: "fret custom-effect mask pass",
        },
        true,
        common.src,
        common.dst,
        common.src_size,
        common.dst_size,
        common.dst_scissor,
        common.mask_uniform_index,
        common.mask,
        common.load,
        packed.as_ref(),
        packed.len() as u64,
        |r| &r.effect_params.custom_effect_param_buffer,
        |r| r.custom_effect_bind_group_layout_ref(),
        |r| r.custom_effect_mask_bind_group_layout_ref(),
        move |r| r.custom_effect_pipeline_ref(effect),
        move |r| r.custom_effect_masked_pipeline_ref(effect),
        move |r| r.custom_effect_mask_pipeline_ref(effect),
        Some("clip-based custom effect expects full-size destination"),
        Some("mask-based custom effect expects full-size destination"),
    );
}

pub(in super::super) fn record_custom_effect_v2_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CustomEffectV2Pass,
) {
    let common = pass.common;
    let effect = common.effect;

    let Some(entry) = exec
        .renderer
        .material_effect_state
        .custom_effects
        .get(effect)
    else {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    };
    if entry.abi != CustomEffectAbi::V2 {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    exec.renderer
        .ensure_custom_effect_v2_pipelines(exec.device, exec.format, effect);
    if !exec
        .renderer
        .pipelines
        .custom_effect_v2_pipelines
        .contains_key(&effect)
    {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    // Params (64B)
    let packed = pack_effect_params_v1(common.params);
    exec.queue.write_buffer(
        &exec.renderer.effect_params.custom_effect_param_buffer,
        0,
        packed.as_ref(),
    );
    if exec.perf_enabled {
        exec.frame_perf.uniform_bytes = exec
            .frame_perf
            .uniform_bytes
            .saturating_add(packed.len() as u64);
    }

    // Input meta (uv rect)
    let uv = pass.input_uv;
    let mut input_meta = [0u8; 16];
    input_meta[0..4].copy_from_slice(&uv.u0.to_bits().to_le_bytes());
    input_meta[4..8].copy_from_slice(&uv.v0.to_bits().to_le_bytes());
    input_meta[8..12].copy_from_slice(&uv.u1.to_bits().to_le_bytes());
    input_meta[12..16].copy_from_slice(&uv.v1.to_bits().to_le_bytes());
    exec.queue.write_buffer(
        &exec
            .renderer
            .effect_params
            .custom_effect_v2_input_meta_buffer,
        0,
        &input_meta,
    );
    if exec.perf_enabled {
        exec.frame_perf.uniform_bytes = exec.frame_perf.uniform_bytes.saturating_add(16);
    }

    let Some(src_view) = require_color_src_view(
        &mut *exec.frame_targets,
        common.src,
        common.src_size,
        "CustomEffectV2",
    ) else {
        return;
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        &mut *exec.frame_targets,
        &mut exec.renderer.intermediate_state.pool,
        exec.device,
        common.dst,
        common.dst_size,
        exec.format,
        exec.usage,
        "CustomEffectV2",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(exec.target_view);

    let mut input_incompatible = false;
    let input_view = resolve_custom_effect_filterable_user_image_view(
        &exec.renderer,
        exec.device.features(),
        pass.input_image,
        &mut input_incompatible,
    );
    if exec.perf_enabled && input_incompatible {
        exec.frame_perf
            .custom_effect_v2_user_image_incompatible_fallbacks = exec
            .frame_perf
            .custom_effect_v2_user_image_incompatible_fallbacks
            .saturating_add(1);
    }

    let input_sampler = match pass.input_sampling {
        fret_core::scene::ImageSamplingHint::Nearest => {
            &exec.renderer.globals.image_sampler_nearest
        }
        _ => &exec.renderer.globals.viewport_sampler,
    };

    let layout_unmasked = exec.renderer.custom_effect_v2_bind_group_layout_ref();
    let layout_mask = exec.renderer.custom_effect_v2_mask_bind_group_layout_ref();

    let param_buffer = &exec.renderer.effect_params.custom_effect_param_buffer;
    let input_meta_buffer = &exec
        .renderer
        .effect_params
        .custom_effect_v2_input_meta_buffer;

    let uniform_stride = exec.renderer.uniform_stride();

    if let Some(mask) = common.mask {
        let mask_uniform_index = common
            .mask_uniform_index
            .expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let Some(mask_view) = require_mask_view(
            &mut *exec.frame_targets,
            mask.target,
            mask.size,
            "CustomEffectV2",
        ) else {
            return;
        };

        let bind_group = exec.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret custom-effect v2 mask bind group"),
            layout: layout_mask,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(input_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: input_meta_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = exec.renderer.custom_effect_v2_mask_pipeline_ref(effect);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret custom-effect v2 mask pass",
            pipeline,
            dst_view,
            common.load,
            exec.renderer.pick_uniform_bind_group_for_mask_image(
                exec.encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            common.dst_scissor,
            common.dst_size,
            exec.perf_enabled.then_some(&mut *exec.frame_perf),
        );
        return;
    }

    let bind_group = exec.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fret custom-effect v2 bind group"),
        layout: layout_unmasked,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: param_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(input_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(input_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: input_meta_buffer.as_entire_binding(),
            },
        ],
    });

    if let Some(mask_uniform_index) = common.mask_uniform_index {
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let pipeline = exec.renderer.custom_effect_v2_masked_pipeline_ref(effect);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret custom-effect v2 masked pass",
            pipeline,
            dst_view,
            common.load,
            exec.renderer.pick_uniform_bind_group_for_mask_image(
                exec.encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            common.dst_scissor,
            common.dst_size,
            exec.perf_enabled.then_some(&mut *exec.frame_perf),
        );
        return;
    }

    let pipeline = exec.renderer.custom_effect_v2_pipeline_ref(effect);
    run_fullscreen_triangle_pass_uniform_texture(
        &mut *exec.encoder,
        "fret custom-effect v2 pass",
        pipeline,
        dst_view,
        common.load,
        exec.renderer.pick_uniform_bind_group_for_mask_image(None),
        &[0, ctx.render_space_offset_u32],
        &bind_group,
        &[],
        common.dst_scissor,
        common.dst_size,
        exec.perf_enabled.then_some(&mut *exec.frame_perf),
    );
}

pub(in super::super) fn record_custom_effect_v3_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CustomEffectV3Pass,
) {
    let common = pass.common;
    let effect = common.effect;

    let Some(entry) = exec
        .renderer
        .material_effect_state
        .custom_effects
        .get(effect)
    else {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    };
    if entry.abi != CustomEffectAbi::V3 {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    exec.renderer
        .ensure_custom_effect_v3_pipelines(exec.device, exec.format, effect);
    if !exec
        .renderer
        .pipelines
        .custom_effect_v3_pipelines
        .contains_key(&effect)
    {
        let blit = FullscreenBlitPass {
            src: common.src,
            dst: common.dst,
            src_size: common.src_size,
            dst_size: common.dst_size,
            dst_scissor: common.dst_scissor,
            encode_output_srgb: false,
            load: common.load,
        };
        record_fullscreen_blit_pass(exec, &blit);
        return;
    }

    // Params (64B)
    let packed = pack_effect_params_v1(common.params);
    exec.queue.write_buffer(
        &exec.renderer.effect_params.custom_effect_param_buffer,
        0,
        packed.as_ref(),
    );
    if exec.perf_enabled {
        exec.frame_perf.uniform_bytes = exec
            .frame_perf
            .uniform_bytes
            .saturating_add(packed.len() as u64);
    }

    // Meta (48B): pyramid levels + user UV rects.
    let mut meta_bytes = [0u8; 48];
    meta_bytes[0..4].copy_from_slice(&pass.pyramid_levels.to_le_bytes());

    let u0 = pass.user0_uv;
    meta_bytes[16..20].copy_from_slice(&u0.u0.to_bits().to_le_bytes());
    meta_bytes[20..24].copy_from_slice(&u0.v0.to_bits().to_le_bytes());
    meta_bytes[24..28].copy_from_slice(&u0.u1.to_bits().to_le_bytes());
    meta_bytes[28..32].copy_from_slice(&u0.v1.to_bits().to_le_bytes());

    let u1 = pass.user1_uv;
    meta_bytes[32..36].copy_from_slice(&u1.u0.to_bits().to_le_bytes());
    meta_bytes[36..40].copy_from_slice(&u1.v0.to_bits().to_le_bytes());
    meta_bytes[40..44].copy_from_slice(&u1.u1.to_bits().to_le_bytes());
    meta_bytes[44..48].copy_from_slice(&u1.v1.to_bits().to_le_bytes());

    exec.queue.write_buffer(
        &exec.renderer.effect_params.custom_effect_v3_meta_buffer,
        0,
        &meta_bytes,
    );
    if exec.perf_enabled {
        exec.frame_perf.uniform_bytes = exec.frame_perf.uniform_bytes.saturating_add(48);
    }

    let Some(src_view) = require_color_src_view(
        &mut *exec.frame_targets,
        common.src,
        common.src_size,
        "CustomEffectV3",
    ) else {
        return;
    };
    let Some(src_raw_view) = require_color_src_view(
        &mut *exec.frame_targets,
        pass.src_raw,
        common.src_size,
        "CustomEffectV3",
    ) else {
        return;
    };

    let pyramid_override_view = if pass.pyramid_wanted && pass.pyramid_levels >= 2 {
        let reuse = exec.renderer.custom_effect_v3_pyramid.can_reuse(
            pass.src_raw,
            common.src_size,
            exec.format,
            pass.pyramid_levels,
        );
        if exec.perf_enabled {
            if reuse {
                exec.frame_perf.custom_effect_v3_pyramid_cache_hits = exec
                    .frame_perf
                    .custom_effect_v3_pyramid_cache_hits
                    .saturating_add(1);
            } else {
                exec.frame_perf.custom_effect_v3_pyramid_cache_misses = exec
                    .frame_perf
                    .custom_effect_v3_pyramid_cache_misses
                    .saturating_add(1);
            }
        }

        let (mip_views, mip_sizes, full_view) = {
            let scratch = exec.renderer.custom_effect_v3_pyramid.ensure_scratch(
                exec.device,
                common.src_size,
                exec.format,
                pass.pyramid_levels,
            );

            let mip_views = scratch.mip_views.iter().cloned().collect::<Vec<_>>();
            let mip_sizes = (0..scratch.levels)
                .map(|level| scratch.mip_size(level))
                .collect::<Vec<_>>();
            (mip_views, mip_sizes, scratch.full_view.clone())
        };

        if !reuse {
            exec.renderer.ensure_blit_pipeline(exec.device, exec.format);
            exec.renderer
                .ensure_mip_downsample_box_pipeline(exec.device, exec.format);

            fn downsample_scissor_2x(
                scissor: ScissorRect,
                dst_size: (u32, u32),
            ) -> Option<ScissorRect> {
                if scissor.w == 0 || scissor.h == 0 {
                    return None;
                }
                let x0 = scissor.x / 2;
                let y0 = scissor.y / 2;
                let x1 = scissor.x.saturating_add(scissor.w).saturating_add(1) / 2;
                let y1 = scissor.y.saturating_add(scissor.h).saturating_add(1) / 2;
                let x1 = x1.min(dst_size.0);
                let y1 = y1.min(dst_size.1);
                if x1 <= x0 || y1 <= y0 {
                    return None;
                }
                Some(ScissorRect {
                    x: x0,
                    y: y0,
                    w: x1 - x0,
                    h: y1 - y0,
                })
            }

            let mut pyramid_scissor = pass.pyramid_build_scissor.map(|s| s.0);

            // Level 0: blit from src_raw into the pyramid scratch.
            let blit_layout = exec.renderer.blit_bind_group_layout_ref();
            let blit_bind_group = create_texture_bind_group(
                exec.device,
                "fret custom-effect v3 pyramid blit bind group",
                blit_layout,
                &src_raw_view,
            );
            run_fullscreen_triangle_pass(
                &mut *exec.encoder,
                "fret custom-effect v3 pyramid blit",
                exec.renderer.blit_pipeline_ref(),
                &mip_views[0],
                mip_sizes[0],
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                &blit_bind_group,
                &[],
                pyramid_scissor.map(LocalScissorRect),
                exec.perf_enabled.then_some(&mut *exec.frame_perf),
            );

            // Downsample chain: mip(i-1) -> mip(i) via a 2x2 box filter.
            let downsample_layout = exec.renderer.mip_downsample_box_bind_group_layout_ref();

            for level in 1..(mip_views.len() as u32) {
                let src_level = (level - 1) as usize;
                let bind_group = exec.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fret mip downsample box bind group"),
                    layout: downsample_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&mip_views[src_level]),
                    }],
                });
                pyramid_scissor = pyramid_scissor
                    .and_then(|s| downsample_scissor_2x(s, mip_sizes[level as usize]));
                run_fullscreen_triangle_pass(
                    &mut *exec.encoder,
                    "fret mip downsample box pass",
                    exec.renderer.mip_downsample_box_pipeline_ref(),
                    &mip_views[level as usize],
                    mip_sizes[level as usize],
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    &bind_group,
                    &[],
                    pyramid_scissor.map(LocalScissorRect),
                    exec.perf_enabled.then_some(&mut *exec.frame_perf),
                );
            }

            exec.renderer.custom_effect_v3_pyramid.set_cache(
                pass.src_raw,
                common.src_size,
                exec.format,
                pass.pyramid_levels,
            );
        }

        Some(full_view)
    } else {
        None
    };

    let src_pyramid_view = if let Some(view) = pyramid_override_view {
        view
    } else {
        let Some(view) = require_color_src_view(
            &mut *exec.frame_targets,
            pass.src_pyramid,
            common.src_size,
            "CustomEffectV3",
        ) else {
            return;
        };
        view
    };

    let dst_view_owned = ensure_color_dst_view_owned(
        &mut *exec.frame_targets,
        &mut exec.renderer.intermediate_state.pool,
        exec.device,
        common.dst,
        common.dst_size,
        exec.format,
        exec.usage,
        "CustomEffectV3",
    );
    let dst_view = dst_view_owned.as_ref().unwrap_or(exec.target_view);

    let mut user0_incompatible = false;
    let user0_view = resolve_custom_effect_filterable_user_image_view(
        &exec.renderer,
        exec.device.features(),
        pass.user0_image,
        &mut user0_incompatible,
    );
    if exec.perf_enabled && user0_incompatible {
        exec.frame_perf
            .custom_effect_v3_user0_image_incompatible_fallbacks = exec
            .frame_perf
            .custom_effect_v3_user0_image_incompatible_fallbacks
            .saturating_add(1);
    }

    let mut user1_incompatible = false;
    let user1_view = resolve_custom_effect_filterable_user_image_view(
        &exec.renderer,
        exec.device.features(),
        pass.user1_image,
        &mut user1_incompatible,
    );
    if exec.perf_enabled && user1_incompatible {
        exec.frame_perf
            .custom_effect_v3_user1_image_incompatible_fallbacks = exec
            .frame_perf
            .custom_effect_v3_user1_image_incompatible_fallbacks
            .saturating_add(1);
    }

    let user0_sampler = match pass.user0_sampling {
        fret_core::scene::ImageSamplingHint::Nearest => {
            &exec.renderer.globals.image_sampler_nearest
        }
        _ => &exec.renderer.globals.viewport_sampler,
    };
    let user1_sampler = match pass.user1_sampling {
        fret_core::scene::ImageSamplingHint::Nearest => {
            &exec.renderer.globals.image_sampler_nearest
        }
        _ => &exec.renderer.globals.viewport_sampler,
    };

    let layout_unmasked = exec.renderer.custom_effect_v3_bind_group_layout_ref();
    let layout_mask = exec.renderer.custom_effect_v3_mask_bind_group_layout_ref();

    let param_buffer = &exec.renderer.effect_params.custom_effect_param_buffer;
    let meta_buffer = &exec.renderer.effect_params.custom_effect_v3_meta_buffer;

    let uniform_stride = exec.renderer.uniform_stride();

    if let Some(mask) = common.mask {
        let mask_uniform_index = common
            .mask_uniform_index
            .expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let Some(mask_view) = require_mask_view(
            &mut *exec.frame_targets,
            mask.target,
            mask.size,
            "CustomEffectV3",
        ) else {
            return;
        };

        let bind_group = exec.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret custom-effect v3 mask bind group"),
            layout: layout_mask,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&src_raw_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&src_pyramid_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: meta_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(user0_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(user0_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(user1_view),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(user1_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
            ],
        });

        let pipeline = exec.renderer.custom_effect_v3_mask_pipeline_ref(effect);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret custom-effect v3 mask pass",
            pipeline,
            dst_view,
            common.load,
            exec.renderer.pick_uniform_bind_group_for_mask_image(
                exec.encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            common.dst_scissor,
            common.dst_size,
            exec.perf_enabled.then_some(&mut *exec.frame_perf),
        );
        return;
    }

    let bind_group = exec.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fret custom-effect v3 bind group"),
        layout: layout_unmasked,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: param_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&src_raw_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&src_pyramid_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: meta_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(user0_view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Sampler(user0_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(user1_view),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::Sampler(user1_sampler),
            },
        ],
    });

    if let Some(mask_uniform_index) = common.mask_uniform_index {
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let pipeline = exec.renderer.custom_effect_v3_masked_pipeline_ref(effect);
        run_fullscreen_triangle_pass_uniform_texture(
            &mut *exec.encoder,
            "fret custom-effect v3 masked pass",
            pipeline,
            dst_view,
            common.load,
            exec.renderer.pick_uniform_bind_group_for_mask_image(
                exec.encoding
                    .uniform_mask_images
                    .get(mask_uniform_index as usize)
                    .copied()
                    .flatten(),
            ),
            &[uniform_offset, ctx.render_space_offset_u32],
            &bind_group,
            &[],
            common.dst_scissor,
            common.dst_size,
            exec.perf_enabled.then_some(&mut *exec.frame_perf),
        );
        return;
    }

    let pipeline = exec.renderer.custom_effect_v3_pipeline_ref(effect);
    run_fullscreen_triangle_pass_uniform_texture(
        &mut *exec.encoder,
        "fret custom-effect v3 pass",
        pipeline,
        dst_view,
        common.load,
        exec.renderer.pick_uniform_bind_group_for_mask_image(None),
        &[0, ctx.render_space_offset_u32],
        &bind_group,
        &[],
        common.dst_scissor,
        common.dst_size,
        exec.perf_enabled.then_some(&mut *exec.frame_perf),
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
        &mut renderer.intermediate_state.pool,
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
            let uniform_offset = (u64::from(mask_uniform_index) * renderer.uniform_stride()) as u32;
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
                renderer.base_uniform_bind_group(),
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
        renderer.path_composite_vertices_ref(),
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
        &mut renderer.intermediate_state.pool,
        device,
        pass.dst,
        pass.dst_size,
        usage,
        "ClipMask",
    ) else {
        return;
    };

    let pipeline = renderer.clip_mask_pipeline_ref();
    let uniform_offset = (u64::from(pass.uniform_index) * renderer.uniform_stride()) as u32;

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
