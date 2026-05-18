use fret_core::geometry::Point;
use fret_core::scene::{Color, DrawOrder, Paint, Scene, SceneOp, TextOutlineV1};
use fret_core::text::TextCommonFallbackInjection;
use fret_core::{FrameId, Px, TextConstraints, TextInput, TextService, TextStyle, TextWrap};
use fret_render_wgpu::{Renderer, TextFontFamilyConfig, WgpuContext};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn configure_deterministic_fonts(renderer: &mut Renderer) {
    let added = renderer.add_fonts(fret_fonts::test_support::face_blobs(
        fret_fonts::default_profile().faces.iter(),
    ));
    assert!(added > 0, "expected bundled fonts to add at least one face");

    // Keep this test deterministic across machines:
    // - no host-installed system fonts,
    // - explicit font families,
    // - explicit common fallback injection.
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
fn gpu_text_outline_v1_renders_a_visible_ring_for_mask_glyphs() {
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
    style.size = Px(56.0);

    let input = TextInput::Plain {
        text: "Outline".into(),
        style,
    };
    let (blob, _metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );

    let origin = Point::new(Px(16.0), Px(92.0));
    let size = (320u32, 160u32);

    let fill_paint = Paint::Solid(Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    })
    .into();
    let outline_paint = Paint::Solid(Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    })
    .into();

    let mut fill_scene = Scene::default();
    fill_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: None,
        shadow: None,
    });

    let mut outline_scene = Scene::default();
    outline_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: Some(TextOutlineV1 {
            paint: outline_paint,
            width_px: Px(6.0),
        }),
        shadow: None,
    });

    let fill_pixels = render_scene_rgba8(&ctx, &mut renderer, &fill_scene, size, 1.0);
    let outline_pixels = render_scene_rgba8(&ctx, &mut renderer, &outline_scene, size, 1.0);

    let fill_max_alpha = fill_pixels
        .chunks_exact(4)
        .map(|px| px[3])
        .max()
        .unwrap_or(0);
    let outline_max_alpha = outline_pixels
        .chunks_exact(4)
        .map(|px| px[3])
        .max()
        .unwrap_or(0);
    assert!(
        fill_max_alpha > 0,
        "expected fill-only text to render at least one non-transparent pixel"
    );

    // Find any pixel where:
    // - fill-only is (near) transparent,
    // - outline scene is meaningfully non-transparent.
    //
    // We intentionally do not over-constrain by color here: text coverage + gamma correction can
    // make the ring's premultiplied channel values small at the edge.
    let mut found_new_coverage = false;
    let mut max_delta_alpha: u8 = 0;
    for y in 0..size.1 {
        for x in 0..size.0 {
            let a0 = pixel_rgba(&fill_pixels, size.0, x, y)[3];
            let a1 = pixel_rgba(&outline_pixels, size.0, x, y)[3];
            max_delta_alpha = max_delta_alpha.max(a1.saturating_sub(a0));
            if a0 <= 8 && a1 >= 16 {
                found_new_coverage = true;
                break;
            }
        }
        if found_new_coverage {
            break;
        }
    }
    assert!(
        found_new_coverage,
        "expected outline to add new coverage pixels (max_delta_alpha={max_delta_alpha}, fill_max_alpha={fill_max_alpha}, outline_max_alpha={outline_max_alpha})"
    );
}

#[test]
fn gpu_text_outline_v1_invalid_width_sanitizes_to_fill_only() {
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
    let (blob, _metrics) = renderer.prepare(
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
    let size = (256u32, 128u32);

    let fill_paint = Paint::Solid(Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    })
    .into();
    let outline_paint = Paint::Solid(Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    })
    .into();

    let mut fill_scene = Scene::default();
    fill_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: None,
        shadow: None,
    });

    let mut invalid_scene = Scene::default();
    invalid_scene.push(SceneOp::Text {
        order: DrawOrder(0),
        origin,
        text: blob,
        paint: fill_paint,
        outline: Some(TextOutlineV1 {
            paint: outline_paint,
            width_px: Px(0.0),
        }),
        shadow: None,
    });

    let fill_pixels = render_scene_rgba8(&ctx, &mut renderer, &fill_scene, size, 1.0);
    let invalid_pixels = render_scene_rgba8(&ctx, &mut renderer, &invalid_scene, size, 1.0);

    assert_eq!(
        fill_pixels, invalid_pixels,
        "expected invalid outline width to sanitize to fill-only output"
    );
}
