use super::shaders::{
    ALPHA_THRESHOLD_MASK_SHADER, ALPHA_THRESHOLD_SHADER, BACKDROP_WARP_IMAGE_MASK_SHADER,
    BACKDROP_WARP_IMAGE_SHADER, BACKDROP_WARP_MASK_SHADER, BACKDROP_WARP_SHADER, BLIT_SHADER,
    BLIT_SRGB_ENCODE_SHADER, BLUR_H_MASK_SHADER, BLUR_H_SHADER, BLUR_V_MASK_SHADER, BLUR_V_SHADER,
    COLOR_ADJUST_MASK_SHADER, COLOR_ADJUST_SHADER, COLOR_MATRIX_MASK_SHADER, COLOR_MATRIX_SHADER,
    COMPOSITE_PREMUL_MASK_SHADER, COMPOSITE_PREMUL_SHADER, DOWNSAMPLE_NEAREST_SHADER,
    DROP_SHADOW_MASK_SHADER, DROP_SHADOW_SHADER, MASK_SHADER, MIP_DOWNSAMPLE_BOX_2X2_SHADER,
    PATH_CLIP_MASK_SHADER, PATH_SHADER, TEXT_COLOR_SHADER, TEXT_SHADER, TEXT_SUBPIXEL_SHADER,
    UPSCALE_NEAREST_MASK_SHADER, UPSCALE_NEAREST_SHADER, VIEWPORT_SHADER,
    alpha_threshold_masked_shader_source, backdrop_warp_image_masked_shader_source,
    backdrop_warp_masked_shader_source, blur_h_masked_shader_source, blur_v_masked_shader_source,
    clip_mask_shader_source, color_adjust_masked_shader_source, color_matrix_masked_shader_source,
    custom_effect_mask_shader_source, custom_effect_masked_shader_source,
    custom_effect_unmasked_shader_source, custom_effect_v2_mask_shader_source,
    custom_effect_v2_masked_shader_source, custom_effect_v2_unmasked_shader_source,
    custom_effect_v3_mask_shader_source, custom_effect_v3_masked_shader_source,
    custom_effect_v3_unmasked_shader_source, drop_shadow_masked_shader_source, quad_shader_source,
    upscale_nearest_masked_shader_source,
};
use super::{clamp_corner_radii_for_rect, svg_draw_rect_px};
use fret_core::PathService as _;
use fret_core::geometry::{Corners, Point, Px, Transform2D};
use fret_core::{
    Color, DrawOrder, FillStyle, PathCommand, PathConstraints, PathStyle, Rect, Scene,
    SceneMeshVertex, SceneOp, Size, TextConstraints, TextStyle, UvPoint, ViewportFit,
};

fn assert_approx_eq(a: f32, b: f32) {
    assert!(
        (a - b).abs() <= 1.0e-6,
        "expected {a} ~= {b} (diff={})",
        (a - b).abs()
    );
}

fn assert_vertex(v: &super::types::ViewportVertex, pos: (f32, f32), uv: (f32, f32)) {
    assert_approx_eq(v.pos_px[0], pos.0);
    assert_approx_eq(v.pos_px[1], pos.1);
    assert_approx_eq(v.uv[0], uv.0);
    assert_approx_eq(v.uv[1], uv.1);
}

fn assert_vertex_color(v: &super::types::ViewportVertex, color: [f32; 4]) {
    for (actual, expected) in v.color.iter().copied().zip(color) {
        assert_approx_eq(actual, expected);
    }
}

fn assert_vertex_opacity_and_premul(v: &super::types::ViewportVertex, opacity: f32, premul: f32) {
    assert_approx_eq(v.opacity, opacity);
    assert_approx_eq(v.premul, premul);
}

const CUSTOM_EFFECT_IDENTITY_WGSL: &str = r#"
fn fret_custom_effect(tex: vec4<f32>, uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  // Keep this shader intentionally simple: it exists to validate that the custom effect ABI
  // can be stitched into a complete module and compiled under WebGPU/Tint constraints.
  return tex;
}
"#;

#[test]
fn shaders_parse_as_wgsl() {
    let quad_src = quad_shader_source();
    let clip_mask_src = clip_mask_shader_source();
    let upscale_masked_src = upscale_nearest_masked_shader_source();
    let backdrop_warp_masked_src = backdrop_warp_masked_shader_source();
    let backdrop_warp_image_masked_src = backdrop_warp_image_masked_shader_source();
    let color_adjust_masked_src = color_adjust_masked_shader_source();
    let color_matrix_masked_src = color_matrix_masked_shader_source();
    let alpha_threshold_masked_src = alpha_threshold_masked_shader_source();
    let drop_shadow_masked_src = drop_shadow_masked_shader_source();
    let blur_h_masked_src = blur_h_masked_shader_source();
    let blur_v_masked_src = blur_v_masked_shader_source();
    let custom_effect_unmasked_src =
        custom_effect_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_masked_src = custom_effect_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_mask_src = custom_effect_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_unmasked_src =
        custom_effect_v2_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_masked_src =
        custom_effect_v2_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_mask_src =
        custom_effect_v2_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_unmasked_src =
        custom_effect_v3_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_masked_src =
        custom_effect_v3_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_mask_src =
        custom_effect_v3_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", quad_src.as_str()),
        ("blit", BLIT_SHADER),
        ("blit_srgb_encode", BLIT_SRGB_ENCODE_SHADER),
        ("mip_downsample_box_2x2", MIP_DOWNSAMPLE_BOX_2X2_SHADER),
        ("drop_shadow", DROP_SHADOW_SHADER),
        ("drop_shadow_masked", drop_shadow_masked_src.as_str()),
        ("drop_shadow_mask", DROP_SHADOW_MASK_SHADER),
        ("blur_h", BLUR_H_SHADER),
        ("blur_v", BLUR_V_SHADER),
        ("blur_h_masked", blur_h_masked_src.as_str()),
        ("blur_v_masked", blur_v_masked_src.as_str()),
        ("blur_h_mask", BLUR_H_MASK_SHADER),
        ("blur_v_mask", BLUR_V_MASK_SHADER),
        ("downsample_nearest", DOWNSAMPLE_NEAREST_SHADER),
        ("upscale_nearest", UPSCALE_NEAREST_SHADER),
        ("upscale_nearest_masked", upscale_masked_src.as_str()),
        ("upscale_nearest_mask", UPSCALE_NEAREST_MASK_SHADER),
        ("backdrop_warp", BACKDROP_WARP_SHADER),
        ("backdrop_warp_image", BACKDROP_WARP_IMAGE_SHADER),
        ("backdrop_warp_masked", backdrop_warp_masked_src.as_str()),
        (
            "backdrop_warp_image_masked",
            backdrop_warp_image_masked_src.as_str(),
        ),
        ("backdrop_warp_mask", BACKDROP_WARP_MASK_SHADER),
        ("backdrop_warp_image_mask", BACKDROP_WARP_IMAGE_MASK_SHADER),
        ("color_adjust", COLOR_ADJUST_SHADER),
        ("color_adjust_masked", color_adjust_masked_src.as_str()),
        ("color_adjust_mask", COLOR_ADJUST_MASK_SHADER),
        ("color_matrix", COLOR_MATRIX_SHADER),
        ("color_matrix_masked", color_matrix_masked_src.as_str()),
        ("color_matrix_mask", COLOR_MATRIX_MASK_SHADER),
        ("alpha_threshold", ALPHA_THRESHOLD_SHADER),
        (
            "alpha_threshold_masked",
            alpha_threshold_masked_src.as_str(),
        ),
        ("alpha_threshold_mask", ALPHA_THRESHOLD_MASK_SHADER),
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("composite_premul_mask", COMPOSITE_PREMUL_MASK_SHADER),
        ("clip_mask", clip_mask_src.as_str()),
        ("path_clip_mask", PATH_CLIP_MASK_SHADER),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
        ("text_subpixel", TEXT_SUBPIXEL_SHADER),
        ("mask", MASK_SHADER),
        ("custom_effect", custom_effect_unmasked_src.as_str()),
        ("custom_effect_masked", custom_effect_masked_src.as_str()),
        ("custom_effect_mask", custom_effect_mask_src.as_str()),
        ("custom_effect_v2", custom_effect_v2_unmasked_src.as_str()),
        (
            "custom_effect_v2_masked",
            custom_effect_v2_masked_src.as_str(),
        ),
        ("custom_effect_v2_mask", custom_effect_v2_mask_src.as_str()),
        ("custom_effect_v3", custom_effect_v3_unmasked_src.as_str()),
        (
            "custom_effect_v3_masked",
            custom_effect_v3_masked_src.as_str(),
        ),
        ("custom_effect_v3_mask", custom_effect_v3_mask_src.as_str()),
    ] {
        naga::front::wgsl::parse_str(src)
            .unwrap_or_else(|err| panic!("WGSL parse failed for {name} shader: {err}"));
    }
}

