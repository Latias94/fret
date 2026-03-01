use super::*;
use std::sync::Arc;

fn attributed_plain(text: &str) -> fret_core::AttributedText {
    fret_core::AttributedText::new(
        Arc::<str>::from(text),
        [fret_core::TextSpan {
            len: text.len(),
            ..Default::default()
        }],
    )
}

#[test]
fn selectable_text_paints_span_background_quads() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let text = "a code b";
    let bg = Color {
        r: 0.2,
        g: 0.4,
        b: 0.6,
        a: 1.0,
    };

    let spans = [
        fret_core::TextSpan {
            len: "a ".len(),
            ..Default::default()
        },
        fret_core::TextSpan {
            len: "code".len(),
            paint: fret_core::TextPaintStyle {
                bg: Some(bg),
                ..Default::default()
            },
            ..Default::default()
        },
        fret_core::TextSpan {
            len: " b".len(),
            ..Default::default()
        },
    ];
    let rich = fret_core::AttributedText::new(Arc::<str>::from(text), spans);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-paints-span-background-quads",
        |cx| {
            let mut props = crate::element::SelectableTextProps::new(rich.clone());
            // Force a fixed-height allocation so we exercise vertical placement mapping.
            props.layout.size.height = crate::element::Length::Px(Px(60.0));
            vec![cx.selectable_text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let quad_index = scene.ops().iter().position(|op| match op {
        SceneOp::Quad { background, .. } => *background == fret_core::Paint::Solid(bg).into(),
        _ => false,
    });
    let quad_y = scene.ops().iter().find_map(|op| match op {
        SceneOp::Quad {
            background, rect, ..
        } if *background == fret_core::Paint::Solid(bg).into() => Some(rect.origin.y),
        _ => None,
    });
    let text_index = scene
        .ops()
        .iter()
        .position(|op| matches!(op, SceneOp::Text { .. }));

    assert!(
        quad_index.is_some(),
        "expected a background quad for bg spans"
    );
    assert!(
        text_index.is_some(),
        "expected text op to be present in scene"
    );
    assert!(
        quad_index.unwrap() < text_index.unwrap(),
        "expected bg quads to be pushed before the text draw op"
    );

    // `FakeTextService` returns a 10px-tall metrics box; the element is 60px tall, so the text is
    // vertically centered with a 25px offset.
    let expected_vertical_offset = Px((60.0 - 10.0) * 0.5);
    assert_eq!(
        quad_y,
        Some(expected_vertical_offset),
        "expected bg quads to include the same vertical placement offset as the text draw op"
    );
}

#[test]
fn styled_text_paints_span_background_quads() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let text = "a code b";
    let bg = Color {
        r: 0.2,
        g: 0.4,
        b: 0.6,
        a: 1.0,
    };

    let spans = [
        fret_core::TextSpan {
            len: "a ".len(),
            ..Default::default()
        },
        fret_core::TextSpan {
            len: "code".len(),
            paint: fret_core::TextPaintStyle {
                bg: Some(bg),
                ..Default::default()
            },
            ..Default::default()
        },
        fret_core::TextSpan {
            len: " b".len(),
            ..Default::default()
        },
    ];
    let rich = fret_core::AttributedText::new(Arc::<str>::from(text), spans);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "styled-text-paints-span-background-quads",
        |cx| {
            let mut props = crate::element::StyledTextProps::new(rich.clone());
            // Force a fixed-height allocation so we exercise vertical placement mapping.
            props.layout.size.height = crate::element::Length::Px(Px(60.0));
            vec![cx.styled_text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let quad_index = scene.ops().iter().position(|op| match op {
        SceneOp::Quad { background, .. } => *background == fret_core::Paint::Solid(bg).into(),
        _ => false,
    });
    let quad_y = scene.ops().iter().find_map(|op| match op {
        SceneOp::Quad {
            background, rect, ..
        } if *background == fret_core::Paint::Solid(bg).into() => Some(rect.origin.y),
        _ => None,
    });
    let text_index = scene
        .ops()
        .iter()
        .position(|op| matches!(op, SceneOp::Text { .. }));

    assert!(
        quad_index.is_some(),
        "expected a background quad for bg spans"
    );
    assert!(
        text_index.is_some(),
        "expected text op to be present in scene"
    );
    assert!(
        quad_index.unwrap() < text_index.unwrap(),
        "expected bg quads to be pushed before the text draw op"
    );

    let expected_vertical_offset = Px((60.0 - 10.0) * 0.5);
    assert_eq!(
        quad_y,
        Some(expected_vertical_offset),
        "expected bg quads to include the same vertical placement offset as the text draw op"
    );
}

#[test]
fn selectable_text_pointer_hit_test_uses_text_local_coordinates() {
    struct HitTestRecordingServices {
        inner: FakeTextService,
        last_hit_test_point: Option<fret_core::Point>,
    }

    impl fret_core::TextService for HitTestRecordingServices {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            self.inner.prepare(input, constraints)
        }

        fn hit_test_point(
            &mut self,
            _blob: fret_core::TextBlobId,
            point: fret_core::Point,
        ) -> fret_core::HitTestResult {
            self.last_hit_test_point = Some(point);
            fret_core::HitTestResult {
                index: 0,
                affinity: fret_core::CaretAffinity::Downstream,
            }
        }

        fn release(&mut self, blob: fret_core::TextBlobId) {
            self.inner.release(blob);
        }

        fn selection_rects_clipped(
            &mut self,
            blob: fret_core::TextBlobId,
            range: (usize, usize),
            clip: fret_core::Rect,
            out: &mut Vec<fret_core::Rect>,
        ) {
            self.inner.selection_rects_clipped(blob, range, clip, out);
        }
    }

    impl fret_core::PathService for HitTestRecordingServices {
        fn prepare(
            &mut self,
            commands: &[fret_core::PathCommand],
            style: fret_core::PathStyle,
            constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            fret_core::PathService::prepare(&mut self.inner, commands, style, constraints)
        }

        fn release(&mut self, path: fret_core::PathId) {
            fret_core::PathService::release(&mut self.inner, path);
        }
    }

    impl fret_core::SvgService for HitTestRecordingServices {
        fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
            self.inner.register_svg(bytes)
        }

        fn unregister_svg(&mut self, svg: fret_core::SvgId) -> bool {
            self.inner.unregister_svg(svg)
        }
    }

    impl fret_core::MaterialService for HitTestRecordingServices {
        fn register_material(
            &mut self,
            desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            self.inner.register_material(desc)
        }

        fn unregister_material(&mut self, id: fret_core::MaterialId) -> bool {
            self.inner.unregister_material(id)
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = HitTestRecordingServices {
        inner: FakeTextService::default(),
        last_hit_test_point: None,
    };

    let rich = attributed_plain("hello");
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-hit-test-mapping",
        |cx| {
            let mut props = crate::element::SelectableTextProps::new(rich.clone());
            props.layout.size.height = crate::element::Length::Px(Px(60.0));
            vec![cx.selectable_text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selectable_node = ui.children(root)[0];
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");

    let expected_vertical_offset = Px((60.0 - 10.0) * 0.5);
    let click_pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + expected_vertical_offset.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: click_pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        services.last_hit_test_point,
        Some(Point::new(Px(5.0), Px(1.0))),
        "expected pointer hit-test to pass text-local coordinates (subtracting vertical placement offset)"
    );
}

#[test]
fn selectable_text_clamps_selection_indices_on_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let text = "a👨‍👩‍👧‍👦b";
    let rich = attributed_plain(text);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-clamps-selection-indices-on-paint",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| {
            state.selection_anchor = text.len() + 123;
            state.caret = 2;
        },
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (anchor, caret) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );

    assert!(
        anchor <= text.len() && caret <= text.len(),
        "expected selection indices to be clamped to the text length"
    );
    assert!(
        crate::text_edit::utf8::is_grapheme_boundary(text, anchor)
            && crate::text_edit::utf8::is_grapheme_boundary(text, caret),
        "expected selection indices to be clamped to grapheme boundaries"
    );
    assert_eq!(anchor, text.len(), "expected out-of-range anchor to clamp");
    assert_eq!(caret, 1, "expected mid-sequence caret to clamp down");
}

#[test]
fn selectable_text_clamps_selection_indices_before_copy_shortcut() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let text = "hello world";
    let rich = attributed_plain(text);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-clamps-selection-indices-before-copy-shortcut",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| {
            state.selection_anchor = 0;
            state.caret = text.len() + 999;
        },
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::KeyC,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
            repeat: false,
        },
    );

    assert!(
        app.take_effects().iter().any(
            |e| matches!(e, fret_runtime::Effect::ClipboardSetText { text: copied } if copied == text)
        ),
        "expected ctrl+c to copy the clamped selection"
    );
}
