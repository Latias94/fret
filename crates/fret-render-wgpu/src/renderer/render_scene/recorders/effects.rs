use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::blit::record_fullscreen_blit_pass;
use super::effects_shared::{
    FullscreenEffectLabels, pack_effect_params_v1, record_fullscreen_param_effect_pass,
    record_fullscreen_texture_effect_pass,
};

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
