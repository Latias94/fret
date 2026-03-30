use fret_core::text::TextCommonFallbackInjection;
use fret_core::{
    Color, FrameId, Px, TextAlign, TextConstraints, TextInput, TextOverflow, TextService, TextSpan,
    TextStyle, TextWrap,
};
use fret_render_wgpu::{Renderer, TextFontFamilyConfig, WgpuContext};

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

fn assert_metrics_match(measured: fret_core::TextMetrics, prepared: fret_core::TextMetrics) {
    let eps = 0.01;
    let dw = (measured.size.width.0 - prepared.size.width.0).abs();
    let dh = (measured.size.height.0 - prepared.size.height.0).abs();
    let db = (measured.baseline.0 - prepared.baseline.0).abs();

    assert!(
        dw <= eps && dh <= eps && db <= eps,
        "expected measure() to match prepare() metrics; measured={measured:?} prepared={prepared:?}"
    );
}

#[test]
fn text_measure_matches_prepare_across_fractional_scale_factors() {
    // This test must not rely on host-installed font availability.
    // `set_var` is `unsafe` on Rust 1.92+; we set it at the top of a dedicated test binary to
    // avoid cross-test races.
    unsafe {
        std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
    }

    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    configure_deterministic_fonts(&mut renderer);

    let mut style = TextStyle::default();
    style.size = Px(16.0);

    let text = "The quick brown fox jumps over the lazy dog.";
    let plain = TextInput::Plain {
        text: text.into(),
        style: style.clone(),
    };
    let attributed = TextInput::Attributed {
        text: text.into(),
        base: style,
        spans: vec![
            TextSpan {
                len: 10,
                shaping: Default::default(),
                paint: fret_core::TextPaintStyle {
                    fg: Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
            },
            TextSpan {
                len: text.len() - 10,
                shaping: Default::default(),
                paint: fret_core::TextPaintStyle {
                    fg: Some(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
            },
        ]
        .into(),
    };

    let scales = [1.0_f32, 1.25_f32, 1.5_f32];
    let widths = [Px(60.0), Px(90.0), Px(140.0)];

    for scale_factor in scales {
        for max_width in widths {
            for wrap in [TextWrap::Word, TextWrap::Grapheme] {
                let constraints = TextConstraints {
                    max_width: Some(max_width),
                    wrap,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::Start,
                    scale_factor,
                };

                for input in [&plain, &attributed] {
                    let measured = renderer.measure(input, constraints);
                    let (blob, prepared) = renderer.prepare(input, constraints);
                    renderer.release(blob);

                    assert_metrics_match(measured, prepared);
                }
            }
        }
    }
}