#[test]
fn shaders_validate_for_webgpu() {
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    let quad_src = quad_shader_source();
    let clip_mask_src = clip_mask_shader_source();
    let upscale_masked_src = upscale_nearest_masked_shader_source();
    let backdrop_warp_masked_src = backdrop_warp_masked_shader_source();
    let backdrop_warp_image_masked_src = backdrop_warp_image_masked_shader_source();
    let color_adjust_masked_src = color_adjust_masked_shader_source();
    let color_matrix_masked_src = color_matrix_masked_shader_source();
    let alpha_threshold_masked_src = alpha_threshold_masked_shader_source();
    let drop_shadow_masked_src = drop_shadow_masked_shader_source();
    let blur_h_masked_src = blur_h_masked_shader_source();
    let blur_v_masked_src = blur_v_masked_shader_source();
    let custom_effect_unmasked_src =
        custom_effect_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_masked_src = custom_effect_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_mask_src = custom_effect_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_unmasked_src =
        custom_effect_v2_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_masked_src =
        custom_effect_v2_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v2_mask_src =
        custom_effect_v2_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_unmasked_src =
        custom_effect_v3_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_masked_src =
        custom_effect_v3_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    let custom_effect_v3_mask_src =
        custom_effect_v3_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", quad_src.as_str()),
        ("blit", BLIT_SHADER),
        ("blit_srgb_encode", BLIT_SRGB_ENCODE_SHADER),
        ("mip_downsample_box_2x2", MIP_DOWNSAMPLE_BOX_2X2_SHADER),
        ("drop_shadow", DROP_SHADOW_SHADER),
        ("drop_shadow_masked", drop_shadow_masked_src.as_str()),
        ("drop_shadow_mask", DROP_SHADOW_MASK_SHADER),
        ("blur_h", BLUR_H_SHADER),
        ("blur_v", BLUR_V_SHADER),
        ("blur_h_masked", blur_h_masked_src.as_str()),
        ("blur_v_masked", blur_v_masked_src.as_str()),
        ("blur_h_mask", BLUR_H_MASK_SHADER),
        ("blur_v_mask", BLUR_V_MASK_SHADER),
        ("downsample_nearest", DOWNSAMPLE_NEAREST_SHADER),
        ("upscale_nearest", UPSCALE_NEAREST_SHADER),
        ("upscale_nearest_masked", upscale_masked_src.as_str()),
        ("upscale_nearest_mask", UPSCALE_NEAREST_MASK_SHADER),
        ("backdrop_warp", BACKDROP_WARP_SHADER),
        ("backdrop_warp_image", BACKDROP_WARP_IMAGE_SHADER),
        ("backdrop_warp_masked", backdrop_warp_masked_src.as_str()),
        (
            "backdrop_warp_image_masked",
            backdrop_warp_image_masked_src.as_str(),
        ),
        ("backdrop_warp_mask", BACKDROP_WARP_MASK_SHADER),
        ("backdrop_warp_image_mask", BACKDROP_WARP_IMAGE_MASK_SHADER),
        ("color_adjust", COLOR_ADJUST_SHADER),
        ("color_adjust_masked", color_adjust_masked_src.as_str()),
        ("color_adjust_mask", COLOR_ADJUST_MASK_SHADER),
        ("color_matrix", COLOR_MATRIX_SHADER),
        ("color_matrix_masked", color_matrix_masked_src.as_str()),
        ("color_matrix_mask", COLOR_MATRIX_MASK_SHADER),
        ("alpha_threshold", ALPHA_THRESHOLD_SHADER),
        (
            "alpha_threshold_masked",
            alpha_threshold_masked_src.as_str(),
        ),
        ("alpha_threshold_mask", ALPHA_THRESHOLD_MASK_SHADER),
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("composite_premul_mask", COMPOSITE_PREMUL_MASK_SHADER),
        ("clip_mask", clip_mask_src.as_str()),
        ("path_clip_mask", PATH_CLIP_MASK_SHADER),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
        ("text_subpixel", TEXT_SUBPIXEL_SHADER),
        ("mask", MASK_SHADER),
        ("custom_effect", custom_effect_unmasked_src.as_str()),
        ("custom_effect_masked", custom_effect_masked_src.as_str()),
        ("custom_effect_mask", custom_effect_mask_src.as_str()),
        ("custom_effect_v2", custom_effect_v2_unmasked_src.as_str()),
        (
            "custom_effect_v2_masked",
            custom_effect_v2_masked_src.as_str(),
        ),
        ("custom_effect_v2_mask", custom_effect_v2_mask_src.as_str()),
        ("custom_effect_v3", custom_effect_v3_unmasked_src.as_str()),
        (
            "custom_effect_v3_masked",
            custom_effect_v3_masked_src.as_str(),
        ),
        ("custom_effect_v3_mask", custom_effect_v3_mask_src.as_str()),
    ] {
        let module = naga::front::wgsl::parse_str(src)
            .unwrap_or_else(|err| panic!("WGSL parse failed for {name} shader: {err}"));
        Validator::new(ValidationFlags::all(), Capabilities::empty())
            .validate(&module)
            .unwrap_or_else(|err| panic!("WGSL validation failed for {name} shader: {err}"));
    }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-webgpu-tests"))]
