use super::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[test]
fn theme_color_change_does_not_reprepare_text_in_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    // Ensure the theme is stored as a global so we can mutate it between frames.
    app.set_global(crate::Theme::global(&app).clone());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-cache",
        |cx| vec![cx.keyed(1u64, |cx| cx.text("hello"))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepares_after_first_paint = services.prepare_calls;

    // Paint-only theme change: should not invalidate the text blob cache path.
    crate::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = crate::ThemeConfig::default();
        cfg.colors
            .insert("foreground".to_string(), "#ff0000".to_string());
        theme.extend_tokens_from_config(&cfg);
    });

    // Intentionally skip `render_root`/`layout_all` so the only possible text service work is from paint.
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(
        services.prepare_calls, prepares_after_first_paint,
        "paint-only theme changes should not force re-preparing text blobs"
    );
}

fn fingerprint_text_style(style: &TextStyle, h: &mut impl Hasher) {
    style.font.hash(h);
    style.size.0.to_bits().hash(h);
    style.weight.hash(h);
    style.slant.hash(h);
    style.line_height.map(|v| v.0.to_bits()).hash(h);
    style.letter_spacing_em.map(f32::to_bits).hash(h);
}

fn fingerprint_shaping_style(style: &fret_core::TextShapingStyle, h: &mut impl Hasher) {
    style.font.hash(h);
    style.weight.hash(h);
    style.slant.hash(h);
    style.letter_spacing_em.map(f32::to_bits).hash(h);
}

fn fingerprint_text_input(input: &fret_core::TextInput) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    input.text().hash(&mut h);
    match input {
        fret_core::TextInput::Plain { style, .. } => {
            0u8.hash(&mut h);
            fingerprint_text_style(style, &mut h);
        }
        fret_core::TextInput::Attributed { base, spans, .. } => {
            1u8.hash(&mut h);
            fingerprint_text_style(base, &mut h);
            for span in spans.iter() {
                span.len.hash(&mut h);
                fingerprint_shaping_style(&span.shaping, &mut h);
            }
        }
        _ => {
            2u8.hash(&mut h);
        }
    }
    h.finish()
}

#[derive(Default)]
struct FingerprintingServices {
    calls: Vec<(String, u64)>,
}

impl TextService for FingerprintingServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.calls
            .push((input.text().to_string(), fingerprint_text_input(input)));
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FingerprintingServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for FingerprintingServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for FingerprintingServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[test]
fn theme_color_change_does_not_change_text_input_fingerprints() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FingerprintingServices::default();

    app.set_global(crate::Theme::global(&app).clone());

    let rich_a = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("styled"),
        [fret_core::TextSpan {
            len: "styled".len(),
            ..Default::default()
        }],
    );
    let rich_b = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("selectable"),
        [fret_core::TextSpan {
            len: "selectable".len(),
            ..Default::default()
        }],
    );

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-fingerprint",
        move |cx| {
            vec![
                cx.text("plain"),
                cx.styled_text(rich_a.clone()),
                cx.selectable_text(rich_b.clone()),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut baseline: HashMap<String, u64> = HashMap::new();
    for (text, fp) in services.calls.drain(..) {
        if let Some(existing) = baseline.get(&text) {
            assert_eq!(
                *existing, fp,
                "expected measure/paint to use the same text input fingerprint for {text:?}"
            );
        } else {
            baseline.insert(text, fp);
        }
    }

    crate::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = crate::ThemeConfig::default();
        cfg.colors
            .insert("foreground".to_string(), "#00ff00".to_string());
        theme.extend_tokens_from_config(&cfg);
    });

    // Paint-only theme change: should not force any new text preparation.
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services.calls.is_empty(),
        "expected paint-only theme changes to avoid re-preparing any text blobs"
    );
}
