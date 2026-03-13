use super::super::super::*;
use super::super::executor::{RecordPassCtx, RenderSceneExecutor};
use super::blit::record_fullscreen_blit_pass;
use super::effects_shared::{
    FullscreenEffectLabels, pack_effect_params_v1, record_fullscreen_param_effect_pass,
};

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