mod webgpu_tint_guardrail {
    use super::*;
    use wasm_bindgen_test::*;

    const CUSTOM_EFFECT_DERIVATIVES_SMOKE_WGSL: &str = r#"
fn fret_custom_effect(tex: vec4<f32>, uv: vec2<f32>, pos_px: vec2<f32>, params: EffectParamsV1) -> vec4<f32> {
  // WebGPU/Tint requires derivatives to be used from uniform control flow.
  // This shader ensures the *host* custom effect fragment shaders do not guard the custom effect call
  // behind non-uniform bounds checks or early returns.
  let d = fwidth(pos_px.x);
  return tex + vec4<f32>(0.0, 0.0, 0.0, 0.0) * d;
}
"#;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn request_webgpu_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("WebGPU adapter must be available in browser tests");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("request_device must succeed in browser tests")
    }

    #[wasm_bindgen_test(async)]
    async fn webgpu_tint_compiles_all_wgsl_shaders() {
        let (device, _queue) = request_webgpu_device().await;

        let quad_src = quad_shader_source();
        let clip_mask_src = clip_mask_shader_source();
        let upscale_masked_src = upscale_nearest_masked_shader_source();
        let color_adjust_masked_src = color_adjust_masked_shader_source();
        let color_matrix_masked_src = color_matrix_masked_shader_source();
        let alpha_threshold_masked_src = alpha_threshold_masked_shader_source();
        let drop_shadow_masked_src = drop_shadow_masked_shader_source();
        let blur_h_masked_src = blur_h_masked_shader_source();
        let blur_v_masked_src = blur_v_masked_shader_source();
        let custom_effect_unmasked_src =
            custom_effect_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_masked_src =
            custom_effect_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_mask_src = custom_effect_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_v2_unmasked_src =
            custom_effect_v2_unmasked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_v2_masked_src =
            custom_effect_v2_masked_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_v2_mask_src =
            custom_effect_v2_mask_shader_source(CUSTOM_EFFECT_IDENTITY_WGSL);
        let custom_effect_v2_unmasked_derivatives_src =
            custom_effect_v2_unmasked_shader_source(CUSTOM_EFFECT_DERIVATIVES_SMOKE_WGSL);
        let custom_effect_v2_masked_derivatives_src =
            custom_effect_v2_masked_shader_source(CUSTOM_EFFECT_DERIVATIVES_SMOKE_WGSL);
        let custom_effect_v2_mask_derivatives_src =
            custom_effect_v2_mask_shader_source(CUSTOM_EFFECT_DERIVATIVES_SMOKE_WGSL);
        for (name, src) in [
            ("viewport", VIEWPORT_SHADER),
            ("quad", quad_src.as_str()),
            ("blit", BLIT_SHADER),
            ("drop_shadow", DROP_SHADOW_SHADER),
            ("drop_shadow_masked", drop_shadow_masked_src.as_str()),
            ("drop_shadow_mask", DROP_SHADOW_MASK_SHADER),
            ("blur_h", BLUR_H_SHADER),
            ("blur_v", BLUR_V_SHADER),
            ("blur_h_masked", blur_h_masked_src.as_str()),
            ("blur_v_masked", blur_v_masked_src.as_str()),
            ("blur_h_mask", BLUR_H_MASK_SHADER),
            ("blur_v_mask", BLUR_V_MASK_SHADER),
            ("downsample_nearest", DOWNSAMPLE_NEAREST_SHADER),
            ("upscale_nearest", UPSCALE_NEAREST_SHADER),
            ("upscale_nearest_masked", upscale_masked_src.as_str()),
            ("upscale_nearest_mask", UPSCALE_NEAREST_MASK_SHADER),
            ("color_adjust", COLOR_ADJUST_SHADER),
            ("color_adjust_masked", color_adjust_masked_src.as_str()),
            ("color_adjust_mask", COLOR_ADJUST_MASK_SHADER),
            ("color_matrix", COLOR_MATRIX_SHADER),
            ("color_matrix_masked", color_matrix_masked_src.as_str()),
            ("color_matrix_mask", COLOR_MATRIX_MASK_SHADER),
            ("alpha_threshold", ALPHA_THRESHOLD_SHADER),
            (
                "alpha_threshold_masked",
                alpha_threshold_masked_src.as_str(),
            ),
            ("alpha_threshold_mask", ALPHA_THRESHOLD_MASK_SHADER),
            ("composite_premul", COMPOSITE_PREMUL_SHADER),
            ("composite_premul_mask", COMPOSITE_PREMUL_MASK_SHADER),
            ("clip_mask", clip_mask_src.as_str()),
            ("path_clip_mask", PATH_CLIP_MASK_SHADER),
            ("path", PATH_SHADER),
            ("text", TEXT_SHADER),
            ("text_color", TEXT_COLOR_SHADER),
            ("text_subpixel", TEXT_SUBPIXEL_SHADER),
            ("mask", MASK_SHADER),
            ("custom_effect", custom_effect_unmasked_src.as_str()),
            ("custom_effect_masked", custom_effect_masked_src.as_str()),
            ("custom_effect_mask", custom_effect_mask_src.as_str()),
            ("custom_effect_v2", custom_effect_v2_unmasked_src.as_str()),
            (
                "custom_effect_v2_masked",
                custom_effect_v2_masked_src.as_str(),
            ),
            ("custom_effect_v2_mask", custom_effect_v2_mask_src.as_str()),
            (
                "custom_effect_v2_derivatives",
                custom_effect_v2_unmasked_derivatives_src.as_str(),
            ),
            (
                "custom_effect_v2_masked_derivatives",
                custom_effect_v2_masked_derivatives_src.as_str(),
            ),
            (
                "custom_effect_v2_mask_derivatives",
                custom_effect_v2_mask_derivatives_src.as_str(),
            ),
        ] {
            let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
            let _module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });
            let err = error_scope.pop().await;
            assert!(
                err.is_none(),
                "WebGPU/Tint validation failed for {name} shader: {err:?}"
            );
        }
    }
}

