#[test]
fn liquid_glass_custom_effect_v3_lens_wgsl_parses_in_unmasked_pipeline() {
    let part_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_unmasked_part_a.wgsl"
    );
    let part_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_unmasked_part_b.wgsl"
    );
    let user = fret_examples::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;
    let combined = format!("{part_a}\n{user}\n{part_b}");
    naga::front::wgsl::parse_str(&combined)
        .expect("CUSTOM_EFFECT_V3_LENS_WGSL should parse when wrapped in the unmasked pipeline");
}

#[test]
fn liquid_glass_custom_effect_v3_lens_wgsl_parses_in_masked_pipeline() {
    let part_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_masked_part_a.wgsl"
    );
    let clip_sdf_core = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/clip_sdf_core.wgsl"
    );
    let part_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_masked_part_b.wgsl"
    );
    let user = fret_examples::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;
    let combined = format!("{part_a}{clip_sdf_core}{user}\n{part_b}");
    naga::front::wgsl::parse_str(&combined)
        .expect("CUSTOM_EFFECT_V3_LENS_WGSL should parse when wrapped in the masked pipeline");
}

#[test]
fn liquid_glass_custom_effect_v3_lens_wgsl_parses_in_mask_pipeline() {
    let part_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_mask_part_a.wgsl"
    );
    let part_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_mask_part_b.wgsl"
    );
    let user = fret_examples::custom_effect_v3_wgsl::CUSTOM_EFFECT_V3_LENS_WGSL;
    let combined = format!("{part_a}\n{user}\n{part_b}");
    naga::front::wgsl::parse_str(&combined)
        .expect("CUSTOM_EFFECT_V3_LENS_WGSL should parse when wrapped in the mask pipeline");
}

fn assert_custom_effect_wgsl_parses(
    abi_label: &str,
    variant: &str,
    part_a: &str,
    user: &str,
    part_b: &str,
) {
    let combined = format!("{part_a}\n{user}\n{part_b}");
    naga::front::wgsl::parse_str(&combined).unwrap_or_else(|err| {
        panic!(
            "expected custom effect WGSL to parse (abi={abi_label}, variant={variant}): {err:?}"
        );
    });
}

const IDENTITY_WGSL: &str = r#"
fn fret_custom_effect(
  src: vec4<f32>,
  _uv: vec2<f32>,
  _pos_px: vec2<f32>,
  _params: EffectParamsV1
) -> vec4<f32> {
  return src;
}
"#;

#[test]
fn custom_effect_v1_identity_wgsl_parses_in_unmasked_pipeline() {
    let unmasked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_unmasked_part_a.wgsl"
    );
    let unmasked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_unmasked_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v1", "unmasked", unmasked_a, IDENTITY_WGSL, unmasked_b);
}

#[test]
fn custom_effect_v1_identity_wgsl_parses_in_masked_pipeline() {
    let masked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_masked_part_a.wgsl"
    );
    let clip_sdf_core = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/clip_sdf_core.wgsl"
    );
    let masked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_masked_part_b.wgsl"
    );
    let combined = format!("{masked_a}{clip_sdf_core}{IDENTITY_WGSL}\n{masked_b}");
    naga::front::wgsl::parse_str(&combined).unwrap_or_else(|err| {
        panic!("expected custom effect WGSL to parse (abi=v1, variant=masked): {err:?}");
    });
}

#[test]
fn custom_effect_v1_identity_wgsl_parses_in_mask_pipeline() {
    let mask_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_mask_part_a.wgsl"
    );
    let mask_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_mask_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v1", "mask", mask_a, IDENTITY_WGSL, mask_b);
}

#[test]
fn custom_effect_v2_identity_wgsl_parses_in_unmasked_pipeline() {
    let unmasked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_unmasked_part_a.wgsl"
    );
    let unmasked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_unmasked_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v2", "unmasked", unmasked_a, IDENTITY_WGSL, unmasked_b);
}

#[test]
fn custom_effect_v2_identity_wgsl_parses_in_masked_pipeline() {
    let masked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_masked_part_a.wgsl"
    );
    let clip_sdf_core = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/clip_sdf_core.wgsl"
    );
    let masked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_masked_part_b.wgsl"
    );
    let combined = format!("{masked_a}{clip_sdf_core}{IDENTITY_WGSL}\n{masked_b}");
    naga::front::wgsl::parse_str(&combined).unwrap_or_else(|err| {
        panic!("expected custom effect WGSL to parse (abi=v2, variant=masked): {err:?}");
    });
}

#[test]
fn custom_effect_v2_identity_wgsl_parses_in_mask_pipeline() {
    let mask_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_mask_part_a.wgsl"
    );
    let mask_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_mask_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v2", "mask", mask_a, IDENTITY_WGSL, mask_b);
}

#[test]
fn custom_effect_v3_identity_wgsl_parses_in_unmasked_pipeline() {
    let unmasked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_unmasked_part_a.wgsl"
    );
    let unmasked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_unmasked_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v3", "unmasked", unmasked_a, IDENTITY_WGSL, unmasked_b);
}

#[test]
fn custom_effect_v3_identity_wgsl_parses_in_masked_pipeline() {
    let masked_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_masked_part_a.wgsl"
    );
    let clip_sdf_core = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/clip_sdf_core.wgsl"
    );
    let masked_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_masked_part_b.wgsl"
    );
    let combined = format!("{masked_a}{clip_sdf_core}{IDENTITY_WGSL}\n{masked_b}");
    naga::front::wgsl::parse_str(&combined).unwrap_or_else(|err| {
        panic!("expected custom effect WGSL to parse (abi=v3, variant=masked): {err:?}");
    });
}

#[test]
fn custom_effect_v3_identity_wgsl_parses_in_mask_pipeline() {
    let mask_a = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_mask_part_a.wgsl"
    );
    let mask_b = include_str!(
        "../../../crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v3_mask_part_b.wgsl"
    );
    assert_custom_effect_wgsl_parses("v3", "mask", mask_a, IDENTITY_WGSL, mask_b);
}
