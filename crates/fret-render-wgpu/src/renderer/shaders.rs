const CLIP_SDF_CORE_WGSL: &str = include_str!("pipelines/wgsl/clip_sdf_core.wgsl");

const QUAD_SHADER_PART_A: &str = include_str!("pipelines/wgsl/quad_part_a.wgsl");

const QUAD_SHADER_PART_B: &str = include_str!("pipelines/wgsl/quad_part_b.wgsl");

pub(super) fn quad_shader_source() -> String {
    format!("{QUAD_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{QUAD_SHADER_PART_B}")
}

pub(super) const VIEWPORT_SHADER: &str = include_str!("pipelines/wgsl/viewport.wgsl");

// Large effect shaders live as external WGSL files for reviewable diffs and lower merge conflict risk.
pub(super) const BLIT_SHADER: &str = include_str!("pipelines/wgsl/blit.wgsl");

pub(super) const BLIT_SRGB_ENCODE_SHADER: &str =
    include_str!("pipelines/wgsl/blit_srgb_encode.wgsl");

pub(super) const MIP_DOWNSAMPLE_BOX_2X2_SHADER: &str =
    include_str!("pipelines/wgsl/mip_downsample_box_2x2.wgsl");

pub(super) const DROP_SHADOW_SHADER: &str = include_str!("pipelines/wgsl/drop_shadow.wgsl");

const DROP_SHADOW_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/drop_shadow_masked_part_a.wgsl");

const DROP_SHADOW_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/drop_shadow_masked_part_b.wgsl");

pub(super) fn drop_shadow_masked_shader_source() -> String {
    format!(
        "{DROP_SHADOW_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{DROP_SHADOW_MASKED_SHADER_PART_B}"
    )
}

pub(super) const DROP_SHADOW_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/drop_shadow_mask.wgsl");

pub(super) const DOWNSAMPLE_NEAREST_SHADER: &str =
    include_str!("pipelines/wgsl/downsample_nearest.wgsl");

pub(super) const UPSCALE_NEAREST_SHADER: &str = include_str!("pipelines/wgsl/upscale_nearest.wgsl");

const UPSCALE_NEAREST_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/upscale_nearest_masked_part_a.wgsl");

const UPSCALE_NEAREST_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/upscale_nearest_masked_part_b.wgsl");

pub(super) fn upscale_nearest_masked_shader_source() -> String {
    format!(
        "{UPSCALE_NEAREST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{UPSCALE_NEAREST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const UPSCALE_NEAREST_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/upscale_nearest_mask.wgsl");

const CLIP_MASK_SHADER_PART_A: &str = include_str!("pipelines/wgsl/clip_mask_part_a.wgsl");

const CLIP_MASK_SHADER_PART_B: &str = include_str!("pipelines/wgsl/clip_mask_part_b.wgsl");

pub(super) fn clip_mask_shader_source() -> String {
    format!("{CLIP_MASK_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{CLIP_MASK_SHADER_PART_B}")
}

pub(super) const BACKDROP_WARP_SHADER: &str = include_str!("pipelines/wgsl/backdrop_warp.wgsl");

pub(super) const BACKDROP_WARP_IMAGE_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image.wgsl");

const BACKDROP_WARP_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/backdrop_warp_masked_part_a.wgsl");

const BACKDROP_WARP_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/backdrop_warp_masked_part_b.wgsl");

pub(super) fn backdrop_warp_masked_shader_source() -> String {
    format!(
        "{BACKDROP_WARP_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BACKDROP_WARP_MASKED_SHADER_PART_B}"
    )
}

const BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_masked_part_a.wgsl");

const BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_masked_part_b.wgsl");

pub(super) fn backdrop_warp_image_masked_shader_source() -> String {
    format!(
        "{BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BACKDROP_WARP_IMAGE_MASKED_SHADER_PART_B}"
    )
}

pub(super) const BACKDROP_WARP_IMAGE_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_image_mask.wgsl");

pub(super) const BACKDROP_WARP_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/backdrop_warp_mask.wgsl");

pub(super) const COLOR_ADJUST_SHADER: &str = include_str!("pipelines/wgsl/color_adjust.wgsl");

const COLOR_ADJUST_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/color_adjust_masked_part_a.wgsl");

const COLOR_ADJUST_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/color_adjust_masked_part_b.wgsl");

pub(super) fn color_adjust_masked_shader_source() -> String {
    format!(
        "{COLOR_ADJUST_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_ADJUST_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_ADJUST_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/color_adjust_mask.wgsl");

pub(super) const COLOR_MATRIX_SHADER: &str = include_str!("pipelines/wgsl/color_matrix.wgsl");

const COLOR_MATRIX_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/color_matrix_masked_part_a.wgsl");

const COLOR_MATRIX_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/color_matrix_masked_part_b.wgsl");

pub(super) fn color_matrix_masked_shader_source() -> String {
    format!(
        "{COLOR_MATRIX_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{COLOR_MATRIX_MASKED_SHADER_PART_B}"
    )
}

pub(super) const COLOR_MATRIX_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/color_matrix_mask.wgsl");

pub(super) const ALPHA_THRESHOLD_SHADER: &str = include_str!("pipelines/wgsl/alpha_threshold.wgsl");

const ALPHA_THRESHOLD_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/alpha_threshold_masked_part_a.wgsl");

const ALPHA_THRESHOLD_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/alpha_threshold_masked_part_b.wgsl");

pub(super) fn alpha_threshold_masked_shader_source() -> String {
    format!(
        "{ALPHA_THRESHOLD_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{ALPHA_THRESHOLD_MASKED_SHADER_PART_B}"
    )
}

pub(super) const ALPHA_THRESHOLD_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/alpha_threshold_mask.wgsl");

pub(super) const DITHER_SHADER: &str = include_str!("pipelines/wgsl/dither.wgsl");

const DITHER_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/dither_masked_part_a.wgsl");

const DITHER_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/dither_masked_part_b.wgsl");

pub(super) fn dither_masked_shader_source() -> String {
    format!("{DITHER_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{DITHER_MASKED_SHADER_PART_B}")
}

pub(super) const DITHER_MASK_SHADER: &str = include_str!("pipelines/wgsl/dither_mask.wgsl");

const CUSTOM_EFFECT_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_masked_part_a.wgsl");
const CUSTOM_EFFECT_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_masked_part_b.wgsl");

const CUSTOM_EFFECT_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_mask_part_a.wgsl");
const CUSTOM_EFFECT_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_mask_part_b.wgsl");

pub(super) fn custom_effect_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_mask_shader_source(user_source: &str) -> String {
    format!("{CUSTOM_EFFECT_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_MASK_SHADER_PART_B}")
}

const CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_V2_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_masked_part_a.wgsl");
const CUSTOM_EFFECT_V2_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_masked_part_b.wgsl");

const CUSTOM_EFFECT_V2_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_mask_part_a.wgsl");
const CUSTOM_EFFECT_V2_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v2_mask_part_b.wgsl");

pub(super) fn custom_effect_v2_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V2_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v2_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_V2_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v2_mask_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V2_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V2_MASK_SHADER_PART_B}"
    )
}

const CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_unmasked_part_a.wgsl");
const CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_unmasked_part_b.wgsl");

const CUSTOM_EFFECT_V3_MASKED_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_masked_part_a.wgsl");
const CUSTOM_EFFECT_V3_MASKED_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_masked_part_b.wgsl");

const CUSTOM_EFFECT_V3_MASK_SHADER_PART_A: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_mask_part_a.wgsl");
const CUSTOM_EFFECT_V3_MASK_SHADER_PART_B: &str =
    include_str!("pipelines/wgsl/custom_effect_v3_mask_part_b.wgsl");

pub(super) fn custom_effect_v3_unmasked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V3_UNMASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v3_masked_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{user_source}\n{CUSTOM_EFFECT_V3_MASKED_SHADER_PART_B}"
    )
}

pub(super) fn custom_effect_v3_mask_shader_source(user_source: &str) -> String {
    format!(
        "{CUSTOM_EFFECT_V3_MASK_SHADER_PART_A}{user_source}\n{CUSTOM_EFFECT_V3_MASK_SHADER_PART_B}"
    )
}

pub(super) const BLUR_H_SHADER: &str = include_str!("pipelines/wgsl/blur_h.wgsl");

pub(super) const BLUR_V_SHADER: &str = include_str!("pipelines/wgsl/blur_v.wgsl");

const BLUR_H_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/blur_h_masked_part_a.wgsl");

const BLUR_H_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/blur_h_masked_part_b.wgsl");

pub(super) fn blur_h_masked_shader_source() -> String {
    format!("{BLUR_H_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_H_MASKED_SHADER_PART_B}")
}

const BLUR_V_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/blur_v_masked_part_a.wgsl");

const BLUR_V_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/blur_v_masked_part_b.wgsl");

pub(super) fn blur_v_masked_shader_source() -> String {
    format!("{BLUR_V_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{BLUR_V_MASKED_SHADER_PART_B}")
}

pub(super) const BLUR_H_MASK_SHADER: &str = include_str!("pipelines/wgsl/blur_h_mask.wgsl");

pub(super) const BLUR_V_MASK_SHADER: &str = include_str!("pipelines/wgsl/blur_v_mask.wgsl");

pub(super) const COMPOSITE_PREMUL_SHADER: &str =
    include_str!("pipelines/wgsl/composite_premul.wgsl");

pub(super) const COMPOSITE_PREMUL_MASK_SHADER: &str =
    include_str!("pipelines/wgsl/composite_premul_mask.wgsl");

pub(super) const PATH_CLIP_MASK_SHADER: &str = include_str!("pipelines/wgsl/path_clip_mask.wgsl");

pub(super) const PATH_SHADER: &str = include_str!("pipelines/wgsl/path.wgsl");

pub(super) const TEXT_SHADER: &str = include_str!("pipelines/wgsl/text.wgsl");

pub(super) const TEXT_COLOR_SHADER: &str = include_str!("pipelines/wgsl/text_color.wgsl");

pub(super) const TEXT_SUBPIXEL_SHADER: &str = include_str!("pipelines/wgsl/text_subpixel.wgsl");

pub(super) const MASK_SHADER: &str = include_str!("pipelines/wgsl/mask.wgsl");

pub(super) const NOISE_SHADER: &str = include_str!("pipelines/wgsl/noise.wgsl");

const NOISE_MASKED_SHADER_PART_A: &str = include_str!("pipelines/wgsl/noise_masked_part_a.wgsl");

const NOISE_MASKED_SHADER_PART_B: &str = include_str!("pipelines/wgsl/noise_masked_part_b.wgsl");

pub(super) fn noise_masked_shader_source() -> String {
    format!("{NOISE_MASKED_SHADER_PART_A}{CLIP_SDF_CORE_WGSL}{NOISE_MASKED_SHADER_PART_B}")
}

pub(super) const NOISE_MASK_SHADER: &str = include_str!("pipelines/wgsl/noise_mask.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_shader_wgsl_validates_under_naga() {
        let module = naga::front::wgsl::parse_str(PATH_SHADER).expect("PATH_SHADER must parse");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .expect("PATH_SHADER must validate under naga");
    }
}
