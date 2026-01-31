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
