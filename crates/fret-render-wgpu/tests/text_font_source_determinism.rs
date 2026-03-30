use fret_core::text::TextCommonFallbackInjection;
use fret_core::{FrameId, TextConstraints, TextInput, TextService, TextStyle, TextWrap};
use fret_render_wgpu::{Renderer, TextFontFamilyConfig, WgpuContext};

#[test]
fn text_can_run_with_system_fonts_disabled_and_bundled_fonts_injected() {
    // This test locks down the deterministic "no system fonts" path for text conformance gates.
    // It must not rely on host-installed font availability.
    //
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

    // Prepare a trivial blob to ensure the shaping path is exercised under the configured stacks.
    let input = TextInput::Plain {
        text: "Hello".into(),
        style: TextStyle::default(),
    };
    let (_blob, _metrics) = renderer.prepare(
        &input,
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: Default::default(),
            align: Default::default(),
            scale_factor: 1.0,
        },
    );
}
