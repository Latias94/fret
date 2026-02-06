use super::*;

#[test]
fn escape_dismisses_topmost_overlay_without_focus() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let dismissed = app.models_mut().insert(false);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(TestStack);
    ui.set_root(base);

    let overlay = ui.create_node(TestStack);
    let overlay_element = crate::GlobalElementId(0xdead_beef);
    ui.set_node_element(overlay, Some(overlay_element));
    let _layer = ui.push_overlay_root_ex(overlay, false, true);

    crate::elements::with_element_state(
        &mut app,
        window,
        overlay_element,
        crate::action::DismissibleActionHooks::default,
        |hooks| {
            let dismissed = dismissed.clone();
            hooks.on_dismiss_request = Some(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, crate::action::DismissReason::Escape);
                let _ = host.models_mut().update(&dismissed, |v| *v = true);
            }));
        },
    );

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.models().get_copied(&dismissed).unwrap_or(false),
        "expected Escape to route to the topmost dismissible overlay"
    );
}