#[test]
fn transform_rows_match_apply_point() {
    let t = Transform2D {
        a: 1.3,
        b: -0.2,
        c: 0.7,
        d: 0.9,
        tx: 10.0,
        ty: -5.0,
    };
    let row0 = [t.a, t.c, t.tx, 0.0];
    let row1 = [t.b, t.d, t.ty, 0.0];

    for (x, y) in [(0.0, 0.0), (12.5, -3.25), (-100.0, 50.0)] {
        let p = t.apply_point(Point::new(Px(x), Px(y)));
        let x2 = row0[0] * x + row0[1] * y + row0[2];
        let y2 = row1[0] * x + row1[1] * y + row1[2];
        assert!((p.x.0 - x2).abs() < 1e-4);
        assert!((p.y.0 - y2).abs() < 1e-4);
    }
}

#[test]
fn inverse_rows_match_apply_point() {
    let t = Transform2D {
        a: 1.3,
        b: -0.2,
        c: 0.7,
        d: 0.9,
        tx: 10.0,
        ty: -5.0,
    };
    let inv = t.inverse().expect("invertible");
    let inv0 = [inv.a, inv.c, inv.tx, 0.0];
    let inv1 = [inv.b, inv.d, inv.ty, 0.0];

    for (x, y) in [(0.0, 0.0), (12.5, -3.25), (-100.0, 50.0)] {
        let p = inv.apply_point(Point::new(Px(x), Px(y)));
        let x2 = inv0[0] * x + inv0[1] * y + inv0[2];
        let y2 = inv1[0] * x + inv1[1] * y + inv1[2];
        assert!((p.x.0 - x2).abs() < 1e-4);
        assert!((p.y.0 - y2).abs() < 1e-4);
    }
}

#[test]
fn corner_radii_are_clamped_to_half_min_rect_dim() {
    let radii = clamp_corner_radii_for_rect(100.0, 6.0, [999.0, 999.0, 999.0, 999.0]);
    assert_eq!(radii, [3.0, 3.0, 3.0, 3.0]);
}

