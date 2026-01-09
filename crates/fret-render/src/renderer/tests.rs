use super::shaders::{
    BLIT_SHADER, BLUR_H_SHADER, BLUR_V_SHADER, COLOR_ADJUST_SHADER, COMPOSITE_PREMUL_SHADER,
    DOWNSAMPLE_NEAREST_SHADER, MASK_SHADER, PATH_SHADER, QUAD_SHADER, TEXT_COLOR_SHADER,
    TEXT_SHADER, UPSCALE_NEAREST_SHADER, VIEWPORT_SHADER,
};
use super::{clamp_corner_radii_for_rect, svg_draw_rect_px};
use fret_core::geometry::{Point, Px, Transform2D};

#[test]
fn shaders_parse_as_wgsl() {
    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", QUAD_SHADER),
        ("blit", BLIT_SHADER),
        ("blur_h", BLUR_H_SHADER),
        ("blur_v", BLUR_V_SHADER),
        ("downsample_nearest", DOWNSAMPLE_NEAREST_SHADER),
        ("upscale_nearest", UPSCALE_NEAREST_SHADER),
        ("color_adjust", COLOR_ADJUST_SHADER),
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
        ("mask", MASK_SHADER),
    ] {
        naga::front::wgsl::parse_str(src)
            .unwrap_or_else(|err| panic!("WGSL parse failed for {name} shader: {err}"));
    }
}

#[test]
fn shaders_validate_for_webgpu() {
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    for (name, src) in [
        ("viewport", VIEWPORT_SHADER),
        ("quad", QUAD_SHADER),
        ("blit", BLIT_SHADER),
        ("blur_h", BLUR_H_SHADER),
        ("blur_v", BLUR_V_SHADER),
        ("downsample_nearest", DOWNSAMPLE_NEAREST_SHADER),
        ("upscale_nearest", UPSCALE_NEAREST_SHADER),
        ("color_adjust", COLOR_ADJUST_SHADER),
        ("composite_premul", COMPOSITE_PREMUL_SHADER),
        ("path", PATH_SHADER),
        ("text", TEXT_SHADER),
        ("text_color", TEXT_COLOR_SHADER),
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
