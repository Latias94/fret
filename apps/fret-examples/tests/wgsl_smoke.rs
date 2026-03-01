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