#[test]
fn corner_radii_clamp_is_nan_safe() {
    let radii = clamp_corner_radii_for_rect(f32::NAN, 6.0, [999.0, -1.0, f32::NAN, 0.0]);
    assert_eq!(radii, [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn svg_draw_rect_centers_contained_raster() {
    // target 100x50, raster 100x100 at smooth=2 => draw 50x50 centered.
    let (x0, y0, x1, y1) = svg_draw_rect_px(
        0.0,
        0.0,
        100.0,
        50.0,
        (100, 100),
        2.0,
        fret_core::SvgFit::Contain,
    );
    assert_eq!((x0, y0, x1, y1), (25.0, 0.0, 75.0, 50.0));
}

#[test]
fn svg_draw_rect_width_can_overflow_height() {
    // target 50x50, raster 100x200 at smooth=2 => draw 50x100, centered (overflows vertically).
    let (x0, y0, x1, y1) = svg_draw_rect_px(
        0.0,
        0.0,
        50.0,
        50.0,
        (100, 200),
        2.0,
        fret_core::SvgFit::Width,
    );
    assert_eq!((x0, y0, x1, y1), (0.0, -25.0, 50.0, 75.0));
}

#[test]
fn image_fit_cover_encodes_cropped_uvs() {
    use crate::images::{AlphaMode, ImageColorSpace, ImageDescriptor};

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let source_size = (200u32, 100u32);
    let source_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image fit cover test source"),
        size: wgpu::Extent3d {
            width: source_size.0,
            height: source_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let source_view = source_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let image = renderer.register_image(ImageDescriptor {
        view: source_view,
        size: source_size,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        color_space: ImageColorSpace::Srgb,
        alpha_mode: AlphaMode::Opaque,
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image fit cover test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Image {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        image,
        fit: ViewportFit::Cover,
        sampling: fret_core::scene::ImageSamplingHint::Default,
        opacity: 1.0,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::Image(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one image draw");
    };
    assert_eq!(draw.vertex_count, 6);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 6];

    assert_vertex(&verts[0], (0.0, 0.0), (0.25, 0.0));
    assert_vertex(&verts[1], (100.0, 0.0), (0.75, 0.0));
    assert_vertex(&verts[2], (100.0, 100.0), (0.75, 1.0));
    assert_vertex(&verts[3], (0.0, 0.0), (0.25, 0.0));
    assert_vertex(&verts[4], (100.0, 100.0), (0.75, 1.0));
    assert_vertex(&verts[5], (0.0, 100.0), (0.25, 1.0));
}

#[test]
fn image_fit_contain_encodes_centered_draw_rect() {
    use crate::images::{AlphaMode, ImageColorSpace, ImageDescriptor};

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let source_size = (200u32, 100u32);
    let source_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image fit contain test source"),
        size: wgpu::Extent3d {
            width: source_size.0,
            height: source_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let source_view = source_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let image = renderer.register_image(ImageDescriptor {
        view: source_view,
        size: source_size,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        color_space: ImageColorSpace::Srgb,
        alpha_mode: AlphaMode::Opaque,
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image fit contain test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Image {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        image,
        fit: ViewportFit::Contain,
        sampling: fret_core::scene::ImageSamplingHint::Default,
        opacity: 1.0,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::Image(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one image draw");
    };
    assert_eq!(draw.vertex_count, 6);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 6];

    // Contain: 200x100 in 100x100 -> 100x50 centered at y=25.
    assert_vertex(&verts[0], (0.0, 25.0), (0.0, 0.0));
    assert_vertex(&verts[1], (100.0, 25.0), (1.0, 0.0));
    assert_vertex(&verts[2], (100.0, 75.0), (1.0, 1.0));
    assert_vertex(&verts[3], (0.0, 25.0), (0.0, 0.0));
    assert_vertex(&verts[4], (100.0, 75.0), (1.0, 1.0));
    assert_vertex(&verts[5], (0.0, 75.0), (0.0, 1.0));
}

#[test]
fn vertex_color_quad_encodes_two_triangles_with_corner_colors() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vertex color quad encode test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let points = [
        Point::new(Px(10.0), Px(20.0)),
        Point::new(Px(70.0), Px(16.0)),
        Point::new(Px(80.0), Px(60.0)),
        Point::new(Px(12.0), Px(72.0)),
    ];
    let colors = [
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 0.75,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 0.5,
        },
        Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.25,
        },
    ];
    let mut scene = Scene::default();
    scene.push(SceneOp::VertexColorQuad {
        order: DrawOrder(0),
        points,
        colors,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::VertexColor(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one vertex color draw");
    };
    assert_eq!(draw.vertex_count, 6);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 6];

    assert_vertex(&verts[0], (10.0, 20.0), (0.0, 0.0));
    assert_vertex_color(&verts[0], [1.0, 0.0, 0.0, 1.0]);
    assert_vertex(&verts[1], (70.0, 16.0), (0.0, 0.0));
    assert_vertex_color(&verts[1], [0.0, 1.0, 0.0, 0.75]);
    assert_vertex(&verts[2], (80.0, 60.0), (0.0, 0.0));
    assert_vertex_color(&verts[2], [0.0, 0.0, 1.0, 0.5]);
    assert_vertex(&verts[3], (10.0, 20.0), (0.0, 0.0));
    assert_vertex_color(&verts[3], [1.0, 0.0, 0.0, 1.0]);
    assert_vertex(&verts[4], (80.0, 60.0), (0.0, 0.0));
    assert_vertex_color(&verts[4], [0.0, 0.0, 1.0, 0.5]);
    assert_vertex(&verts[5], (12.0, 72.0), (0.0, 0.0));
    assert_vertex_color(&verts[5], [1.0, 1.0, 0.0, 0.25]);
}

#[test]
fn vertex_color_triangle_encodes_three_custom_vertices() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vertex color triangle encode test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let vertices = [
        SceneMeshVertex::colored(
            Point::new(Px(9.0), Px(11.0)),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ),
        SceneMeshVertex::colored(
            Point::new(Px(61.0), Px(13.0)),
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 0.75,
            },
        ),
        SceneMeshVertex::colored(
            Point::new(Px(22.0), Px(70.0)),
            Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 0.5,
            },
        ),
    ];

    let mut scene = Scene::default();
    scene.push(SceneOp::VertexColorTriangle {
        order: DrawOrder(0),
        vertices,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::VertexColor(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one vertex color draw");
    };
    assert_eq!(draw.vertex_count, 3);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 3];

    assert_vertex(&verts[0], (9.0, 11.0), (0.0, 0.0));
    assert_vertex_color(&verts[0], [1.0, 0.0, 0.0, 1.0]);
    assert_vertex(&verts[1], (61.0, 13.0), (0.0, 0.0));
    assert_vertex_color(&verts[1], [0.0, 1.0, 0.0, 0.75]);
    assert_vertex(&verts[2], (22.0, 70.0), (0.0, 0.0));
    assert_vertex_color(&verts[2], [0.0, 0.0, 1.0, 0.5]);
}

#[test]
fn image_quad_encodes_custom_points_uvs_and_tint() {
    use crate::images::{AlphaMode, ImageColorSpace, ImageDescriptor};

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let source_size = (64u32, 32u32);
    let source_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image quad encode test source"),
        size: wgpu::Extent3d {
            width: source_size.0,
            height: source_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let source_view = source_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let image = renderer.register_image(ImageDescriptor {
        view: source_view,
        size: source_size,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        color_space: ImageColorSpace::Srgb,
        alpha_mode: AlphaMode::Opaque,
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image quad encode test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let points = [
        Point::new(Px(8.0), Px(12.0)),
        Point::new(Px(72.0), Px(10.0)),
        Point::new(Px(76.0), Px(48.0)),
        Point::new(Px(6.0), Px(52.0)),
    ];
    let uvs = [
        UvPoint { u: 0.125, v: 0.25 },
        UvPoint { u: 0.875, v: 0.2 },
        UvPoint { u: 0.75, v: 0.95 },
        UvPoint { u: 0.1, v: 0.8 },
    ];
    let tint = Color {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 0.8,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::ImageQuad {
        order: DrawOrder(0),
        points,
        image,
        uvs,
        sampling: fret_core::scene::ImageSamplingHint::Default,
        tint,
        opacity: 0.5,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::Image(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one image draw");
    };
    assert_eq!(draw.vertex_count, 6);
    assert_eq!(draw.image, image);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 6];
    let expected_tint = [0.25, 0.5, 0.75, 0.8];

    assert_vertex(&verts[0], (8.0, 12.0), (0.125, 0.25));
    assert_vertex(&verts[1], (72.0, 10.0), (0.875, 0.2));
    assert_vertex(&verts[2], (76.0, 48.0), (0.75, 0.95));
    assert_vertex(&verts[3], (8.0, 12.0), (0.125, 0.25));
    assert_vertex(&verts[4], (76.0, 48.0), (0.75, 0.95));
    assert_vertex(&verts[5], (6.0, 52.0), (0.1, 0.8));
    for vertex in verts {
        assert_vertex_color(vertex, expected_tint);
        assert_vertex_opacity_and_premul(vertex, 0.5, 0.0);
    }
}

#[test]
fn image_triangle_encodes_custom_uvs_and_vertex_colors() {
    use crate::images::{AlphaMode, ImageColorSpace, ImageDescriptor};

    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let source_size = (64u32, 32u32);
    let source_tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image triangle encode test source"),
        size: wgpu::Extent3d {
            width: source_size.0,
            height: source_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let source_view = source_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let image = renderer.register_image(ImageDescriptor {
        view: source_view,
        size: source_size,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        color_space: ImageColorSpace::Srgb,
        alpha_mode: AlphaMode::Opaque,
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image triangle encode test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let vertices = [
        SceneMeshVertex::new(
            Point::new(Px(10.0), Px(12.0)),
            UvPoint { u: 0.0, v: 0.0 },
            Color {
                r: 1.0,
                g: 0.5,
                b: 0.25,
                a: 1.0,
            },
        ),
        SceneMeshVertex::new(
            Point::new(Px(70.0), Px(16.0)),
            UvPoint { u: 1.0, v: 0.125 },
            Color {
                r: 0.75,
                g: 1.0,
                b: 0.5,
                a: 0.8,
            },
        ),
        SceneMeshVertex::new(
            Point::new(Px(24.0), Px(68.0)),
            UvPoint { u: 0.25, v: 0.9 },
            Color {
                r: 0.25,
                g: 0.5,
                b: 1.0,
                a: 0.6,
            },
        ),
    ];

    let mut scene = Scene::default();
    scene.push(SceneOp::ImageTriangle {
        order: DrawOrder(0),
        image,
        vertices,
        sampling: fret_core::scene::ImageSamplingHint::Default,
        opacity: 0.5,
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::Image(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one image draw");
    };
    assert_eq!(draw.vertex_count, 3);
    assert_eq!(draw.image, image);

    let first = draw.first_vertex as usize;
    let verts = &encoding.viewport_vertices[first..first + 3];

    assert_vertex(&verts[0], (10.0, 12.0), (0.0, 0.0));
    assert_vertex_color(&verts[0], [1.0, 0.5, 0.25, 1.0]);
    assert_vertex_opacity_and_premul(&verts[0], 0.5, 0.0);
    assert_vertex(&verts[1], (70.0, 16.0), (1.0, 0.125));
    assert_vertex_color(&verts[1], [0.75, 1.0, 0.5, 0.8]);
    assert_vertex_opacity_and_premul(&verts[1], 0.5, 0.0);
    assert_vertex(&verts[2], (24.0, 68.0), (0.25, 0.9));
    assert_vertex_color(&verts[2], [0.25, 0.5, 1.0, 0.6]);
    assert_vertex_opacity_and_premul(&verts[2], 0.5, 0.0);
}

#[test]
fn shadow_rrect_encodes_shadow_quad_instance() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (128u32, 128u32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow rrect encode test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::ShadowRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(20.0), Px(16.0)),
            Size::new(Px(40.0), Px(24.0)),
        ),
        corner_radii: Corners::all(Px(8.0)),
        offset: Point::new(Px(6.0), Px(4.0)),
        spread: Px(3.0),
        blur_radius: Px(7.0),
        color: fret_core::scene::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.4,
        },
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let encoding = renderer.scene_encoding_state.cache();
    let [super::types::OrderedDraw::Quad(draw)] = encoding.ordered_draws.as_slice() else {
        panic!("expected exactly one quad draw");
    };
    assert!(
        draw.pipeline.shadow_mode,
        "shadow path must use shadow-mode pipeline"
    );
    assert_eq!(draw.instance_count, 1);

    let inst = &encoding.instances[draw.first_instance as usize];
    assert_eq!(inst.rect, [20.0, 16.0, 40.0, 24.0]);
    assert_eq!(inst.corner_radii, [8.0, 8.0, 8.0, 8.0]);
    assert_eq!(inst.shadow_params, [6.0, 4.0, 3.0, 7.0]);
    assert_eq!(inst.border, [0.0; 4]);
}

#[test]
fn scene_encoding_cache_is_busted_by_text_quality_changes() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (32, 32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("scene encoding cache test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let scene = fret_core::scene::Scene::default();
    let make_params = || super::RenderSceneParams {
        format,
        target_view: &target_view,
        source: super::RenderSceneSourceSelection::flat_compat(&scene),
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key0 = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");
    assert_eq!(renderer.diagnostics_state.perf.scene_encoding_cache_hits, 0);
    assert_eq!(
        renderer.diagnostics_state.perf.scene_encoding_cache_misses,
        1
    );
    assert_eq!(
        renderer
            .diagnostics_state
            .perf
            .scene_encoding_cache_last_miss_reasons,
        1 << 0
    );

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key1 = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");
    assert_eq!(key1, key0);
    assert_eq!(renderer.diagnostics_state.perf.scene_encoding_cache_hits, 1);

    let changed = renderer.set_text_quality_settings(crate::text::TextQualitySettings {
        gamma: 1.7,
        ..Default::default()
    });
    assert!(changed);

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key2 = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");
    assert_ne!(key2, key0);
    assert_eq!(
        renderer.diagnostics_state.perf.scene_encoding_cache_misses,
        2
    );
    assert_ne!(
        renderer
            .diagnostics_state
            .perf
            .scene_encoding_cache_last_miss_reasons,
        0
    );
    assert_ne!(
        renderer
            .diagnostics_state
            .perf
            .scene_encoding_cache_last_miss_reasons,
        1 << 0
    );
    assert_ne!(
        renderer
            .diagnostics_state
            .perf
            .scene_encoding_cache_last_miss_reasons
            & (1 << 9),
        0
    );
}

#[test]
fn diagnostic_scene_chunk_manifest_does_not_override_flat_scene_encoding() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (32, 32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("diagnostic scene chunk input test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 0.25,
            g: 0.5,
            b: 0.75,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk.clone(),
        Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        Point::new(Px(2.0), Px(3.0)),
    ));

    let params = |source| super::RenderSceneParams {
        format,
        target_view: &target_view,
        source,
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        params(super::RenderSceneSourceSelection::flat_compat(&scene)),
    );
    let key_without_manifest = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        params(super::select_render_scene_source(
            &scene,
            &manifest,
            super::RenderSceneSourcePolicy::flat_compat(),
        )),
    );
    let key_with_diagnostic_manifest = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");
    let last = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");

    assert_eq!(
        key_with_diagnostic_manifest, key_without_manifest,
        "diagnostic manifests must not switch flat render input to chunk-native encoding"
    );
    assert_eq!(last.scene_encoding_cache_hits, 1);
    assert_eq!(last.scene_encoding_cache_misses, 0);
    assert_eq!(last.scene_chunk_input_chunks, 1);
    assert_eq!(last.scene_chunk_input_ops, chunk.ops_len() as u64);
    assert_eq!(last.scene_chunk_input_fingerprint, manifest.fingerprint());
}

#[test]
fn source_selection_flat_compat_keeps_manifest_as_assembly_sidecar() {
    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));

    let selection = super::select_render_scene_source(
        &scene,
        &manifest,
        super::RenderSceneSourcePolicy::flat_compat(),
    );

    assert!(matches!(
        selection.source(),
        super::RenderSceneSource::FlatCompat { .. }
    ));
    assert_eq!(
        selection
            .assembly_manifest()
            .map(fret_core::SceneChunkManifest::len),
        Some(1)
    );
    assert_eq!(
        selection.chunk_support(),
        super::ChunkLaunchSupport::Supported {
            stream_class: super::ChunkLaunchStreamClass::ResourceFreeQuad,
        }
    );
    assert!(!selection.debug_flat_oracle_requested());
}

#[test]
fn source_selection_promotes_resource_free_vertex_color_manifest() {
    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(0.0)),
        Point::new(Px(10.0), Px(10.0)),
        Point::new(Px(0.0), Px(10.0)),
    ];
    let colors = [Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }; 4];
    let mut scene = Scene::default();
    scene.push(SceneOp::VertexColorQuad {
        order: DrawOrder(0),
        points,
        colors,
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));

    let selection = super::select_render_scene_source(
        &scene,
        &manifest,
        super::RenderSceneSourcePolicy::chunk_manifest_when_supported(),
    );

    assert!(matches!(
        selection.source(),
        super::RenderSceneSource::ChunkManifest { .. }
    ));
    assert_eq!(
        selection.chunk_support(),
        super::ChunkLaunchSupport::Supported {
            stream_class: super::ChunkLaunchStreamClass::ResourceFreeVertexColor,
        }
    );
}

#[test]
fn source_selection_blocks_side_table_manifest_with_structured_reason() {
    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRRect {
        rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        corner_radii: Corners::all(Px(2.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopClip);
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));

    let selection = super::select_render_scene_source(
        &scene,
        &manifest,
        super::RenderSceneSourcePolicy::chunk_manifest_when_supported(),
    );

    assert!(matches!(
        selection.source(),
        super::RenderSceneSource::FlatCompat { .. }
    ));
    assert_eq!(
        selection.chunk_support(),
        super::ChunkLaunchSupport::Unsupported {
            stream_class: Some(super::ChunkLaunchStreamClass::ResourceFreeQuad),
            reason: super::ChunkLaunchUnsupportedReason::ManifestUnsupported(
                fret_core::SceneChunkManifestUnsupportedReason::EntrySideTableRequired {
                    entry_index: 0,
                    requirements: fret_core::SceneChunkSideTableRequirements {
                        clip_scopes: 1,
                        ..Default::default()
                    },
                },
            ),
        }
    );
}

#[test]
fn source_selection_debug_flat_oracle_does_not_define_chunk_support() {
    let mut quad_scene = Scene::default();
    quad_scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(0.0)),
        Point::new(Px(10.0), Px(10.0)),
        Point::new(Px(0.0), Px(10.0)),
    ];
    let colors = [Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }; 4];
    let mut vertex_scene = Scene::default();
    vertex_scene.push(SceneOp::VertexColorQuad {
        order: DrawOrder(1),
        points,
        colors,
    });

    let mut debug_scene = Scene::default();
    for op in quad_scene.ops().iter().chain(vertex_scene.ops().iter()) {
        debug_scene.push(*op);
    }

    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        fret_core::SceneChunk::from_scene(&quad_scene),
        Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        fret_core::SceneChunk::from_scene(&vertex_scene),
        Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));

    let selection = super::select_render_scene_source(
        &debug_scene,
        &manifest,
        super::RenderSceneSourcePolicy::chunk_manifest_when_supported().with_debug_flat_oracle(),
    );

    assert!(matches!(
        selection.source(),
        super::RenderSceneSource::FlatCompat { .. }
    ));
    assert!(selection.debug_flat_oracle_requested());
    assert_eq!(
        selection.chunk_support(),
        super::ChunkLaunchSupport::Unsupported {
            stream_class: Some(super::ChunkLaunchStreamClass::Mixed),
            reason: super::ChunkLaunchUnsupportedReason::MixedStreams,
        }
    );
}

#[test]
fn resource_free_quad_scene_chunk_manifest_uses_chunk_native_scene_encoding_key() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (32, 32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("scene chunk input test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 0.25,
            g: 0.5,
            b: 0.75,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk.clone(),
        Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        Point::new(Px(2.0), Px(3.0)),
    ));

    let params = |source| super::RenderSceneParams {
        format,
        target_view: &target_view,
        source,
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        params(super::RenderSceneSourceSelection::flat_compat(&scene)),
    );
    let key_without_manifest = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        params(super::RenderSceneSourceSelection::chunk_manifest(
            &manifest,
            Some(&scene),
        )),
    );
    let key_with_manifest = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");

    assert_ne!(
        key_with_manifest, key_without_manifest,
        "resource-free quad manifests should use a chunk-native scene encoding key"
    );

    let last = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");
    assert_eq!(last.scene_encoding_cache_hits, 0);
    assert_eq!(last.scene_encoding_cache_misses, 1);
    assert_eq!(last.render_scene_source_chunk_manifest_frames, 1);
    assert_eq!(last.render_scene_source_flat_compat_frames, 0);
    assert_eq!(last.render_scene_source_flat_compat_unsupported_frames, 0);
    assert_eq!(last.scene_chunk_input_chunks, 1);
    assert_eq!(last.scene_chunk_input_ops, chunk.ops_len() as u64);
    assert_eq!(last.scene_chunk_input_fingerprint, manifest.fingerprint());
    assert_eq!(last.scene_chunk_encoding_key_cache_entries, 1);
    assert_eq!(last.scene_chunk_encoding_key_cache_hits, 0);
    assert_eq!(last.scene_chunk_encoding_key_cache_misses, 1);
    assert_eq!(last.scene_chunk_encoding_key_cache_stale_entries, 0);
    assert_ne!(last.scene_chunk_encoding_key_cache_context_fingerprint, 0);
    assert_eq!(last.scene_chunk_encoding_payload_cache_hits, 0);
    assert_eq!(last.scene_chunk_encoding_payload_cache_misses, 1);
    assert_eq!(last.scene_chunk_encoding_payload_chunks_encoded, 1);
    assert!(last.scene_chunk_encoding_payload_bytes_estimate > 0);
    assert_eq!(last.scene_chunk_encoding_payload_entries_live, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_candidate_segments, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_shape_matches, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_shape_mismatches, 0);
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_stream_fingerprint_matches,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_dry_run_candidates,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_append_only_matches,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_entries_without_plan_candidate,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_candidates_without_payload,
        0
    );

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        params(super::RenderSceneSourceSelection::chunk_manifest(
            &manifest,
            Some(&scene),
        )),
    );
    let key_with_stable_manifest = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key");

    assert_eq!(key_with_stable_manifest, key_with_manifest);
    let last = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");
    assert_eq!(last.scene_encoding_cache_hits, 1);
    assert_eq!(last.scene_encoding_cache_misses, 0);
    assert_eq!(last.scene_chunk_encoding_key_cache_entries, 1);
    assert_eq!(last.scene_chunk_encoding_key_cache_hits, 1);
    assert_eq!(last.scene_chunk_encoding_key_cache_misses, 0);
    assert_eq!(last.scene_chunk_encoding_key_cache_stale_entries, 0);
    assert_eq!(last.scene_chunk_encoding_payload_cache_hits, 1);
    assert_eq!(last.scene_chunk_encoding_payload_cache_misses, 0);
    assert_eq!(last.scene_chunk_encoding_payload_chunks_encoded, 0);
    assert!(last.scene_chunk_encoding_payload_bytes_estimate > 0);
    assert_eq!(last.scene_chunk_encoding_payload_entries_live, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_candidate_segments, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_shape_matches, 1);
    assert_eq!(last.scene_chunk_encoding_payload_plan_shape_mismatches, 0);
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_stream_fingerprint_matches,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_stream_fingerprint_mismatches,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_dry_run_candidates,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_append_only_matches,
        1
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_entries_without_plan_candidate,
        0
    );
    assert_eq!(
        last.scene_chunk_encoding_payload_plan_candidates_without_payload,
        0
    );
}

