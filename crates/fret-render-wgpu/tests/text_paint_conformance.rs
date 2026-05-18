use fret_core::geometry::Point;
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Paint, Scene, SceneOp,
    TextShadowV1, TileMode,
};
use fret_core::text::TextCommonFallbackInjection;
use fret_core::{FrameId, Px, TextConstraints, TextInput, TextService, TextStyle, TextWrap};
use fret_render_wgpu::{Renderer, TextFontFamilyConfig, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn configure_deterministic_fonts(renderer: &mut Renderer) {
    let added = renderer.add_fonts(fret_fonts::test_support::face_blobs(
        fret_fonts::default_profile().faces.iter(),
    ));
    assert!(added > 0, "expected bundled fonts to add at least one face");

    let mut families = TextFontFamilyConfig::default();
    families.ui_sans = vec!["Inter".to_string()];
    families.ui_mono = vec!["JetBrains Mono".to_string()];
    families.common_fallback_injection = TextCommonFallbackInjection::CommonFallback;
    renderer.set_text_font_families(&families);

    let snap = renderer.text_fallback_policy_snapshot(FrameId(1));
    assert!(
        !snap.system_fonts_enabled,
        "expected system fonts to be disabled via FRET_TEXT_SYSTEM_FONTS=0"
    );
    assert_ne!(snap.font_stack_key, 0, "expected a non-zero font stack key");
}

#[test]
fn gpu_text_linear_gradient_paint_varies_across_x() {
    // This test must not rely on host-installed font availability.
    // `set_var` is `unsafe` on Rust 1.92+; we set it at the top of a dedicated test binary to
    // avoid cross-test races.
    unsafe {
        std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
    }

    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    configure_deterministic_fonts(&mut renderer);

    let mut style = TextStyle::default();
    style.size = Px(48.0);

    let input = TextInput::Plain {
        text: "Hello".into(),
        style,
    };
    let (blob, metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );

    // `origin` is the baseline origin; keep it well inside the viewport so glyph ascent isn't
    // clipped away on different font metrics.
    let origin = Point::new(Px(16.0), Px(80.0));
    let start = origin;
    let end = Point::new(Px(origin.x.0 + metrics.size.width.0), origin.y);

    let (stops, stop_count) = stops_2(
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );
    let gradient = LinearGradient {
        start,
        end,
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count,
        stops,
    };

    let mut scene = Scene::default();
    scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: (Paint::LinearGradient(gradient)).into(),
        outline: None,
        shadow: None,
    });

    let size = (256u32, 128u32);
    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let global_max_alpha = pixels.chunks_exact(4).map(|px| px[3]).max().unwrap_or(0);

    // `SceneOp::Text.origin` is the baseline origin; `metrics.baseline` is the baseline offset
    // from the top of the text box.
    let top_y = origin.y.0 - metrics.baseline.0;

    let x0 = origin.x.0.max(0.0) as u32;
    let y0 = top_y.max(0.0) as u32;
    let x1 = (origin.x.0 + metrics.size.width.0).min(size.0 as f32) as u32;
    let y1 = (top_y + metrics.size.height.0).min(size.1 as f32) as u32;

    assert!(x1 > x0 + 4, "expected non-empty text bounds");
    assert!(y1 > y0 + 4, "expected non-empty text bounds");

    let mut best_y = vec![0u32; (x1 - x0) as usize];
    let mut best_a = vec![0u8; (x1 - x0) as usize];
    for (i, x) in (x0..x1).enumerate() {
        let mut max_a = 0u8;
        let mut max_y = y0;
        for y in y0..y1 {
            let a = pixel_rgba(&pixels, size.0, x, y)[3];
            if a > max_a {
                max_a = a;
                max_y = y;
            }
        }
        best_a[i] = max_a;
        best_y[i] = max_y;
    }

    let max_alpha_in_bounds = *best_a.iter().max().unwrap_or(&0);
    assert!(
        max_alpha_in_bounds > 0,
        "expected some non-zero alpha within text bounds (max_alpha_in_bounds={max_alpha_in_bounds}, global_max_alpha={global_max_alpha})"
    );

    let alpha_threshold_detect = 5u8;
    let alpha_threshold_assert = 80u8;
    let left_i = best_a
        .iter()
        .position(|a| *a >= alpha_threshold_detect)
        .expect("expected at least one glyph-covered column");
    let right_i = best_a
        .iter()
        .rposition(|a| *a >= alpha_threshold_detect)
        .expect("expected at least one glyph-covered column");

    assert!(right_i > left_i + 4, "expected glyph coverage width > 4px");

    let left_x = x0 + left_i as u32;
    let right_x = x0 + right_i as u32;
    let left_px = pixel_rgba(&pixels, size.0, left_x, best_y[left_i]);
    let right_px = pixel_rgba(&pixels, size.0, right_x, best_y[right_i]);

    assert!(
        left_px[3] >= alpha_threshold_assert && right_px[3] >= alpha_threshold_assert,
        "expected opaque-ish glyph pixels: left={left_px:?} right={right_px:?}"
    );

    // With a left-to-right red→blue gradient, the leftmost glyph pixel should be more red and
    // the rightmost glyph pixel should be more blue (premul is fine; alpha is comparable here).
    assert!(
        left_px[0] > right_px[0],
        "expected red to decrease across x: left={left_px:?} right={right_px:?}"
    );
    assert!(
        right_px[2] > left_px[2],
        "expected blue to increase across x: left={left_px:?} right={right_px:?}"
    );
}

