use super::*;

#[test]
fn declarative_text_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text",
        |cx| vec![cx.text("Hello declarative")],
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    // Root is a host widget, so text is in a descendant; ensure at least one Text node carries
    // the label payload.
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some("Hello declarative")),
        "expected a Text semantics node with label"
    );
}

#[test]
fn declarative_text_input_sets_semantics_label() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-label",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model);
            props.a11y_label = Some("Search".into());
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::TextField
                && n.label.as_deref() == Some("Search")),
        "expected a TextField semantics node with label"
    );
}

#[test]
fn declarative_text_input_respects_a11y_role_override_and_expanded() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-role-override",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model);
            props.a11y_label = Some("Combobox".into());
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.expanded = Some(true);
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::ComboBox
                && n.flags.expanded
                && n.label.as_deref() == Some("Combobox")
                && n.value.as_deref() == Some("hello")
        }),
        "expected a ComboBox semantics node with expanded=true and correct label/value"
    );
}

#[test]
fn declarative_text_area_updates_model_on_text_input() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-text-input",
        |cx| {
            let mut props = crate::element::TextAreaProps::new(model.clone());
            props.min_height = Px(80.0);
            vec![cx.text_area(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable text area");
    ui.set_focus(Some(focusable));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("hello\nworld".to_string()),
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("hello\nworld")
    );
}