#[test]
fn unsupported_side_table_manifest_renders_flat_compat_with_reason_counters() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (32, 32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("side table source fallback test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::PushClipRRect {
        rect: Rect::new(Point::default(), Size::new(Px(12.0), Px(12.0))),
        corner_radii: Corners::all(Px(2.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 0.25,
            g: 0.5,
            b: 0.75,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopClip);
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), Size::new(Px(12.0), Px(12.0))),
        Point::default(),
    ));

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::select_render_scene_source(
                &scene,
                &manifest,
                super::RenderSceneSourcePolicy::chunk_manifest_when_supported(),
            ),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let last = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");
    assert_eq!(last.render_scene_source_chunk_manifest_frames, 0);
    assert_eq!(last.render_scene_source_flat_compat_frames, 1);
    assert_eq!(last.render_scene_source_flat_compat_unsupported_frames, 1);
    assert_eq!(last.render_scene_source_unsupported_side_tables, 1);
    assert_eq!(last.render_scene_source_unsupported_scope, 0);
    assert_eq!(last.scene_chunk_input_chunks, 1);
}

#[test]
fn unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (64, 64);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("text resource key scene cache test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let style = TextStyle {
        size: Px(16.0),
        ..Default::default()
    };
    let (text_a, _) = fret_core::TextService::prepare_str(
        &mut renderer,
        "aaaa",
        &style,
        TextConstraints::default(),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(20.0)),
        text: text_a,
        paint: fret_core::Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
        .into(),
        outline: None,
        shadow: None,
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), Size::new(Px(40.0), Px(24.0))),
        Point::default(),
    ));

    let params = || super::RenderSceneParams {
        format,
        target_view: &target_view,
        source: super::select_render_scene_source(
            &scene,
            &manifest,
            super::RenderSceneSourcePolicy::flat_compat(),
        ),
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());
    let warmed_key = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key after warmup");

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());
    let warmed_frame = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");
    assert_eq!(warmed_frame.scene_encoding_cache_hits, 1);
    assert_eq!(warmed_frame.scene_chunk_encoding_key_cache_hits, 1);
    assert_eq!(warmed_frame.scene_chunk_encoding_payload_cache_hits, 1);

    let atlas_revision_before = renderer.text_system.atlas_revision();
    let (text_b, _) = fret_core::TextService::prepare_str(
        &mut renderer,
        "zzzz",
        &style,
        TextConstraints::default(),
    );
    assert_eq!(
        renderer.text_system.atlas_revision(),
        atlas_revision_before,
        "unreferenced prepare should not create atlas churn"
    );
    let mut scene_b = Scene::default();
    scene_b.push(SceneOp::Text {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(20.0)),
        text: text_b,
        paint: fret_core::Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
        .into(),
        outline: None,
        shadow: None,
    });
    renderer
        .text_system
        .test_prepare_full_scene_text(&scene_b, 99);
    assert_ne!(
        renderer.text_system.atlas_revision(),
        atlas_revision_before,
        "test setup should create unreferenced atlas churn through frame residency"
    );

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());
    let churn_key = renderer
        .scene_encoding_state
        .cache_key()
        .expect("scene encoding key after unreferenced text churn");
    let churn_frame = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");

    assert_eq!(churn_key, warmed_key);
    assert_eq!(churn_frame.scene_encoding_cache_hits, 1);
    assert_eq!(churn_frame.scene_encoding_cache_misses, 0);
    assert_eq!(churn_frame.scene_chunk_encoding_key_cache_hits, 1);
    assert_eq!(churn_frame.scene_chunk_encoding_payload_cache_hits, 1);
    assert_eq!(
        churn_frame.text_atlas_revision_changed_scene_text_resources_stable,
        1
    );
    assert_eq!(
        churn_frame
            .scene_encoding_cache_miss_histogram
            .text_scene_resource_key_changed,
        0
    );
}