#[test]
fn gpu_text_shadow_v1_renders_a_separate_layer() {
    unsafe {
        std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
    }

    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    configure_deterministic_fonts(&mut renderer);

    let mut style = TextStyle::default();
    style.size = Px(48.0);

    let input = TextInput::Plain {
        text: "Shadow".into(),
        style,
    };
    let (blob, metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );

    let origin = Point::new(Px(16.0), Px(80.0));
    let top_y = origin.y.0 - metrics.baseline.0;
    let height = metrics.size.height.0;

    // Offset the shadow far enough so it doesn't overlap the main glyphs.
    let shadow_dx = Px(metrics.size.width.0 + 16.0);
    let shadow_offset = Point::new(shadow_dx, Px(0.0));

    let mut scene = Scene::default();
    scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: (Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }))
        .into(),
        outline: None,
        shadow: Some(TextShadowV1::new(
            shadow_offset,
            Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
        )),
    });

    let size = (512u32, 160u32);
    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    fn max_alpha_pixel_in_bounds(
        pixels: &[u8],
        width: u32,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
    ) -> (u32, u32, [u8; 4]) {
        let mut best_a = 0u8;
        let mut best_xy = (x0, y0);
        let mut best_px = [0u8; 4];
        for y in y0..y1 {
            for x in x0..x1 {
                let px = pixel_rgba(pixels, width, x, y);
                if px[3] > best_a {
                    best_a = px[3];
                    best_xy = (x, y);
                    best_px = px;
                }
            }
        }
        (best_xy.0, best_xy.1, best_px)
    }

    let x0 = origin.x.0.max(0.0) as u32;
    let y0 = top_y.max(0.0) as u32;
    let x1 = (origin.x.0 + metrics.size.width.0).min(size.0 as f32) as u32;
    let y1 = (top_y + height).min(size.1 as f32) as u32;
    assert!(x1 > x0 + 4 && y1 > y0 + 4, "expected non-empty text bounds");

    let shadow_x0 = (origin.x.0 + shadow_offset.x.0).max(0.0) as u32;
    let shadow_y0 = (top_y + shadow_offset.y.0).max(0.0) as u32;
    let shadow_x1 =
        (origin.x.0 + shadow_offset.x.0 + metrics.size.width.0).min(size.0 as f32) as u32;
    let shadow_y1 = (top_y + shadow_offset.y.0 + height).min(size.1 as f32) as u32;
    assert!(
        shadow_x1 > shadow_x0 + 4 && shadow_y1 > shadow_y0 + 4,
        "expected non-empty shadow bounds"
    );

    let (_tx, _ty, text_px) = max_alpha_pixel_in_bounds(&pixels, size.0, x0, y0, x1, y1);
    let (_sx, _sy, shadow_px) =
        max_alpha_pixel_in_bounds(&pixels, size.0, shadow_x0, shadow_y0, shadow_x1, shadow_y1);

    assert!(
        text_px[3] >= 80 && shadow_px[3] >= 80,
        "expected visible glyph coverage for both layers: text={text_px:?} shadow={shadow_px:?}"
    );
    assert!(
        text_px[0] > text_px[2].saturating_add(32),
        "expected main text to be red-ish: text={text_px:?}"
    );
    assert!(
        shadow_px[2] > shadow_px[0].saturating_add(32),
        "expected shadow to be blue-ish: shadow={shadow_px:?}"
    );
}
