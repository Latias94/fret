use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::super::executor_recorders::{
    CustomEffectV3PreparedSourceViews, CustomEffectV3PreparedUserImages,
};
use super::blit::record_fullscreen_blit_pass;
use super::effects_bindings::{
    CustomEffectV3BindGroupMode, CustomEffectV3BindGroupResources,
    create_custom_effect_v3_pipeline_and_bind_group,
};
use super::effects_shared::pack_effect_params_v1;

fn record_custom_effect_v3_fallback_blit(
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
        record_custom_effect_v3_fallback_blit(exec, common);
        return;
    };
    if entry.abi != CustomEffectAbi::V3 {
        record_custom_effect_v3_fallback_blit(exec, common);
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
        record_custom_effect_v3_fallback_blit(exec, common);
        return;
    }

    upload_custom_effect_v3_params_and_meta(exec, pass);

    let Some(source_views) = exec.prepare_custom_effect_v3_source_views(pass) else {
        return;
    };
    let prepared_user_images = exec.prepare_custom_effect_v3_user_images(pass);
    record_custom_effect_v3_prepared_pass(exec, ctx, pass, &source_views, &prepared_user_images);
}

pub(super) fn upload_custom_effect_v3_params_and_meta(
    exec: &mut RenderSceneExecutor<'_>,
    pass: &CustomEffectV3Pass,
) {
    let common = pass.common;

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
}

pub(super) fn record_custom_effect_v3_prepared_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    pass: &CustomEffectV3Pass,
    source_views: &CustomEffectV3PreparedSourceViews,
    prepared_user_images: &CustomEffectV3PreparedUserImages,
) {
    let common = pass.common;
    let effect = common.effect;

    let dst_view_owned =
        exec.ensure_color_dst_view_owned(common.dst, common.dst_size, "CustomEffectV3");
    let dst_view = dst_view_owned.as_ref().unwrap_or(exec.target_view);

    let binding_resources = CustomEffectV3BindGroupResources {
        src_view: &source_views.src_view,
        src_raw_view: &source_views.src_raw_view,
        src_pyramid_view: &source_views.src_pyramid_view,
        param_buffer: &exec.renderer.effect_params.custom_effect_param_buffer,
        meta_buffer: &exec.renderer.effect_params.custom_effect_v3_meta_buffer,
        user0_view: &prepared_user_images.user0.view,
        user0_sampler: &prepared_user_images.user0.sampler,
        user1_view: &prepared_user_images.user1.view,
        user1_sampler: &prepared_user_images.user1.sampler,
    };

    let uniform_stride = exec.renderer.uniform_stride();

    if let Some(mask) = common.mask {
        let mask_uniform_index = common
            .mask_uniform_index
            .expect("mask pass needs uniform index");
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let Some(mask_view) = exec.require_mask_view(mask.target, mask.size, "CustomEffectV3")
        else {
            return;
        };

        let (pipeline, bind_group) = create_custom_effect_v3_pipeline_and_bind_group(
            exec.device,
            &*exec.renderer,
            effect,
            binding_resources,
            CustomEffectV3BindGroupMode::TextureMask(&mask_view),
        );
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

    if let Some(mask_uniform_index) = common.mask_uniform_index {
        let uniform_offset = (u64::from(mask_uniform_index) * uniform_stride) as u32;
        let (pipeline, bind_group) = create_custom_effect_v3_pipeline_and_bind_group(
            exec.device,
            &*exec.renderer,
            effect,
            binding_resources,
            CustomEffectV3BindGroupMode::UniformMask,
        );
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

    let (pipeline, bind_group) = create_custom_effect_v3_pipeline_and_bind_group(
        exec.device,
        &*exec.renderer,
        effect,
        binding_resources,
        CustomEffectV3BindGroupMode::Unmasked,
    );
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