#[test]
fn scene_chunk_payload_and_resident_upload_state_warm_without_perf() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (32, 32);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("scene chunk resident warmup test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        background: Color {
            r: 0.25,
            g: 0.5,
            b: 0.75,
            a: 1.0,
        }
        .into(),
        border: fret_core::Edges::all(Px(0.0)),
        border_paint: Color::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    let chunk = fret_core::SceneChunk::from_scene(&scene);
    let mut manifest = fret_core::SceneChunkManifest::default();
    manifest.push(fret_core::SceneChunkManifestEntry::new(
        chunk,
        Rect::new(Point::default(), fret_core::Size::new(Px(10.0), Px(10.0))),
        Point::default(),
    ));

    let params = || super::RenderSceneParams {
        format,
        target_view: &target_view,
        source: super::RenderSceneSourceSelection::chunk_manifest(&manifest, Some(&scene)),
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());
    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());
    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());

    renderer.set_perf_enabled(true);
    let _ = renderer.render_scene(&ctx.device, &ctx.queue, params());

    let last = renderer
        .diagnostics_state
        .last_frame_perf
        .expect("last frame perf snapshot");
    assert_eq!(last.scene_chunk_encoding_payload_cache_hits, 1);
    assert_eq!(
        last.scene_chunk_encoding_payload_reassembly_append_only_matches,
        1
    );
    assert_eq!(last.geometry_upload.resident_stream_candidates, 1);
    assert_eq!(last.geometry_upload.resident_stream_hits, 1);
    assert_eq!(last.geometry_upload.resident_stream_misses, 0);
    assert_eq!(last.geometry_upload.quad_instance_write_count, 0);
    assert_eq!(last.geometry_upload.quad_instance_bytes, 0);
    assert_eq!(last.instance_bytes, 0);
}

