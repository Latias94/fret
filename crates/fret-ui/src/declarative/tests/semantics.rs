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
fn declarative_text_input_region_publishes_text_field_semantics_and_ranges_when_focused() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let region_id: std::cell::RefCell<Option<crate::GlobalElementId>> = Default::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-region",
        |cx| {
            let mut props = crate::element::TextInputRegionProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y_label = Some("Editor".into());
            props.a11y_value = Some("hello".into());
            props.a11y_text_selection = Some((2, 2));
            props.a11y_text_composition = Some((1, 3));

            let region = cx.text_input_region(props, |_cx| Vec::<AnyElement>::new());
            *region_id.borrow_mut() = Some(region.id);
            vec![region]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region_id = region_id.borrow().expect("region element id");
    let region =
        crate::elements::node_for_element(&mut app, window, region_id).expect("region node");
    ui.set_focus(Some(region));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    snap.validate().expect("semantics snapshot should validate");

    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::TextField && n.label.as_deref() == Some("Editor")
        })
        .expect("expected a TextField semantics node for the text input region");

    assert_eq!(node.value.as_deref(), Some("hello"));
    assert_eq!(node.text_selection, Some((2, 2)));
    assert_eq!(node.text_composition, Some((1, 3)));

    // When not focused, the ranges are cleared (label/value remain).
    ui.set_focus(None);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::TextField && n.label.as_deref() == Some("Editor")
        })
        .expect("expected a TextField semantics node for the text input region");
    assert_eq!(node.text_selection, None);
    assert_eq!(node.text_composition, None);
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

#[test]
fn declarative_semantics_can_be_focusable_and_receive_key_hooks() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let invoked = app.models_mut().insert(0u32);
    let mut semantics_id: Option<crate::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-focusable-semantics",
        |cx| {
            let invoked = invoked.clone();
            let semantics = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::List,
                    focusable: true,
                    ..Default::default()
                },
                |cx, id| {
                    semantics_id = Some(id);
                    let invoked = invoked.clone();
                    cx.key_on_key_down_for(
                        id,
                        Arc::new(move |host, _cx, down| {
                            if down.repeat || down.key != fret_core::KeyCode::ArrowDown {
                                return false;
                            }
                            let _ = host.models_mut().update(&invoked, |v: &mut u32| *v += 1);
                            true
                        }),
                    );
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: {
                                let mut layout = crate::element::LayoutStyle::default();
                                layout.size.width = crate::element::Length::Fill;
                                layout.size.height = crate::element::Length::Fill;
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("List container")],
                    )]
                },
            );
            vec![semantics]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics_id = semantics_id.expect("semantics element id");
    let semantics_node =
        crate::elements::node_for_element(&mut app, window, semantics_id).expect("semantics node");
    ui.set_focus(Some(semantics_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&invoked).unwrap_or_default(), 1);
}

#[test]
fn declarative_pressable_focusable_controls_focus_traversal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let mut first_id: Option<crate::GlobalElementId> = None;
    let mut second_id: Option<crate::GlobalElementId> = None;

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-focusable",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Px(Px(80.0));
            props.layout.size.height = Length::Px(Px(32.0));
            props.focusable = false;

            let first = cx.pressable_with_id(props, |cx, _st, id| {
                first_id = Some(id);
                vec![cx.text("first")]
            });

            let mut props2 = crate::element::PressableProps::default();
            props2.layout.size.width = Length::Px(Px(80.0));
            props2.layout.size.height = Length::Px(Px(32.0));
            props2.focusable = true;

            let second = cx.pressable_with_id(props2, |cx, _st, id| {
                second_id = Some(id);
                vec![cx.text("second")]
            });

            vec![cx.row(crate::element::RowProps::default(), move |_cx| {
                vec![first, second]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first_id = first_id.expect("first element id");
    let second_id = second_id.expect("second element id");

    let first_node =
        crate::elements::node_for_element(&mut app, window, first_id).expect("first node");
    let second_node =
        crate::elements::node_for_element(&mut app, window, second_id).expect("second node");

    let focusable = ui
        .first_focusable_descendant_including_declarative(&mut app, window, root)
        .expect("focusable pressable");
    assert_eq!(focusable, second_node);
    assert_ne!(focusable, first_node);
}
