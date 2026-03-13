use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::blit::record_fullscreen_blit_pass;
use super::effects_bindings::{
    CustomEffectV2BindGroupResources, create_custom_effect_v2_bind_group,
};
use super::effects_shared::pack_effect_params_v1;

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

    let features = renderer.adapter.get_texture_format_features(format);
    let ok_usage = features
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

fn record_custom_effect_v2_fallback_blit(
    exec: &mut RenderSceneExecutor<'_>,
    common: CustomEffectPassCommon,
) {
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
}

fn upload_custom_effect_v2_params_and_input_meta(
    exec: &mut RenderSceneExecutor<'_>,
    pass: &CustomEffectV2Pass,
) {
    let packed = pack_effect_params_v1(pass.common.params);
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
        record_custom_effect_v2_fallback_blit(exec, common);
        return;
    };
    if entry.abi != CustomEffectAbi::V2 {
        record_custom_effect_v2_fallback_blit(exec, common);
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
        record_custom_effect_v2_fallback_blit(exec, common);
        return;
    }

    upload_custom_effect_v2_params_and_input_meta(exec, pass);

    let Some(src_view) = exec.require_color_src_view(common.src, common.src_size, "CustomEffectV2")
    else {
        return;
    };

    let dst_view_owned =
        exec.ensure_color_dst_view_owned(common.dst, common.dst_size, "CustomEffectV2");
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
    let binding_resources = CustomEffectV2BindGroupResources {
        src_view: &src_view,
        param_buffer,
        input_view,
        input_sampler,
        input_meta_buffer,
    };

    let uniform_stride = exec.renderer.uniform_stride();

    if let Some(mask) = common.mask {
        let mask_uniform_index = common
            .mask_uniform_index
            .expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, "CustomEffectV2")
        else {
            return;
        };

        let bind_group = create_custom_effect_v2_bind_group(
            exec.device,
            "fret custom-effect v2 mask bind group",
            layout_mask,
            binding_resources,
            Some(&mask_view),
        );

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

    let bind_group = create_custom_effect_v2_bind_group(
        exec.device,
        "fret custom-effect v2 bind group",
        layout_unmasked,
        binding_resources,
        None,
    );

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