#[test]
fn perf_snapshot_counts_path_material_paint_degradation() {
    let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
    let mut renderer = super::Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (64, 64);
    let target = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("path material paint perf test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    let cmds = [
        PathCommand::MoveTo(Point::new(Px(10.0), Px(10.0))),
        PathCommand::LineTo(Point::new(Px(50.0), Px(10.0))),
        PathCommand::LineTo(Point::new(Px(50.0), Px(50.0))),
        PathCommand::LineTo(Point::new(Px(10.0), Px(50.0))),
        PathCommand::Close,
    ];
    let constraints = PathConstraints { scale_factor: 1.0 };
    let (path, _metrics) =
        renderer.prepare(&cmds, PathStyle::Fill(FillStyle::default()), constraints);

    let mut scene = Scene::default();
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: (fret_core::scene::Paint::Material {
            id: fret_core::MaterialId::default(),
            params: fret_core::scene::MaterialParams {
                vec4s: [[1.0, 0.0, 0.0, 1.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            },
        })
        .into(),
    });

    let _ = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        super::RenderSceneParams {
            format,
            target_view: &target_view,
            source: super::RenderSceneSourceSelection::flat_compat(&scene),
            clear: super::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("perf snapshot");
    assert_eq!(snap.path_material_paints_degraded_to_solid_base, 1);
}
