use super::shaders::{
    BLIT_SHADER, BLUR_H_MASK_SHADER, BLUR_H_SHADER, BLUR_V_MASK_SHADER, BLUR_V_SHADER,
    COLOR_ADJUST_MASK_SHADER, COLOR_ADJUST_SHADER, COMPOSITE_PREMUL_MASK_SHADER,
    COMPOSITE_PREMUL_SHADER, DOWNSAMPLE_NEAREST_SHADER, MASK_SHADER, PATH_SHADER,
    TEXT_COLOR_SHADER, TEXT_SHADER, TEXT_SUBPIXEL_SHADER, UPSCALE_NEAREST_MASK_SHADER,
    UPSCALE_NEAREST_SHADER, VIEWPORT_SHADER, blur_h_masked_shader_source,
    blur_v_masked_shader_source, clip_mask_shader_source, color_adjust_masked_shader_source,
    quad_shader_source, upscale_nearest_masked_shader_source,
};
use super::{clamp_corner_radii_for_rect, svg_draw_rect_px};
use fret_core::geometry::{Point, Px, Transform2D};

#[test]
fn shaders_parse_as_wgsl() {
    let quad_src = quad_shader_source();
    let clip_mask_src = clip_mask_shader_source();
    let upscale_masked_src = upscale_nearest_masked_shader_source();
    let color_adjust_masked_src = color_adjust_masked_shader_source();
    let blur_h_masked_src = blur_h_masked_shader_source();
    let blur_v_masked_src = blur_v_masked_shader_source();
    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", quad_src.as_str()),
        ("blit", BLIT_SHADER),
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
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("composite_premul_mask", COMPOSITE_PREMUL_MASK_SHADER),
        ("clip_mask", clip_mask_src.as_str()),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
        ("text_subpixel", TEXT_SUBPIXEL_SHADER),
        ("mask", MASK_SHADER),
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
    let color_adjust_masked_src = color_adjust_masked_shader_source();
    let blur_h_masked_src = blur_h_masked_shader_source();
    let blur_v_masked_src = blur_v_masked_shader_source();
    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", quad_src.as_str()),
        ("blit", BLIT_SHADER),
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
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("composite_premul_mask", COMPOSITE_PREMUL_MASK_SHADER),
        ("clip_mask", clip_mask_src.as_str()),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
        ("text_subpixel", TEXT_SUBPIXEL_SHADER),
        ("mask", MASK_SHADER),
    ] {
        let module = naga::front::wgsl::parse_str(src)
            .unwrap_or_else(|err| panic!("WGSL parse failed for {name} shader: {err}"));
        Validator::new(ValidationFlags::all(), Capabilities::empty())
            .validate(&module)
            .unwrap_or_else(|err| panic!("WGSL validation failed for {name} shader: {err}"));
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
        scene: &scene,
        clear: super::ClearColor::default(),
        scale_factor: 1.0,
        viewport_size,
    };

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key0 = renderer
        .scene_encoding_cache_key
        .expect("scene encoding key");
    assert_eq!(renderer.perf.scene_encoding_cache_hits, 0);
    assert_eq!(renderer.perf.scene_encoding_cache_misses, 1);

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key1 = renderer
        .scene_encoding_cache_key
        .expect("scene encoding key");
    assert_eq!(key1, key0);
    assert_eq!(renderer.perf.scene_encoding_cache_hits, 1);

    let changed = renderer.set_text_quality_settings(crate::text::TextQualitySettings {
        gamma: 1.7,
        ..Default::default()
    });
    assert!(changed);

    let _ = renderer.render_scene(&ctx.device, &ctx.queue, make_params());
    let key2 = renderer
        .scene_encoding_cache_key
        .expect("scene encoding key");
    assert_ne!(key2, key0);
    assert_eq!(renderer.perf.scene_encoding_cache_misses, 2);
}
